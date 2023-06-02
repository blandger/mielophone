use crate::bbit::device::BBitResult;
use crate::Error;

/// A common device's state type
// ??? Probably it's returned from UUID = 6E400002-B534-F393-68A9-E50E24DCCA9E (READ / NOTIFY)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonDeviceState {
    /// device is not initialized
    Invalid,
    /// device is turned on and sends advertisements
    Advertising,
    /// sensor is connected
    Connected,
    /// accum has discharged, turning off device
    PowerDown,
    /// DFU loader mode
    Dfu,
}

/// Structure to contain HR data and RR interval.
#[derive(Debug, Clone)]
pub struct EggData {
    data: Vec<u16>,
}

/// Contains common information about device like:
/// model, serial number, HW, SW revision
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    model_number: String,
    serial_number: String,
    hardware_revision: String,
    firmware_revision: String,
}

impl DeviceInfo {
    pub fn new(
        model_number: String,
        serial_number: String,
        hardware_revision: String,
        firmware_revision: String,
    ) -> Self {
        Self {
            model_number,
            serial_number,
            hardware_revision,
            firmware_revision,
        }
    }
}

/// Common Device status including NSS2 service state, Commands execution state, battery level, Firmware version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStatus {
    /// NNS service state
    status_nss2: Nss2Status,
    // Error code. It's reset when new command is received
    cmd_error: CommandResultError,
    /// Battery level in percents (%) is stored by lower seven bits, 8-th bit keeps 'charging flag'
    battery_level: u8,
    /// Firmware version
    firmware_version: u8,
}

impl DeviceStatus {
    /// Create new instance of [`DeviceStatus`] from Vec<u8>.
    pub fn new(value: Vec<u8>) -> BBitResult<Self> {
        if value.is_empty() || value.len() != 4 {
            eprintln!("Invalid DeviceStatus result vec length {:?}", value);
            return Err(Error::InvalidData("device status Vec length".to_string()));
        }
        let status_nss2 = Nss2Status::try_from(value[0])
            .map_err(|_| Error::InvalidData("NSS2 status byte value".to_string()))?;
        let cmd_error = CommandResultError::try_from(value[1])
            .map_err(|_| Error::InvalidData("NSS2 CMD byte value".to_string()))?;
        let battery_level = value[2];
        let firmware_version = value[3];

        Ok(Self {
            status_nss2,
            cmd_error,
            battery_level,
            firmware_version,
        })
    }
}

/// A main GATT NSS2 service state/mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nss2Status {
    /// initial = invalid state
    Initial,
    /// service is initialized, stopped, but it is ready to start work
    Stopped,
    /// sensor is connected, started signal measurement, service sends eeg data to host
    EegTransmission,
    /// sensor is connected, started resistance measurement, service sends resist data to host
    ResistTransmission,
    /// DFU loader mode
    DfuBootLoderMode,
}
impl TryFrom<u8> for Nss2Status {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0x0 => Ok(Self::Initial),
            0x1 => Ok(Self::Stopped),
            0x2 => Ok(Self::EegTransmission),
            0x3 => Ok(Self::ResistTransmission),
            0x4 => Ok(Self::DfuBootLoderMode),
            _ => Err(()),
        }
    }
}

/// A main GATT NSS2 service sending, executing command result type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResultError {
    /// No error after command
    NoError,
    /// Command had an incorrect length
    ErrorLength,
    /// Error on changing device mode, changing working mode is not possible
    ErrorSwitchMode,
}
impl TryFrom<u8> for CommandResultError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0x0 => Ok(Self::NoError),
            0x1 => Ok(Self::ErrorLength),
            0x2 => Ok(Self::ErrorSwitchMode),
            _ => Err(()),
        }
    }
}
