use crate::bbit::control::{ControlPoint, ControlPointCommand};
use crate::bbit::eeg_uuids::{
    EventType, NotifyStream, NotifyUuid, FIRMWARE_REVISION_STRING_UUID,
    HARDWARE_REVISION_STRING_UUID, MODEL_NUMBER_STRING_UUID, NSS2_SERVICE_UUID,
    SERIAL_NUMBER_STRING_UUID,
};
use crate::bbit::responses::{DeviceInfo, DeviceStatusData, Nss2Status};
use crate::bbit::sealed::{Bluetooth, Configure, Connected, EventLoop, Level};
use crate::{find_characteristic, Error, EventHandler};

use crate::bbit::{ADS1294ChannelInput, MeasurementType};
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use futures::stream::StreamExt;
use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::instrument;
use uuid::Uuid;

pub type BBitResult<T> = Result<T, Error>;

/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct CommandData {
    data: i16,
    cmd_type: ControlPointCommand,
}

/// The core sensor manager
pub struct BBitSensor<L: Level> {
    /// BLE connection manager
    ble_manager: Manager,
    /// Connected and controlled device
    ble_device: Option<Peripheral>,
    /// BLE event types subscribed and processed
    pub subscribed_data_event_types: Vec<EventType>,
    /// Device manage and send commands
    pub control_point: Option<ControlPoint>,
    pub level: L,
    /// Common device information like model, serial numbers, HW, SW revisions
    pub device_info: OnceLock<DeviceInfo>,
}

impl BBitSensor<Bluetooth> {
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
    pub async fn block_connect(mut self, device_name: &str) -> BBitResult<BBitSensor<Configure>> {
        let mut error_on_connect_max_attempts_count = 20; // error attempts

        while !self.is_connected().await {
            // try to do specified connect attempts
            match self.try_connect(device_name).await {
                Err(e @ Error::NoBleAdaptor) => {
                    tracing::error!("No bluetooth adaptors found");
                    return Err(e);
                }
                Err(e) => {
                    error_on_connect_max_attempts_count -= 1;
                    tracing::warn!("Could not connect to '{device_name}' on attempt = '{error_on_connect_max_attempts_count}', error: {}", e);
                    if error_on_connect_max_attempts_count <= 0 {
                        tracing::error!("Stopped connecting attempts after limit !");
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Ok(_) => {
                    tracing::debug!("BLE '{device_name}' is connected...");
                }
            }
        }

        let new_self: BBitSensor<Configure> = BBitSensor {
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
    /// use lib::bbit::device::BBitSensor;
    ///
    /// let mut bbit = BBitSensor::new().await.unwrap()
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
    ) -> BBitResult<BBitSensor<Configure>>
    where
        F: FnMut(BBitResult<()>) -> BBitResult<()>,
    {
        while !self.is_connected().await {
            if let Err(e) = f(self.try_connect(device_id).await) {
                return Err(e);
            }
        }
        let new_self: BBitSensor<Configure> = BBitSensor {
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
        tracing::debug!("trying to connect to '{device_name}'...");
        let adapters = self
            .ble_manager
            .adapters()
            .await
            .map_err(|_| Error::NoBleAdaptor)?;
        let Some(central) = adapters.first() else {
            tracing::error!("No ble adaptor found");
            return Err(Error::NoBleAdaptor);
        };

        tracing::debug!("Start scanning for 2 sec...");
        let mut scan_filter = ScanFilter::default();
        scan_filter.services.push(NSS2_SERVICE_UUID);
        central.start_scan(scan_filter).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        for p in central.peripherals().await? {
            if p.properties()
                .await?
                .unwrap()
                .local_name
                .iter()
                .any(|name| name.starts_with(device_name))
            {
                self.ble_device = Some(p);
                break;
            }
        }

        let Some(device) = &self.ble_device else {
            tracing::warn!("Device '{device_name}' is not found !");
            return Err(Error::NoDevice);
        };
        tracing::debug!("BLE '{device_name}' is found, try to connect...");

        device.connect().await?;
        tracing::debug!("Try to discover...");
        device.discover_services().await?;

        let controller = ControlPoint::new(device).await?;
        self.control_point = Some(controller);

        Ok(())
    }
}

/// Assign configurable parameters for BBit device
impl BBitSensor<Configure> {
    /// Add a data type to listen to
    #[instrument(skip(self))]
    pub fn listen(mut self, event_type: EventType) -> Self {
        if self.subscribed_data_event_types.contains(&event_type) {
            return self;
        }
        tracing::info!("'{event_type:?}' added to subscribed_data_event_types field");
        match event_type {
            EventType::State => {
                if !self.level.device_status {
                    self.level.device_status = true;
                }
            }
            EventType::Resistance | EventType::Eeg => {
                if !self.level.eeg_rate {
                    self.level.eeg_rate = true;
                }
            }
        }

        self.subscribed_data_event_types.push(event_type);
        self
    }

    /// Produce the sensor ready for build
    #[instrument(skip(self))]
    pub async fn build(self) -> BBitResult<BBitSensor<EventLoop>> {
        if self.level.eeg_rate {
            tracing::debug!("Subs to Resist");
            self.subscribe(EventType::Resistance.into()).await?;
        }
        if self.level.device_status {
            tracing::debug!("Subs to Dev Status");
            self.subscribe(EventType::State.into()).await?;
        }
        tracing::info!("Make sure measurements from previous connections are stopped");
        self.stop_measurement().await?;

        Ok(BBitSensor {
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            subscribed_data_event_types: self.subscribed_data_event_types,
            level: EventLoop,
            device_info: self.device_info,
        })
    }
}

impl BBitSensor<EventLoop> {
    /// Start the event loop
    #[instrument(skip_all)]
    pub async fn event_loop<H: EventHandler + Sync + Send + 'static>(
        self,
        handler: H,
    ) -> BleHandle {
        tracing::info!("starting measurements");

        // look for subscribed events
        for event_type in &self.subscribed_data_event_types {
            use EventType::*;
            if let State = event_type {
                let _ = self.subscribe_device_status_change().await;
            }
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
                    tracing::debug!("paused: ignoring data all data");
                    continue;
                }
                if data.uuid == NotifyUuid::DeviceStateChange.into() {
                    let result = DeviceStatusData::try_from(data.value);
                    tracing::debug!("received DeviceStatusData: {result:?}");
                    match result {
                        Ok(status_data) => {
                            let Ok(_) = bt_tx.send(BluetoothEvent::DeviceStatus(status_data)).await else { break };
                        }
                        Err(error) => {
                            tracing::debug!("Error receiving Device Status data: {error:?}");
                        }
                    }
                } else if data.uuid == NotifyUuid::EegMeasurement.into() {
                    let eeg_data = data.value;
                    let Ok(_) = bt_tx.send(BluetoothEvent::Egg(eeg_data)).await else { break };
                } else if data.uuid == NotifyUuid::ResistanceMeasurement.into() {
                    let resist_data = data.value;
                    tracing::debug!("received resist_data: {resist_data:?}");
                    let Ok(_) = bt_tx.send(BluetoothEvent::Resistance(resist_data)).await else { break };
                }
            }

            Ok::<_, Error>(())
        });

        tracing::info!("starting event task");
        let (event_tx, mut event_rx) = mpsc::channel(4);
        tokio::task::spawn(async move {
            loop {
                // either BLE messages or commands comes
                tokio::select! {
                    Some(data) = bt_rx.recv() => {
                        tracing::debug!("received bt channel message: {data:?}");
                        use BluetoothEvent::*;
                        match data {
                            DeviceStatus(status_data) => handler.device_status_update(status_data).await,
                            Egg(eeg_data) => handler.eeg_update(eeg_data).await,
                            Resistance(resist_data) => handler.resistance_update(resist_data).await,
                        }
                    }
                    Some(event) = event_rx.recv() => {
                        tracing::debug!("received event: {event:?}");
                        match event {
                            BleDeviceEvent::Stop => {
                                break;
                            },
                            BleDeviceEvent::StartSignal => {
                                let res = event_sensor.start_measurement(MeasurementType::Eeg).await;
                                tracing::debug!("Started Signal Measurement?: {res:?}");
                            },
                            BleDeviceEvent::StartResistance => {
                                let res = event_sensor.start_measurement(MeasurementType::Resistance).await;
                                tracing::debug!("Started Resists Measurement?: {res:?}");
                            },
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

impl<L: Level + Connected> BBitSensor<L> {
    #[instrument(skip(self))]
    async fn subscribe(&self, notify_stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("subscribing to '{:?}'", notify_stream);
        let device = self.ble_device.as_ref().expect("device already connected");

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == notify_stream.into())
            .ok_or(Error::CharacteristicNotFound)?;

        device.subscribe(&characteristic).await?;
        tracing::debug!("subscribed to '{:?}'", notify_stream);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn unsubscribe(&self, notify_stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("unsubscribing from '{notify_stream:?}'");
        let device = self.ble_device.as_ref().unwrap();

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == notify_stream.into())
            .ok_or(Error::CharacteristicNotFound)?;

        device.unsubscribe(&characteristic).await?;

        Ok(())
    }

    /// Fetch all characteristics of the device
    pub fn characteristics(&self) -> BTreeSet<Characteristic> {
        let device = self.ble_device.as_ref().unwrap();
        device.characteristics()
    }

    /// Read the battery level of the device
    #[instrument(skip_all)]
    pub async fn subscribe_device_status_change(&self) -> BBitResult<()> {
        tracing::info!("Subscribe device status changes, including cmd error, battery level");
        let device = self.ble_device.as_ref().unwrap();

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == NotifyStream::from(EventType::State).into())
            .ok_or(Error::CharacteristicNotFound)?;

        device.subscribe(&characteristic).await?;

        Ok(())
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

    /// Stop any type of possible measurement
    #[instrument(skip(self))]
    async fn stop_measurement(&self) -> BBitResult<()> {
        tracing::debug!("Stopping any measurement...");
        let controller = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();
        controller
            .send_control_command_enum(&device, &ControlPointCommand::StopAll)
            .await?;
        Ok(())
    }

    /// We start measurement (resistance OR eeg) by sending command for one EEG channel and collecting
    /// returned data.
    #[instrument(skip(self))]
    async fn start_measurement(&self, measure_type: MeasurementType) -> BBitResult<()> {
        tracing::debug!("Starting an '{measure_type:?}' measurement...");
        let controller = self.control_point.as_ref().unwrap();
        let device = self.ble_device.as_ref().unwrap();
        let command: ControlPointCommand = match measure_type {
            MeasurementType::Resistance => ControlPointCommand::StartResist([
                Nss2Status::ResistTransmission.into(),
                ADS1294ChannelInput::PowerDownGain3.into(),
                ADS1294ChannelInput::PowerUpGain1.into(),
                ADS1294ChannelInput::PowerUpGain1.into(),
                ADS1294ChannelInput::PowerUpGain1.into(),
                0x00,
                0x00,
                0x0,
            ]),
            MeasurementType::Eeg => ControlPointCommand::StartEegSignal([
                Nss2Status::EegTransmission.into(),
                0,
                0,
                0,
                0,
            ]),
        };
        controller
            .send_control_command_enum(&device, &command)
            .await?;
        tracing::debug!("Started an '{measure_type:?}' measurement");
        Ok(())
    }
}

/// Handle to the [`BBitSensor`] that is running an event loop
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

    /// Stop Signal or Resistance measurement
    #[instrument(skip(self))]
    pub async fn stop(self) {
        tracing::info!("stopping bbit sensor");
        let _ = self.sender.send(BleDeviceEvent::Stop).await;
    }

    /// Stop Signal or Resistance measurement
    #[instrument(skip(self))]
    pub async fn start(&self) {
        tracing::info!("starting Resistance measurement on bbit sensor...");
        let _ = self.sender.send(BleDeviceEvent::StartResistance).await;
    }

    /// Pause handling of bluetooth events. This will stop all Bluetooth
    /// events from being sent to your handler.
    #[instrument(skip_all)]
    pub fn pause(&self) {
        tracing::info!("pausing bluetooth event handling");
        let _ = self.pause.send(true);
    }

    /// Resume handling of bluetooth events. This will resume Bluetooth
    /// event handling.
    #[instrument(skip_all)]
    pub fn resume(&self) {
        tracing::info!("resuming bluetooth event handling");
        let _ = self.pause.send(false);
    }
}

/// Type of events sent to the event loop from [`BBitSensor`]
#[derive(Debug)]
enum BleDeviceEvent {
    /// Stop the Signal or Resistance measurement
    Stop,
    /// Send config command for Signal and start the event loop
    StartSignal,
    /// Start resistance measurement
    StartResistance,
}

/// Bluetooth data received from the sensor
#[derive(Debug)]
enum BluetoothEvent {
    DeviceStatus(DeviceStatusData),
    Egg(Vec<u8>),
    Resistance(Vec<u8>),
}
