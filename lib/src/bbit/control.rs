use crate::bbit::device::{BBitResult};
use crate::bbit::eeg_uuids::WRITE_COMMAN_UUID;
use crate::{find_characteristic, Error};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

/// Struct that has access to command point.
#[derive(Debug, PartialEq, Eq)]
pub struct ControlPoint {
    control_point: Characteristic,
}

impl ControlPoint {
    /// Create new [`ControlPoint`].
    pub async fn new(device: &Peripheral) -> BBitResult<Self> {
        let control_point = find_characteristic(device, WRITE_COMMAN_UUID).await?;

        Ok(Self { control_point })
    }

    /// Send command to [`ControlPoint`] waiting for a response from device.
    pub async fn send_command(&self, device: &Peripheral, data: &[u8]) -> BBitResult<()> {
        self.write(device, data).await?;
        Ok(())
    }

    /// Send command to [`ControlPoint`] waiting for a response from device.
    pub async fn send_control_command_enum(
        &self,
        device: &Peripheral,
        command: &ControlPointCommand,
    ) -> BBitResult<()> {
        let command_as_bytes: &[u8] = unsafe { Self::get_enum_as_u8_slice(&command) };
        self.write(device, &command_as_bytes).await?;
        Ok(())
    }

    /// Send command to [`ControlPoint`] without a response.
    async fn write(&self, device: &Peripheral, data: &[u8]) -> BBitResult<()> {
        device
            .write(&self.control_point, data, WriteType::WithResponse)
            .await
            .map_err(Error::BleError)
    }

    unsafe fn get_enum_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

/// Command enum stores internal u8 array with config data.
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum ControlPointCommand {
    /// Impossible command
    Invalid = 0x00,
    /// Stop resistance or signal measurement
    StopAll = 0x01,
    /// Start signal measurement
    StartEegSignal([u8; 5]),
    /// Start resistance measurement
    StartResist([u8; 8]),
    /// Switch to dfu mode
    StartDfu = 0x04,
}

impl TryFrom<Vec<u8>> for ControlPointCommand {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > 8 {
            return Err("Source Vec<u8> length is incorrect");
        }
        match value[..] {
            [0] => Ok(ControlPointCommand::Invalid),
            [1] => Ok(ControlPointCommand::StopAll),
            [2, signal_config_ch1, signal_config_ch2, signal_config_ch3, signal_config_ch4] => {
                let data_array = [
                    2,
                    signal_config_ch1,
                    signal_config_ch2,
                    signal_config_ch3,
                    signal_config_ch4,
                ];
                Ok(ControlPointCommand::StartEegSignal(data_array))
            }
            [4, resist_config_ch1, resist_config_ch2, resist_config_ch3, resist_config_ch4, resist_sensp, resist_sensn, resist_flipp] =>
            {
                let data_array = [
                    3,
                    resist_config_ch1,
                    resist_config_ch2,
                    resist_config_ch3,
                    resist_config_ch4,
                    resist_sensp,
                    resist_sensn,
                    resist_flipp,
                ];
                Ok(ControlPointCommand::StartResist(data_array))
            }
            [5] => Ok(ControlPointCommand::StartDfu),
            _ => Err("ControlPointCommand is unknown inside Vec<u8>"),
        }
    }
}

impl TryFrom<&[u8]> for ControlPointCommand {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(ControlPointCommand::Invalid),
            [1] => Ok(ControlPointCommand::StopAll),
            [2, signal_config_ch1, signal_config_ch2, signal_config_ch3, signal_config_ch4] => {
                let data_array = [
                    2,
                    *signal_config_ch1,
                    *signal_config_ch2,
                    *signal_config_ch3,
                    *signal_config_ch4,
                ];
                Ok(ControlPointCommand::StartEegSignal(data_array))
            }
            [4, resist_config_ch1, resist_config_ch2, resist_config_ch3, resist_config_ch4, resist_sensp, resist_sensn, resist_flipp] =>
            {
                let data_array = [
                    3,
                    *resist_config_ch1,
                    *resist_config_ch2,
                    *resist_config_ch3,
                    *resist_config_ch4,
                    *resist_sensp,
                    *resist_sensn,
                    *resist_flipp,
                ];
                Ok(ControlPointCommand::StartResist(data_array))
            }
            [5] => Ok(ControlPointCommand::StartDfu),
            _ => Err("ControlPointCommand is unknown inside array[u8]"),
        }
    }
}
