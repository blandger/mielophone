use crate::bbit::control::{ControlPoint, ControlPointCommand};
use crate::bbit::eeg_uuids::{
    EventType, NotifyStream, NotifyUuid, FIRMWARE_REVISION_STRING_UUID,
    HARDWARE_REVISION_STRING_UUID, MODEL_NUMBER_STRING_UUID, PERIPHERAL_NAME_MATCH_FILTER,
    SERIAL_NUMBER_STRING_UUID, WRITE_COMMAN_UUID,
};
use crate::bbit::responses::DeviceInfo;
use crate::bbit::sealed::{Bluetooth, Configure, Connected, EventLoop, Level};
use crate::{find_characteristic, Error, EventHandler};
use btleplug::api::WriteType;
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use futures::stream::StreamExt;
use futures::AsyncReadExt;
use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{instrument, Event};
use uuid::Uuid;

pub type BBitResult<T> = Result<T, Error>;

/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct CommandData {
    data: i16,
    cmd_type: ControlPointCommand,
}

/// The core sensor manager
pub struct BleSensor<L: Level> {
    /// BLE connection manager
    ble_manager: Manager,
    /// Connected and controlled device
    ble_device: Option<Peripheral>,
    /// BLE event types subscribed and processed
    pub subscribed_data_event_types: Vec<EventType>,
    /// Device manage and send commands
    pub control_point: Option<ControlPoint>,
    level: L,
    /// Common device information like model, serial numbers, HW, SW revisions
    pub device_info: OnceLock<DeviceInfo>,
}

impl BleSensor<Bluetooth> {
    /// Construct a BleSensor
    pub async fn new() -> BBitResult<Self> {
        Ok(Self {
            ble_manager: Manager::new().await?,
            ble_device: None,
            subscribed_data_event_types: vec![],
            control_point: None,
            level: Bluetooth,
            device_info: OnceLock::new(),
        })
    }

    /// Connect to a device. Blocks until a connection is found
    #[instrument(skip(self))]
    pub async fn block_connect(mut self, device_id: &str) -> BBitResult<BleSensor<Configure>> {
        while !self.is_connected().await {
            match self.try_connect(device_id).await {
                Err(e @ Error::NoBleAdaptor) => {
                    tracing::error!("No bluetooth adaptors found");
                    return Err(e);
                }
                Err(e) => tracing::warn!("could not connect: {}", e),
                Ok(_) => {}
            }
        }
        tracing::debug!("BLE is connected...");

        let new_self: BleSensor<Configure> = BleSensor {
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            subscribed_data_event_types: self.subscribed_data_event_types,
            level: Configure::default(),
            device_info: self.device_info,
        };

        Ok(new_self)
    }

    /// Connect to a device, but override the behavior after each attempted connect
    /// Return [`Ok`] from the closure to continue trying to connect or [`Err`]
    /// give up and return.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn main() {
    /// use lib::Error;
    /// use lib::bbit::device::BleSensor;
    ///
    /// let mut bbit = BleSensor::new().await.unwrap()
    ///     // default handling that is applied to BleSensor::block_connect
    ///     .map_connect("BrainBit", |r| {
    ///         match r {
    ///             Err(e @ Error::NoBleAdaptor) => {
    ///                 tracing::error!("no bluetooth adaptors found");
    ///                 return Err(e);
    ///             }
    ///             Err(e) => tracing::warn!("could not connect: {}", e),
    ///             Ok(_) => {}
    ///         };
    ///         Ok(())
    ///     }).await.unwrap();
    /// # }
    /// ```
    #[instrument(skip(self, f))]
    pub async fn map_connect<F>(
        mut self,
        device_id: &str,
        mut f: F,
    ) -> BBitResult<BleSensor<Configure>>
    where
        F: FnMut(BBitResult<()>) -> BBitResult<()>,
    {
        while !self.is_connected().await {
            if let Err(e) = f(self.try_connect(device_id).await) {
                return Err(e);
            }
        }
        let new_self: BleSensor<Configure> = BleSensor {
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            subscribed_data_event_types: self.subscribed_data_event_types,
            level: Configure::default(),
            device_info: self.device_info,
        };

        Ok(new_self)
    }

    async fn is_connected(&self) -> bool {
        // async in iterators when?
        // self.ble_device
        //     .and_then(|d| d.is_connected().await.ok())
        //     .ok_or(false)
        if let Some(device) = &self.ble_device {
            if let Ok(v) = device.is_connected().await {
                return v;
            }
        }
        false
    }

    /// Try to connect to a device. Implements the [`crate::BleSensor::connect`] function
    #[instrument(skip(self))]
    async fn try_connect(&mut self, device_name: &str) -> BBitResult<()> {
        tracing::debug!("trying to connect");
        let adapters = self
            .ble_manager
            .adapters()
            .await
            .map_err(|_| Error::NoBleAdaptor)?;
        let Some(central) = adapters.first() else {
            tracing::error!("No ble adaptor found");
            return Err(Error::NoBleAdaptor);
        };

        tracing::debug!("start scanning for 2 sec...");
        central.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        for p in central.peripherals().await? {
            if p.properties()
                .await?
                .unwrap()
                .local_name
                .iter()
                .any(|name| {
                    name.starts_with(PERIPHERAL_NAME_MATCH_FILTER) || name.starts_with(device_name)
                })
            {
                self.ble_device = Some(p);
                break;
            }
        }

        let Some(device) = &self.ble_device else {
            tracing::warn!("device not found");
            return Err(Error::NoDevice);
        };
        tracing::debug!("BLE is found, try to connect...");

        device.connect().await?;
        tracing::debug!("Try to discover...");
        device.discover_services().await?;

        let controller = ControlPoint::new(device).await?;
        self.control_point = Some(controller);

        Ok(())
    }
}

impl BleSensor<EventLoop> {
    /// Start the event loop
    #[instrument(skip_all)]
    pub async fn event_loop<H: EventHandler + Sync + Send + 'static>(
        self,
        handler: H,
    ) -> BleHandle {
        tracing::info!("starting measurements");

        // look for subscribed events
        for event_type in &self.subscribed_data_event_types {
            // use EventType::*;
            // TODO: send eeg config data
        }
        let bt_sensor = Arc::new(self);
        let event_sensor = Arc::clone(&bt_sensor);
        tracing::info!("starting bluetooth task");
        let (bt_tx, mut bt_rx) = mpsc::channel(128);
        let (pause_tx, pause_rx) = watch::channel(false);

        tokio::task::spawn(async move {
            let device = bt_sensor.ble_device.as_ref().unwrap();
            let mut notification_stream = device.notifications().await?;

            while let Some(data) = notification_stream.next().await {
                tracing::debug!("received bluetooth data: {data:?}");
                if *pause_rx.borrow() {
                    tracing::debug!("paused: ignoring data");
                    continue;
                }
                if data.uuid == NotifyUuid::BatteryLevel.into() {
                    let battery = data.value[0];
                    let Ok(_) = bt_tx.send(BluetoothEvent::Battery(battery)).await else { break };
                } else if data.uuid == NotifyUuid::EegMeasurement.into() {
                    let eeg = data.value;
                    let Ok(_) = bt_tx.send(BluetoothEvent::Egg(eeg)).await else { break };
                } else if data.uuid == NotifyUuid::ResistanceMeasurement.into() {
                    let resist = data.value;
                    let Ok(_) = bt_tx.send(BluetoothEvent::Resistance(resist)).await else { break };
                }
            }

            Ok::<_, Error>(())
        });

        tracing::info!("starting event task");
        let (event_tx, mut event_rx) = mpsc::channel(4);
        tokio::task::spawn(async move {
            loop {
                tokio::select! {
                    Some(data) = bt_rx.recv() => {
                        tracing::debug!("received bt channel message");
                        use BluetoothEvent::*;
                        match data {
                            Battery(bat) => handler.battery_update(bat).await,
                            Egg(eeg) => handler.eeg_update(eeg).await,
                            Resistance(resist) => handler.resistance_update(resist).await,
                        }
                    }
                    Some(event) = event_rx.recv() => {
                        tracing::debug!("received event: {event:?}");
                        match event {
                            BleDeviceEvent::Stop => {
                                break;
                            }
                            BleDeviceEvent::Start => {
                            // BleDeviceEvent::Add { ty, ret } => {
                            //     let res = event_sensor.get_pmd_response(ControlPointCommand::RequestMeasurementStart, ty).await;
                            //     let _ = ret.send(res);
                            }
                            BleDeviceEvent::Resistance => {
                            // BleDeviceEvent::Remove { ty, ret } => {
                            //     let res = event_sensor.get_pmd_response(ControlPointCommand::StopMeasurement, ty).await;
                            //     let _ = ret.send(res);
                            }
                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        });

        BleHandle::new(event_tx, pause_tx)
    }
}

impl<L: Level + Connected> BleSensor<L> {
    #[instrument(skip(self))]
    async fn subscribe(&self, ty: NotifyStream) -> BBitResult<()> {
        tracing::info!("subscribing to '{:?}'", ty);
        let device = self.ble_device.as_ref().expect("device already connected");

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == ty.into())
            .ok_or(Error::CharacteristicNotFound)?;

        device.subscribe(&characteristic).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn unsubscribe(&self, ty: NotifyStream) -> BBitResult<()> {
        tracing::info!("unsubscribing from '{ty:?}'");
        let device = self.ble_device.as_ref().unwrap();

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == ty.into())
            .ok_or(Error::CharacteristicNotFound)?;

        device.unsubscribe(&characteristic).await?;

        Ok(())
    }

    /// Fetch the characteristics of the device
    pub fn characteristics(&self) -> BTreeSet<Characteristic> {
        let device = self.ble_device.as_ref().unwrap();
        device.characteristics()
    }

    /// Read the battery level of the device
    #[instrument(skip_all)]
    pub async fn battery(&self) -> BBitResult<u8> {
        tracing::info!("fetching battery level");
        let device = self.ble_device.as_ref().unwrap();

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == NotifyStream::from(EventType::Battery).into())
            .ok_or(Error::CharacteristicNotFound)?;

        let bytes = device.read(&characteristic).await?;
        let byte = bytes[0];
        tracing::debug!("read {} bytes: {bytes:x?}", bytes.len());

        Ok(u8::from_le_bytes([byte]))
    }

    /// Read the internal device info - model, serial, SW, HW revision
    #[instrument(skip(self))]
    pub async fn device_info(&self) -> BBitResult<DeviceInfo> {
        tracing::info!("fetching device info...");
        // on time initialization
        if self.device_info.get().is_none() {
            let model_number = self.read_string(MODEL_NUMBER_STRING_UUID).await?;
            let serial_number = self.read_string(SERIAL_NUMBER_STRING_UUID).await?;
            let hardware_revision = self.read_string(HARDWARE_REVISION_STRING_UUID).await?;
            let firmware_revision = self.read_string(FIRMWARE_REVISION_STRING_UUID).await?;
            let device_info = DeviceInfo::new(
                model_number,
                serial_number,
                hardware_revision,
                firmware_revision,
            );
            let _ = self.device_info.set(device_info);
        }
        tracing::debug!("device info: '{:?}'", self.device_info.get());
        Ok(self.device_info.get().unwrap().clone())
    }

    /// low level reading bytes as String
    async fn read_string(&self, uuid: Uuid) -> BBitResult<String> {
        let data = self.read(uuid).await?;

        let string = String::from_utf8_lossy(&data).into_owned();
        Ok(string.trim_matches(char::from(0)).to_string())
    }

    async fn read(&self, uuid: Uuid) -> BBitResult<Vec<u8>> {
        let device = self.ble_device.as_ref().unwrap();
        // let device = self.device().await?;
        if let Ok(char) = find_characteristic(device, uuid).await {
            return device.read(&char).await.map_err(Error::BleError);
        }
        Err(Error::CharacteristicNotFound)
    }

    /// Send command as enum to [`ControlPoint`].
    #[instrument(skip(self))]
    pub async fn send_command(&self, command: ControlPointCommand) -> BBitResult<()> {
        let control_point = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();

        control_point
            .send_control_command_enum(device, &command)
            .await?;
        Ok(())
    }

    async fn stop_measurement(&self) -> BBitResult<()> {
        let controller = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();
        controller
            .send_control_command_enum(&device, &ControlPointCommand::CommandStop)
            .await?;
        Ok(())
    }

    async fn start_resist_measurement(&self) -> BBitResult<()> {
        let controller = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();
        controller
            .send_control_command_enum(&device, &ControlPointCommand::CommandStartResist)
            .await?;
        Ok(())
    }

    async fn start_signal_measurement(&self) -> BBitResult<()> {
        let controller = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();
        controller
            .send_control_command_enum(&device, &ControlPointCommand::CommandStartSignal)
            .await?;
        Ok(())
    }
}

/// Handle to the [`BleSensor`] that is running an event loop
#[derive(Clone)]
pub struct BleHandle {
    sender: mpsc::Sender<BleDeviceEvent>,
    pause: Arc<watch::Sender<bool>>,
}

impl BleHandle {
    fn new(sender: mpsc::Sender<BleDeviceEvent>, pause: watch::Sender<bool>) -> Self {
        Self {
            sender,
            pause: Arc::new(pause),
        }
    }
}

/// Type of events sent to the event loop from [`BleSensor`]
#[derive(Debug)]
enum BleDeviceEvent {
    /// Send config command to BleSensor and start the event loop
    Start,
    /// Stop the event loop
    Stop,
    ///
    /// Start resistance measurement
    Resistance,
}

/// Bluetooth data received from the sensor
#[derive(Debug)]
enum BluetoothEvent {
    Battery(u8),
    Egg(Vec<u8>),
    Resistance(Vec<u8>),
}
