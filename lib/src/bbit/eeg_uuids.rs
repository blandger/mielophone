use uuid::{uuid, Uuid};

/// Device name to search for
pub const PERIPHERAL_NAME_MATCH_FILTER: &'static str = "BrainBit";

/// GAT access service for access to several characteristics below
// const GENERIC_ACCESS_SERVICE_UUID: Uuid = uuid!("00001800-0000-1000-8000-00805F9B34FB");

/// Device name for reading (in GENERIC_ACCESS_SERVICE_UUID)
// const DEVICE_NAME_STRING_UUID: Uuid = uuid!("00002A00-0000-1000-8000-00805F9B34FB");
/// Device Appearance value reading (in GENERIC_ACCESS_SERVICE_UUID)
// const DEVICE_APPEARANCE_STRING_UUID: Uuid = uuid!("00002A01-0000-1000-8000-00805F9B34FB");

/// GAT attribute service for several device's characteristics
pub const GENERIC_ATTRIBUTE_SERVICE_UUID: Uuid = uuid!("0000180A-0000-1000-8000-00805F9B34FB");

/// Device Model number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
pub(crate) const MODEL_NUMBER_STRING_UUID: Uuid = uuid!("00002A24-0000-1000-8000-00805F9B34FB");
/// Serial number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
pub(crate) const SERIAL_NUMBER_STRING_UUID: Uuid = uuid!("00002A25-0000-1000-8000-00805F9B34FB");
/// HW revision number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
pub(crate) const HARDWARE_REVISION_STRING_UUID: Uuid =
    uuid!("00002A26-0000-1000-8000-00805F9B34FB");
/// SW revision number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
pub(crate) const FIRMWARE_REVISION_STRING_UUID: Uuid =
    uuid!("00002A27-0000-1000-8000-00805F9B34FB");

/// Main GAT service to receive data and transmit commands from/to device
pub const NSS2_SERVICE_UUID: Uuid = uuid!("6E400001-B534-F393-68A9-E50E24DCCA9E");

/// Device state for receiving (in NSS2_SERVICE_UUID)
pub const DEVICE_STATE_NOTIFY_CHARACTERISTIC_UUID: Uuid =
    uuid!("6E400002-B534-F393-68A9-E50E24DCCA9E");
/// EEG data for receiving (in NSS2_SERVICE_UUID)
pub const EEG_DATA_NOTIFY_CHARACTERISTIC_UUID: Uuid = uuid!("6E400004-B534-F393-68A9-E50E24DCCA9E");
/// Commands data for transmitting (in NSS2_SERVICE_UUID)
pub const WRITE_COMMAN_UUID: Uuid = uuid!("6E400003-B534-F393-68A9-E50E24DCCA9E");

/// Which UUID to send BLE messages to.
pub enum NotifyUuid {
    /// Notify about device status change, including command errors, battery level change
    DeviceStateChange,
    /// Notify about EEG signal measurement/updates
    EegOrResistanceMeasurementChange,
}

/// A list of stream types that can be subscribed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyStream {
    /// Device status updates, includes command errors, battery level
    DeviceState,
    /// Receive eeg data updates.
    EegOrResistanceMeasurement,
}

impl From<NotifyStream> for Uuid {
    fn from(item: NotifyStream) -> Self {
        NotifyUuid::from(item).into()
    }
}

impl From<NotifyStream> for NotifyUuid {
    fn from(item: NotifyStream) -> Self {
        match item {
            NotifyStream::DeviceState => Self::DeviceStateChange,
            NotifyStream::EegOrResistanceMeasurement => Self::EegOrResistanceMeasurementChange,
        }
    }
}

/// Types the [`BleSensor`] can listen for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// EEG data or Electrode Resistance data
    EegOrResistance,
    /// Internal device status with additional data
    State,
}

impl EventType {
    /// Convert [`EventType`] into its [`Uuid[]`]
    pub fn to_uuid(&self) -> Uuid {
        NotifyUuid::from(*self).into()
    }
}

impl From<EventType> for NotifyStream {
    fn from(value: EventType) -> Self {
        match value {
            EventType::State => Self::DeviceState,
            EventType::EegOrResistance => Self::EegOrResistanceMeasurement,
        }
    }
}

impl From<EventType> for NotifyUuid {
    fn from(value: EventType) -> Self {
        match value {
            EventType::State => Self::DeviceStateChange,
            EventType::EegOrResistance => Self::EegOrResistanceMeasurementChange,
        }
    }
}

impl From<NotifyUuid> for Uuid {
    fn from(item: NotifyUuid) -> Self {
        match item {
            NotifyUuid::DeviceStateChange => DEVICE_STATE_NOTIFY_CHARACTERISTIC_UUID,
            NotifyUuid::EegOrResistanceMeasurementChange => EEG_DATA_NOTIFY_CHARACTERISTIC_UUID,
        }
    }
}

pub enum StringUuid {
    ModelNumber,
    HardwareRevision,
    FirmwareRevision,
    SerialNumber,
}

impl From<StringUuid> for Uuid {
    fn from(item: StringUuid) -> Self {
        match item {
            StringUuid::ModelNumber => MODEL_NUMBER_STRING_UUID,
            StringUuid::HardwareRevision => HARDWARE_REVISION_STRING_UUID,
            StringUuid::FirmwareRevision => FIRMWARE_REVISION_STRING_UUID,
            StringUuid::SerialNumber => SERIAL_NUMBER_STRING_UUID,
        }
    }
}
