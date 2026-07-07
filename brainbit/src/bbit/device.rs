use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use futures::stream::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::bbit::control_point::{ControlCommandType, ControlPoint, ControlPointCommand};
use crate::bbit::device_mode::{ADS1294ChannelInput, ChannelType, DeviceMode};
use crate::bbit::responses::DeviceStatusData;
use crate::bbit::errors::BBitResult;
use crate::bbit::traits::EventHandler;
use crate::bbit::uuids::{EventType, NotifyStream, NotifyUuid, FIRMWARE_REVISION_STRING_UUID, HARDWARE_REVISION_STRING_UUID, MODEL_NUMBER_STRING_UUID, NSS2_SERVICE_UUID, PERIPHERAL_NAME_MATCH_FILTER, SERIAL_NUMBER_STRING_UUID};
use crate::{find_characteristic, Error};
use crate::bbit::device_info::DeviceInfo;
use tokio::time::{self, Duration};

/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct CommandData {
    data: i16,
    cmd_type: ControlPointCommand,
}

/// The core sensor manager
pub struct BBitSensor {
    /// The device id or name in the device (e.g, "BrainBit")
    device_id: String,
    /// BLE connection manager
    ble_manager: Manager,
    /// Connected and controlled device
    ble_device: Option<Peripheral>,
    /// Handler for event callbacks
    event_handler: Option<Arc<dyn EventHandler>>,
    /// BLE event type currently subscribed and processed
    pub data_type: Vec<EventType>,
    /// Device manage and send commands
    pub control_point: Option<ControlPoint>,
    /// Common device information like model, serial numbers, HW, SW revisions
    pub device_info: OnceLock<DeviceInfo>,
}

impl BBitSensor {
    /// Construct a BleSensor
    ///
    /// Returns a [`Error::BleError`] if the Bluetooth manager could not be created
    pub async fn new(device_id: String) -> BBitResult<Self> {
        if device_id.len() != 8 {
            return Err(Error::InvalidData("BrainBit device name is missing".to_string()));
        }
        let ble_manager = Manager::new().await.map_err(Error::BleError)?;
        Ok(Self {
            device_id,
            ble_manager,
            ble_device: None,
            event_handler: None,
            data_type: vec![],
            control_point: None,
            device_info: OnceLock::new(),
        })
    }

    async fn find_device(&self, central: &Adapter) -> Option<Peripheral> {
        for p in central.peripherals().await.unwrap() {
            if p.properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| name.starts_with(PERIPHERAL_NAME_MATCH_FILTER) && name.ends_with(&self.device_id))
            {
                return Some(p);
            }
        }
        None
    }

    /// Tries find and connect to the device instance by using id associated with this device.
    ///
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> BBitResult<()> {
        debug!("trying to connect to '{:?}'...", &self.device_id);
        let adapters_result = self.ble_manager.adapters().await.map_err(Error::BleError);

        if let Ok(adapters) = adapters_result {
            if adapters.is_empty() {
                tracing::error!("No ble adaptor found");
                return Err(Error::NoBleAdaptor);
            }

            let central = adapters.into_iter().next().expect("No adaptor found, crash");
            debug!("Start scanning for 2 sec...");
            let mut scan_filter = ScanFilter::default();
            scan_filter.services.push(NSS2_SERVICE_UUID);
            central
                .start_scan(scan_filter)
                .await
                .map_err(Error::BleError)?;
            time::sleep(Duration::from_secs(2)).await;

            self.ble_device = self.find_device(&central).await;

            if let Some(device) = &self.ble_device {
                debug!("BLE '{}' is found, try to connect...", &self.device_id);
                device.connect().await.map_err(Error::BleError)?;
                debug!("Try to discover services...");
                device.discover_services().await.map_err(Error::BleError)?;

                let controller = ControlPoint::new(device).await?;
                self.control_point = Some(controller);
                return Ok(());
            }
            return Err(Error::NoDevice);
        }
        tracing::error!("No ble adaptor found, end...");
        Err(Error::NoBleAdaptor)
    }

    /// Subscribes to a notify event on the device. These events will be sent via the [`EventHandler`].
    ///
    /// # Errors
    ///
    /// Will return:
    /// - [`Error::NotConnected`] if the device is not currently connected
    /// - [`Error::CharacteristicNotFound`] if a given notify type is not found on the device
    /// - [`Error::BleError`] if there is an error subscribing to the event
    pub async fn subscribe(&self, stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("subscribing to stream of '{:#?}' type...", stream);
        let device = self.device().await?;

        if let Ok(true) = device.is_connected().await {
            let characteristic = find_characteristic(device, stream.into()).await?;
            return device
                .subscribe(&characteristic)
                .await
                .map_err(Error::BleError);
        }
        Err(Error::NotConnected)
    }

    /// Unsubscribes to a notify event on your device.
    ///
    /// Will return:
    /// - [`Error::NotConnected`] if the device isn't connected
    /// - [`Error::CharacteristicNotFound`] if the specified notify type isn't found on the device
    /// - [`Error::BleError`] if there is an error subscribing to the event from within BLE
    pub async fn unsubscribe(&self, stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("unsubscribing from stream of '{stream:?} type...'");
        let device = self.device().await?;

        if let Ok(true) = device.is_connected().await {
            let characteristic = find_characteristic(device, stream.into()).await?;

            return device
                .unsubscribe(&characteristic)
                .await
                .map_err(Error::BleError);
        }
        Err(Error::NotConnected)
    }

    pub async fn is_connected(&self) -> bool {
        if let Some(device) = &self.ble_device {
            if let Ok(v) = device.is_connected().await {
                return v;
            }
        }
        false
    }

    async fn controller(&self) -> BBitResult<&ControlPoint> {
        if let Some(controller) = &self.control_point {
            return Ok(controller);
        }
        Err(Error::NoControlPointAssigned)
    }

    async fn device(&self) -> BBitResult<&Peripheral> {
        if let Some(device) = &self.ble_device {
            return Ok(device);
        }
        Err(Error::NoDevice)
    }

    async fn read(&self, uuid: Uuid) -> BBitResult<Vec<u8>> {
        let device = self.device().await?;
        if let Ok(char) = find_characteristic(device, uuid).await {
            return device.read(&char).await.map_err(Error::BleError);
        }
        Err(Error::CharacteristicNotFound)
    }

    async fn read_string(&self, uuid: Uuid) -> BBitResult<String> {
        let data = self.read(uuid).await?;
        let string = String::from_utf8_lossy(&data).into_owned();
        Ok(string.trim_matches(char::from(0)).to_string())
    }

    /// Sets an event handler with multiple methods for each possible event.
    pub fn event_handler<H: EventHandler + 'static>(&mut self, event_handler: H) {
        self.event_handler = Some(Arc::new(event_handler));
    }

    /// Send command as enum to [`ControlPoint`].
    #[instrument(skip(self))]
    pub async fn send_command(&self, command: ControlPointCommand) -> BBitResult<()> {
        let controller = self.controller().await?;
        let device = self.device().await?;
        controller
            .send_control_command_enum(device, command)
            .await?;
        Ok(())
    }

    /// Stop any type of possible measurement
    #[instrument(skip(self))]
    async fn stop_measurement(&self) -> BBitResult<()> {
        debug!("Stopping any measurement...");
        let controller = self.controller().await?;
        let device = self.device().await?;
        let command = ControlPointCommand::new(ControlCommandType::StopAll, None);
        controller
            .send_control_command_enum(&device, command)
            .await?;
        Ok(())
    }

    /// We start measurement (resistance OR eeg) by sending command for one EEG channel and collecting
    /// returned data.
    #[instrument(skip(self))]
    async fn start_measurement(&self, measure_type: DeviceMode) -> BBitResult<()> {
        debug!("Starting an '{measure_type:?}' measurement...");
        let controller = self.controller().await?;
        let device = self.device().await?;
        let command: ControlPointCommand = match measure_type {
            DeviceMode::Resistance(ChannelType::O1) => {
                let cmd_data = [
                    ADS1294ChannelInput::PowerDownGain3.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    0b00000001,
                    0x01,
                    0x0,
                ];
                ControlPointCommand::new(ControlCommandType::StartResist, Some(Vec::from(cmd_data)))
            }
            DeviceMode::Resistance(ChannelType::T3) => {
                let cmd_data = [
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerDownGain3.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    0b00000010,
                    0x03,
                    0x0,
                ];
                ControlPointCommand::new(ControlCommandType::StartResist, Some(Vec::from(cmd_data)))
            }
            DeviceMode::Resistance(ChannelType::T4) => {
                let cmd_data = [
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerDownGain3.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    0b00000100,
                    0x05,
                    0x0,
                ];
                ControlPointCommand::new(ControlCommandType::StartResist, Some(Vec::from(cmd_data)))
            }
            DeviceMode::Resistance(ChannelType::O2) => {
                let cmd_data = [
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerUpGain1.into(),
                    ADS1294ChannelInput::PowerDownGain3.into(),
                    0b0001000,
                    0b0001000,
                    0x0,
                ];
                ControlPointCommand::new(ControlCommandType::StartResist, Some(Vec::from(cmd_data)))
            }
            DeviceMode::Eeg => {
                let cmd_data = [ADS1294ChannelInput::PowerDownGain6.into(), 0x00, 0x00, 0x0];
                ControlPointCommand::new(
                    ControlCommandType::StartEegSignal,
                    Some(Vec::from(cmd_data)),
                )
            }
        };
        controller
            .send_control_command_enum(&device, command)
            .await?;
        debug!("DONE. Started an '{measure_type:?}' measurement");
        Ok(())
    }

    /// Read the battery level of the device
    #[instrument(skip_all)]
    pub async fn subscribe_device_status_change(&self) -> BBitResult<()> {
        tracing::info!("Subscribe device status changes, including cmd error, battery level");
        let device = self.device().await?;

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == Uuid::from(NotifyStream::from(EventType::State)))
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
        debug!("device info: '{:?}'", self.device_info.get());
        Ok(self.device_info.get().expect("DeviceInfo is not initialized?").clone())
    }

    /// Fetch all characteristics of the device
    pub fn characteristics(&self) -> BBitResult<BTreeSet<Characteristic>> {
        let device = self.ble_device.as_ref().unwrap();
        Ok(device.characteristics())
    }
}

/// Assign configurable parameters for BBit device
/*impl BBitSensor<Configure> {
    /// Add a data type to listen to
    #[instrument(skip(self))]
    pub fn listen(mut self, event_type: EventType) -> Self {
        if self.data_type.contains(&event_type) {
            return self;
        }
        tracing::info!("'{event_type:?}' added to subscribed_data_event_types field");
        match event_type {
            EventType::State => {
                if !self.level.device_status {
                    self.level.device_status = true;
                }
            }
            EventType::EegOrResistance => {
                if !self.level.eeg_rate {
                    self.level.eeg_rate = true;
                }
            }
        }

        self.data_type.push(event_type);
        self
    }

    /// Produce the sensor ready for build
    #[instrument(skip(self))]
    pub async fn build(self) -> BBitResult<BBitSensor<EventLoop>> {
        tracing::info!(
            "Building sensor... Make sure measurements from previous connections are stopped."
        );
        self.stop_measurement().await?;
        if self.level.eeg_rate {
            debug!("Will subscribe to Resist event...");
            self.subscribe(EventType::EegOrResistance.into()).await?;
        }
        if self.level.device_status {
            debug!("Will subscribe to DeviceStatus event...");
            self.subscribe_device_status_change().await?;
        }

        Ok(BBitSensor {
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            data_type: self.data_type,
            device_info: self.device_info,
        })
    }
}*/

/*impl BBitSensor<EventLoop> {
    /// Start the event loop
    #[instrument(skip_all)]
    pub async fn event_loop<H>(
        self,
        mut handler: H,
    ) -> BleHandle where H: EventHandler + Sync + Send + 'static, {
        tracing::info!(
            "starting event_loop... we have event list to subscribe to: {:?}",
            &self.data_type
        );

        // look for subscribed events
        for event_type in &self.data_type {
            use EventType::*;
            if let State = event_type {
                let _ = self.subscribe_device_status_change().await;
            }
            if let EegOrResistance = event_type {
                let _ = self
                    .subscribe(NotifyStream::EegOrResistanceMeasurement)
                    .await;
            }
        }
        let bt_sensor = Arc::new(self);
        let event_sensor = Arc::clone(&bt_sensor);

        tracing::info!("loop - starting bluetooth task");
        let (bt_tx, mut bt_rx) = mpsc::channel(128);
        let (pause_tx, pause_rx) = watch::channel(false);

        tokio::task::spawn(async move {
            let device = bt_sensor.ble_device.as_ref().unwrap();
            let mut notification_stream = device.notifications().await?;

            while let Some(data) = notification_stream.next().await {
                tracing::trace!("loop - received Bluetooth data: {:02X?}", data);
                if *pause_rx.borrow() {
                    debug!("loop paused: ignoring data all data");
                    continue;
                }
                if data.uuid == Uuid::from(NotifyUuid::DeviceStateChange) {
                    let result = DeviceStatusData::try_from(data.value);
                    tracing::trace!("loop - received DeviceStatusData: {result:?}");
                    match result {
                        Ok(status_data) => {
                            let Ok(_) = bt_tx.send(BluetoothEvent::DeviceStatus(status_data)).await
                            else {
                                break;
                            };
                        }
                        Err(error) => {
                            debug!("Error receiving Device Status data: {error:?}");
                        }
                    }
                } else if data.uuid == Uuid::from(NotifyUuid::EegOrResistanceMeasurementChange) {
                    let eeg_or_resist_data = data.value;
                    tracing::trace!(
                        "loop - received eeg-resist_data: {:02X?}",
                        eeg_or_resist_data
                    );
                    let Ok(_) = bt_tx
                        .send(BluetoothEvent::EggOrResistanceData(eeg_or_resist_data))
                        .await
                    else {
                        break;
                    };
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
                        debug!("received bt channel message: {:02X?}", data);
                        use BluetoothEvent::*;
                        match data {
                            DeviceStatus(status_data) => handler.device_status_update(status_data).await,
                            EggOrResistanceData(eeg_data) => handler.eeg_update(eeg_data).await,
                        }
                    }
                    Some(event) = event_rx.recv() => {
                        debug!("received event: {:02x?}", event);
                        match event {
                            BleDeviceEvent::Stop => {
                                let res = event_sensor.stop_measurement().await;
                                debug!("Stop Signal?: {res:?}");
                                break;
                            },
                            BleDeviceEvent::StartSignal{ret} => {
                                let res = event_sensor.start_measurement(DeviceMode::Eeg).await;
                                debug!("Started Signal Measurement?: {res:?}");
                                let _ = ret.send(res);
                            },
                            BleDeviceEvent::StartResistance{channel_type, ret} => {
                                let res = event_sensor.start_measurement(
                                    DeviceMode::Resistance(channel_type)).await;
                                debug!("Started Resists Measurement?: {res:?}");
                                let _ = ret.send(res);
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
}*/

/*impl<L: Level + Connected> BBitSensor<L> {
    #[instrument(skip(self))]
    async fn subscribe(&self, notify_stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("subscribing to stream of '{:#?}' type...", notify_stream);
        let device = self.ble_device.as_ref().expect("device already connected");

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == Uuid::from(notify_stream))
            .ok_or(Error::CharacteristicNotFound)?;

        device.subscribe(&characteristic).await?;
        debug!("DONE, subscribed to stream of '{:?}' type", notify_stream);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn unsubscribe(&self, notify_stream: NotifyStream) -> BBitResult<()> {
        tracing::info!("unsubscribing from stream of '{notify_stream:?} type...'");
        let device = self.ble_device.as_ref().unwrap();

        let characteristics = device.characteristics();
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == Uuid::from(notify_stream))
            .ok_or(Error::CharacteristicNotFound)?;

        device.unsubscribe(&characteristic).await?;
        debug!(
            "DONE, unsubscribed from stream of '{:?}' type",
            notify_stream
        );

        Ok(())
    }

    /// Fetch all characteristics of the device
    pub fn characteristics(&self) -> BTreeSet<Characteristic> {
        let device = self.ble_device.as_ref().unwrap();
        device.characteristics()
    }


    async fn read(&self, uuid: Uuid) -> BBitResult<Vec<u8>> {
        let device = self.ble_device.as_ref().unwrap();
        // let device = self.device().await?;
        if let Ok(char) = find_characteristic(device, uuid).await {
            return device.read(&char).await.map_err(Error::BleError);
        }
        Err(Error::CharacteristicNotFound)
    }

}*/

/// Handle to the [`BBitSensor`] that is running an event loop
#[derive(Clone)]
pub struct BleHandle {
    sender: mpsc::Sender<BleDeviceEventType>,
    pause: Arc<watch::Sender<bool>>,
}

impl BleHandle {
    fn new(sender: mpsc::Sender<BleDeviceEventType>, pause: watch::Sender<bool>) -> Self {
        Self {
            sender,
            pause: Arc::new(pause),
        }
    }

    /// Stop Signal or Resistance measurement
    #[instrument(skip(self))]
    pub async fn stop(self) {
        tracing::info!("stopping bbit sensor");
        let _ = self.sender.send(BleDeviceEventType::Stop).await;
    }

    /// Start Signal or Resistance measurement
    #[instrument(skip(self))]
    pub async fn start(&self) -> Option<BBitResult<()>> {
        tracing::info!("starting Resistance measurement on bbit sensor...");
        let (ret, rx) = oneshot::channel();
        let channel_type = ChannelType::O1;
        let _ = self
            .sender
            .send(BleDeviceEventType::StartResistance { channel_type, ret })
            .await;

        rx.await.ok()
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

/// Type of events sent to the event loop of [`BBitSensor`]
#[derive(Debug)]
enum BleDeviceEventType {
    /// Stop the Signal or Resistance measurement
    Stop,
    /// Send config command for Signal and start the event loop
    StartSignal {
        /// channel to receive return value
        ret: oneshot::Sender<BBitResult<()>>,
    },
    /// Start resistance measurement
    StartResistance {
        /// Channel number/type
        channel_type: ChannelType,
        /// channel to receive return value
        ret: oneshot::Sender<BBitResult<()>>,
    },
}

/// Bluetooth data received from the sensor
#[derive(Debug)]
enum BluetoothEvent {
    DeviceStatus(DeviceStatusData),
    EggOrResistanceData(Vec<u8>),
}
