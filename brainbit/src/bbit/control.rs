use crate::bbit::device::BBitResult;
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
        tracing::debug!("Send command to sensor: {:02X?}", data);
        self.write(device, data).await?;
        Ok(())
    }

    /// Send command to [`ControlPoint`] waiting for a response from device.
    pub async fn send_control_command_enum(
        &self,
        device: &Peripheral,
        command: ControlPointCommand,
    ) -> BBitResult<()> {
        tracing::debug!("Send control enum command to sensor: {command:?}");
        let command_as_bytes: Vec<u8> =
            <ControlPointCommand as TryInto<Vec<u8>>>::try_into(command).unwrap();
        self.write(device, command_as_bytes.as_slice()).await?;
        tracing::debug!(
            "Written control enum command to sensor: {:02X?}",
            command_as_bytes
        );
        Ok(())
    }

    /// Send command to [`ControlPoint`] without a response.
    async fn write(&self, device: &Peripheral, data: &[u8]) -> BBitResult<()> {
        tracing::debug!("Write data command to sensor: {:02X?}", data);
        device
            .write(&self.control_point, data, WriteType::WithResponse)
            .await
            .map_err(Error::BleError)
    }

    /*    unsafe fn get_enum_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }*/
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlCommandType {
    /// Impossible command
    Invalid = 0x00,
    /// Stop resistance or signal measurement
    StopAll = 0x01,
    /// Start signal measurement
    StartEegSignal = 0x02,
    /// Start resistance measurement
    StartResist = 0x03,
    /// Switch to dfu mode
    StartDfu = 0x04,
}

/// Command enum stores internal u8 array with config data.
#[derive(Clone, Debug)]
pub struct ControlPointCommand {
    /// type of command
    pub cmd_type: ControlCommandType,
    /// optional data
    pub data: Option<Vec<u8>>,
}

impl ControlPointCommand {
    pub fn new(cmd_type: ControlCommandType, data: Option<Vec<u8>>) -> Self {
        Self {
            cmd_type: cmd_type,
            data: data,
        }
    }
}

impl TryInto<Vec<u8>> for ControlPointCommand {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self.cmd_type {
            ControlCommandType::Invalid => Ok(Vec::from([0x00])),
            ControlCommandType::StopAll => Ok(Vec::from([0x01])),
            ControlCommandType::StartEegSignal => {
                let mut cmd = Vec::from([0x02]);
                cmd.extend(self.data.unwrap());
                Ok(cmd)
            }
            ControlCommandType::StartResist => {
                let mut cmd = Vec::from([0x03]);
                cmd.extend(self.data.unwrap());
                Ok(cmd)
            }
            ControlCommandType::StartDfu => Ok(Vec::from([0x01])),
        }
    }
}

impl TryFrom<&[u8]> for ControlPointCommand {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(ControlPointCommand::new(ControlCommandType::Invalid, None)),
            [1] => Ok(ControlPointCommand::new(ControlCommandType::StopAll, None)),
            [2, signal_config_ch1, signal_config_ch2, signal_config_ch3, signal_config_ch4] => {
                let data_array = [
                    *signal_config_ch1,
                    *signal_config_ch2,
                    *signal_config_ch3,
                    *signal_config_ch4,
                ];
                Ok(ControlPointCommand::new(
                    ControlCommandType::StartEegSignal,
                    Some(Vec::from(data_array)),
                ))
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
                Ok(ControlPointCommand::new(
                    ControlCommandType::StartResist,
                    Some(Vec::from(data_array)),
                ))
            }
            [5] => Ok(ControlPointCommand::new(ControlCommandType::StartDfu, None)),
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
        let cmd_data = [
            ADS1294ChannelInput::PowerDownGain3.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            ADS1294ChannelInput::PowerUpGain1.into(),
            0x00,
            0x00,
            0x00,
        ];
        let command =
            ControlPointCommand::new(ControlCommandType::StartResist, Some(Vec::from(cmd_data)));
        tracing::debug!("source = {command:?}");

        let expected: [u8; 8] = [0x03, 0x91, 0x48, 0x48, 0x48, 0x00, 0x00, 0x00];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: Vec<u8> =
            <ControlPointCommand as TryInto<Vec<u8>>>::try_into(command).unwrap();
        tracing::debug!("commands as bytes = {:?}", &command_as_bytes);

        assert_eq!(&expected, command_as_bytes.as_slice())
    }

    #[test]
    fn test_stop_command() {
        let command = ControlPointCommand::new(ControlCommandType::StopAll, None);
        tracing::debug!("source = {command:?}");

        let expected: [u8; 1] = [0x01];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: Vec<u8> =
            <ControlPointCommand as TryInto<Vec<u8>>>::try_into(command).unwrap();
        tracing::debug!("commands as bytes = {:?}", &command_as_bytes);

        assert_eq!(&expected, command_as_bytes.as_slice())
    }

    #[test]
    fn test_eeg_command() {
        let cmd_data = [0x00, 0x00, 0x00, 0x00];
        let command = ControlPointCommand::new(
            ControlCommandType::StartEegSignal,
            Some(Vec::from(cmd_data)),
        );
        tracing::debug!("source = {command:?}");

        let expected: [u8; 5] = [0x02, 0x00, 0x00, 0x00, 0x00];
        tracing::debug!("expected = {command:?}");
        let command_as_bytes: Vec<u8> =
            <ControlPointCommand as TryInto<Vec<u8>>>::try_into(command).unwrap();
        tracing::debug!("commands as bytes = {:?}", &command_as_bytes);

        assert_eq!(&expected, command_as_bytes.as_slice())
    }
}
