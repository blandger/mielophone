use crate::bbit::device::{BBitResult, CommandData};
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

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum ControlPointCommand {
    CommandInvalid = 0x00,
    CommandStop = 0x01,
    CommandStartSignal = 2,
    // CommandStartSignal([u8; 5]),
    CommandStartResist = 3,
    // CommandStartResist([u8; 8]),
    CommandStartBootloader = 0x04,
}
impl From<u8> for ControlPointCommand {
    fn from(value: u8) -> ControlPointCommand {
        match value {
            0 => ControlPointCommand::CommandInvalid,
            1 => ControlPointCommand::CommandStop,
            // 2 => ControlPointCommand::CommandStartSignal([u8; 5]),
            2 => ControlPointCommand::CommandStartSignal,
            // 4 => ControlPointCommand::CommandStartResist([u8; 8]),
            4 => ControlPointCommand::CommandStartResist,
            5 => ControlPointCommand::CommandStartBootloader,
            _ => panic!("ControlPointCommand is unknown for value = {}", value),
        }
    }
}
