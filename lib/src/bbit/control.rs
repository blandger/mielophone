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

    /// Send command to [`ControlPoint`].
    pub async fn send_command(&self, device: &Peripheral, data: Vec<u8>) -> BBitResult<()> {
        self.write(device, data).await?;

        Ok(())
    }

    async fn write(&self, device: &Peripheral, data: Vec<u8>) -> BBitResult<()> {
        device
            .write(&self.control_point, &data, WriteType::WithResponse)
            .await
            .map_err(Error::BleError)
    }

    /// Read data from [`ControlPoint`] (for reading the features of a device).
    pub async fn read(&self, device: &Peripheral) -> BBitResult<Vec<u8>> {
        device
            .read(&self.control_point)
            .await
            .map_err(Error::BleError)
    }
}

#[derive(Clone, Debug)]
pub enum ControlPointCommand {
    CommandStartSignal,
    CommandStopSignal,
    CommandStartResist,
}
