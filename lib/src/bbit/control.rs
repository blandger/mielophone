use crate::bbit::device::BBitResult;
use crate::bbit::eeg_uuids::WRITE_COMMAN_UUID;
use crate::{find_characteristic, Error};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use core::slice::from_raw_parts;

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
        tracing::debug!("Send command to sensor: {:04X?}", data);
        self.write(device, data).await?;
        Ok(())
    }

    /// Send command to [`ControlPoint`] waiting for a response from device.
    pub async fn send_control_command_enum(
        &self,
        device: &Peripheral,
        command: &ControlPointCommand,
    ) -> BBitResult<()> {
        tracing::debug!("Send control enum command to sensor: {command:?}");
        let command_as_bytes: &[u8] = unsafe { Self::get_enum_as_u8_slice(&command) };
        self.write(device, &command_as_bytes).await?;
        tracing::debug!(
            "Written control enum command to sensor: {:04X?}",
            command_as_bytes
        );
        Ok(())
    }

    /// Send command to [`ControlPoint`] without a response.
    async fn write(&self, device: &Peripheral, data: &[u8]) -> BBitResult<()> {
        tracing::debug!("Write data command to sensor: {:04X?}", data);
        device
            .write(&self.control_point, data, WriteType::WithResponse)
            .await
            .map_err(Error::BleError)
    }

    unsafe fn get_enum_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
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
    StartEegSignal([u8; 4]),
    /// Start resistance measurement
    StartResist([u8; 7]),
    /// Switch to dfu mode
    StartDfu = 0x04,
}

impl ControlPointCommand {
    const fn as_u8(&self) -> u8 {
        match *self {
            Self::Invalid => 0x00,
            Self::StopAll => 0x01,
            Self::StartEegSignal(array) => array[0],
            Self::StartResist(array) => array[0],
            Self::StartDfu => 0x04,
        }
    }

    const fn as_bytes(&self) -> u8 {
        match *self {
            Self::Invalid => 0,
            Self::StopAll => 1,
            Self::StartEegSignal(array) => array[0],
            Self::StartResist(array) => array[0],
            Self::StartDfu => 4,
        }
    }
}

impl Into<u8> for ControlPointCommand {
    fn into(self) -> u8 {
        match self {
            Self::Invalid => 0,
            Self::StopAll => 1,
            Self::StartEegSignal(array) => array[0],
            Self::StartResist(array) => array[0],
            Self::StartDfu => 4,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bbit::ADS1294ChannelInput;

    #[test]
    fn test_resist_command_layout() {
        let command = ControlPointCommand::StartResist([
            ADS1294ChannelInput::PowerDownGain3.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            0x00,
            0x00,
            0x00,
        ]);
        tracing::debug!("source = {command:?}");

        let expected: [u8; 8] = [0x03, 0x91, 0x48, 0x48, 0x48, 0x00, 0x00, 0x00];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: &[u8] = unsafe { ControlPoint::get_enum_as_u8_slice(&command) };
        tracing::debug!("commands as bytes = {command:?}");

        assert_eq!(expected, command_as_bytes)
    }

    #[test]
    fn test_stop_command() {
        let command = ControlPointCommand::StopAll;
        tracing::debug!("source = {command:?}");

        let expected: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: &[u8] = unsafe { ControlPoint::get_enum_as_u8_slice(&command) };
        tracing::debug!("commands as bytes = {command:?}");

        assert_eq!(expected, command_as_bytes)
    }

    #[test]
    fn test_eeg_command() {
        let command = ControlPointCommand::StartEegSignal([0x00, 0x00, 0x00, 0x00]);
        tracing::debug!("source = {command:?}");

        let expected: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: &[u8] = unsafe { ControlPoint::get_enum_as_u8_slice(&command) };
        tracing::debug!("commands as bytes = {command:?}");

        assert_eq!(expected, command_as_bytes)
    }
}
