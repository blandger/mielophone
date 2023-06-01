use uuid::{uuid, Uuid};

/// Device name to search for
pub const PERIPHERAL_NAME_MATCH_FILTER: &'static str = "BrainBit";

/// GAT access service for access to several characteristics below
pub const GENERIC_ACCESS_SERVICE_UUID: Uuid = uuid!("00001800-0000-1000-8000-00805F9B34FB");

/// Device name for reading (in GENERIC_ACCESS_SERVICE_UUID)
const DEVICE_NAME_STRING_UUID: Uuid = uuid!("00002A00-0000-1000-8000-00805F9B34FB");
/// Device Appearance value reading (in GENERIC_ACCESS_SERVICE_UUID)
const DEVICE_APPEARANCE_STRING_UUID: Uuid = uuid!("00002A01-0000-1000-8000-00805F9B34FB");

/// GAT attribute service for several device's characteristics
pub const GENERIC_ATTRIBUTE_SERVICE_UUID: Uuid = uuid!("00001801-0000-1000-8000-00805F9B34FB");

/// Device Model number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
const MODEL_NUMBER_STRING_UUID: Uuid = uuid!("00002A24-0000-1000-8000-00805F9B34FB");
/// Serial number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
const SERIAL_NUMBER_STRING_UUID: Uuid = uuid!("00002A25-0000-1000-8000-00805F9B34FB");
/// HW revision number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
const HARDWARE_REVISION_STRING_UUID: Uuid = uuid!("00002A26-0000-1000-8000-00805F9B34FB");
/// SW revision number, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
const FIRMWARE_REVISION_STRING_UUID: Uuid = uuid!("00002A27-0000-1000-8000-00805F9B34FB");
/// Battery level, 2 bytes (in GENERIC_ATTRIBUTE_SERVICE_UUID)
pub const BATTERY_LEVEL_CHARACTERISTIC_UUID: Uuid = uuid!("00002A05-0000-1000-8000-00805F9B34FB");

/// Main GAT service to receive data and transmit commands from/to device
pub const NSS2_SERVICE_UUID: Uuid = uuid!("6E400001-B534-F393-68A9-E50E24DCCA9E");

/// EEG data for receiving (in NSS2_SERVICE_UUID)
pub const EEG_DATA_NOTIFY_CHARACTERISTIC_UUID: Uuid = uuid!("6E400004-B534-F393-68A9-E50E24DCCA9E");
/// Commands data for transmitting (in NSS2_SERVICE_UUID)
pub const WRITE_COMMAN_UUID: Uuid = uuid!("6E400003-B534-F393-68A9-E50E24DCCA9E");

/// Which UUID to send BLE messages to.
pub enum NotifyUuid {
    BatteryLevel,
    EegMeasurement,
    ResistanceMeasurement,
}

/// A list of stream types that can be subscribed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyStream {
    /// Receive battery updates.
    Battery,
    /// Receive eeg data updates.
    EegMeasurement,
    /// Receive eeg data updates.
    ResistanceMeasurement,
}

impl From<NotifyStream> for Uuid {
    fn from(item: NotifyStream) -> Self {
        NotifyUuid::from(item).into()
    }
}

impl From<NotifyStream> for NotifyUuid {
    fn from(item: NotifyStream) -> Self {
        match item {
            NotifyStream::Battery => Self::BatteryLevel,
            NotifyStream::EegMeasurement => Self::EegMeasurement,
            NotifyStream::ResistanceMeasurement => Self::ResistanceMeasurement,
        }
    }
}

/// Types the [`BleSensor`] can listen for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// EEG data
    Eeg,
    /// Battery
    Battery,
    // electrode resistance
    Resistance,
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
            EventType::Battery => Self::Battery,
            EventType::Eeg => Self::EegMeasurement,
            EventType::Resistance => Self::ResistanceMeasurement,
        }
    }
}

impl From<EventType> for NotifyUuid {
    fn from(value: EventType) -> Self {
        match value {
            EventType::Battery => Self::BatteryLevel,
            EventType::Eeg => Self::EegMeasurement,
            EventType::Resistance => Self::ResistanceMeasurement,
        }
    }
}

impl From<NotifyUuid> for Uuid {
    fn from(item: NotifyUuid) -> Self {
        match item {
            NotifyUuid::BatteryLevel => BATTERY_LEVEL_CHARACTERISTIC_UUID,
            NotifyUuid::EegMeasurement => EEG_DATA_NOTIFY_CHARACTERISTIC_UUID,
            NotifyUuid::ResistanceMeasurement => EEG_DATA_NOTIFY_CHARACTERISTIC_UUID,
        }
    }
}

pub enum StringUuid {
    DeviceName,
    DeviceAppearance,
    ModelNumber,
    HardwareRevision,
    FirmwareRevision,
    SerialNumber,
}

impl From<StringUuid> for Uuid {
    fn from(item: StringUuid) -> Self {
        match item {
            StringUuid::DeviceName => DEVICE_NAME_STRING_UUID,
            StringUuid::DeviceAppearance => DEVICE_APPEARANCE_STRING_UUID,
            StringUuid::ModelNumber => MODEL_NUMBER_STRING_UUID,
            StringUuid::HardwareRevision => HARDWARE_REVISION_STRING_UUID,
            StringUuid::FirmwareRevision => FIRMWARE_REVISION_STRING_UUID,
            StringUuid::SerialNumber => SERIAL_NUMBER_STRING_UUID,
        }
    }
}
