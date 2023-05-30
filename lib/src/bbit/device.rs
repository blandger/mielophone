use crate::bbit::control::ControlPoint;
use crate::bbit::eeg_uuids::{EventType, NotifyStream, PERIPHERAL_NAME_MATCH_FILTER};
use crate::bbit::sealed::{Bluetooth, Configure, Connected, EventLoop, Level};
use crate::{Error, EventHandler};
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::instrument;

pub type BBitResult<T> = Result<T, Error>;

/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct EegData {
    data: i16,
    index: u8,
}
/// Structure to contain EEG data and interval.
#[derive(Debug, Clone)]
pub struct CommandData {
    data: i16,
    cmd_type: CommandType,
}

/// The core sensor manager
pub struct BleSensor<L: Level> {
    /// BLE connection manager
    ble_manager: Manager,
    /// Connected and controlled device
    ble_device: Option<Peripheral>,
    /// BLE event types subscribed and processed
    subscribed_data_event_types: Vec<EventType>,
    /// Device manage and send commands
    control_point: Option<ControlPoint>,
    level: L,
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
        let new_self: BleSensor<Configure> = BleSensor {
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            subscribed_data_event_types: self.subscribed_data_event_types,
            level: Configure::default(),
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
    ///     .map_connect("7B45F72B", |r| {
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

    /// Try to connect to a device. Implements the v1 [`crate::BleSensor::connect`] function
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

        device.connect().await?;
        device.discover_services().await?;

        let controller = ControlPoint::new(device).await?;
        self.control_point = Some(controller);

        Ok(())
    }
}

impl BleSensor<Configure> {
    /// Add a data type to listen to
    #[instrument(skip(self))]
    pub fn listen(mut self, ty: EventType) -> Self {
        if self.subscribed_data_event_types.contains(&ty) {
            return self;
        }
        tracing::info!("'{ty:?}' added to subscribed_data_event_types field");
        match ty {
            EventType::Eeg => {
                if !self.level.eeg_rate {
                    self.level.eeg_rate = true;
                }
            }
            EventType::Battery => {
                if !self.level.battery {
                    self.level.battery = true;
                }
            }
        }

        self.subscribed_data_event_types.push(ty);
        self
    }

    /// Produce the sensor ready for build
    #[instrument(skip(self))]
    pub async fn build(self) -> BBitResult<BleSensor<EventLoop>> {
        if self.level.eeg_rate {
            self.subscribe(EventType::Eeg.into()).await?;
        }
        if self.level.battery {
            self.subscribe(EventType::Battery.into()).await?;
        }

        Ok(BleSensor {
            level: EventLoop,
            ble_manager: self.ble_manager,
            ble_device: self.ble_device,
            control_point: self.control_point,
            subscribed_data_event_types: self.subscribed_data_event_types,
        })
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
        todo!()
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
}

/// Handle to the [`BleSensor`] that is running an event loop
#[derive(Clone)]
pub struct BleHandle {
    sender: mpsc::Sender<Event>,
    pause: Arc<watch::Sender<bool>>,
}

impl BleHandle {
    fn new(sender: mpsc::Sender<Event>, pause: watch::Sender<bool>) -> Self {
        Self {
            sender,
            pause: Arc::new(pause),
        }
    }
}

/// Type of events to send to the event loop of [`BleSensor`]
#[derive(Debug)]
enum Event {
    /// Stop the event loop
    Stop,
}

#[derive(Clone, Debug)]
pub enum CommandType {
    CommandStartSignal,
    CommandStopSignal,
    CommandStartResist,
    CommandStopResist,
    CommandStartMEMS,
    CommandStopMEMS,
    CommandStartRespiration,
    CommandStopRespiration,
    CommandStartStimulation,
    CommandStopStimulation,
    CommandEnableMotionAssistant,
    CommandDisableMotionAssistant,
    CommandFindMe,
}

#[derive(Clone, Debug)]
pub struct CommandArray<'a> {
    cmd_array: &'a [CommandData],
    cmd_array_size: usize,
}
