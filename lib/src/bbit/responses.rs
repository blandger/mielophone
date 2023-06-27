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
// #[derive(Debug, Clone)]
// pub struct EggData {
//     data: Vec<u16>,
// }

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

/// Common Device status data including NSS2 service state, Commands execution state, battery level, Firmware version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStatusData {
    /// NNS service state
    pub status_nss2: Nss2Status,
    // Error code. It's reset when new command is received
    pub cmd_error: CommandResultError,
    /// Battery level in percents (%) is stored by lower seven bits, 8-th bit keeps 'charging flag'
    pub battery_level: u8,
    /// Firmware version
    pub firmware_version: u8,
}

impl TryFrom<Vec<u8>> for DeviceStatusData {
    type Error = &'static str;

    /// Create new instance of [`DeviceStatusData`] from Vec<u8>.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() != 4 {
            eprintln!("Invalid DeviceStatus result vec length {:?}", value);
            return Err("device status Vec length");
        }
        let status_nss2 = Nss2Status::try_from(value[0])?;
        // .map_err(|_| Error::InvalidData("NSS2 status byte value".to_string()))?;
        let cmd_error = CommandResultError::try_from(value[1])?;
        // .map_err(|_| Error::InvalidData("NSS2 CMD byte value".to_string()))?;
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
    Initial = 0x00,
    /// service is initialized, stopped, but it is ready to start work
    Stopped = 0x01,
    /// sensor is connected, started signal measurement, service sends eeg data to host
    EegTransmission = 0x02,
    /// sensor is connected, started resistance measurement, service sends resist data to host
    ResistTransmission = 0x03,
    /// DFU loader mode
    DfuBootLoderMode = 0x04,
}
impl TryFrom<u8> for Nss2Status {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Initial),
            0x01 => Ok(Self::Stopped),
            0x02 => Ok(Self::EegTransmission),
            0x03 => Ok(Self::ResistTransmission),
            0x04 => Ok(Self::DfuBootLoderMode),
            _ => Err("Nss2Status value is incorrect"),
        }
    }
}
impl Into<u8> for Nss2Status {
    fn into(self) -> u8 {
        self as u8
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
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::NoError),
            0x1 => Ok(Self::ErrorLength),
            0x2 => Ok(Self::ErrorSwitchMode),
            _ => Err("BBit command execution result error"),
        }
    }
}
