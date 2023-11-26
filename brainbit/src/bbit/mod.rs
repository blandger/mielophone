use crate::bbit::device::{BBitResult, CommandData};
use crate::bbit::responses::DeviceStatusData;
use crate::Error;
use async_trait::async_trait;

pub(crate) mod control;
pub mod device;
pub mod eeg_uuids;
pub mod resist;
pub mod responses;
pub(crate) mod sealed;

/// Base trait for handling events coming from a BrainBit device.
#[async_trait]
pub trait EventHandler {
    /// Dispatched when a internal device status update is received.
    ///
    /// Contains the status, cmd error, battery level.
    async fn device_status_update(&self, _status: DeviceStatusData) {}

    /// Dispatched when an eeg data is received.
    ///
    /// Contains information about the O1, O2, T3, T4 + interval.
    async fn eeg_update(&mut self, _eeg_data: Vec<u8>) {}

    /// Dispatched when measurement data is received over the PMD data UUID.
    ///
    /// Contains data in a [`CommandData`].
    async fn send_command(&self, _command_data: CommandData) {}

    /// Checked at start of each event loop.
    ///
    /// Returns [`false`] if the event loop should be terminated and close connection.
    async fn should_continue(&self) -> bool {
        true
    }
}

/// List of measurement types you can request.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MeasurementType {
    /// Resistance
    Resistance(ChannelType),
    /// EEG
    Eeg,
}

/// List of channels in BBit.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelType {
    /// Channel 0, o1, occipital lobe = o, left
    O1 = 0,
    /// Channel 1, t3, temporal lobe = t, left
    T3 = 1,
    /// Channel 2, t4, temporal lobe = t, right
    T4 = 2,
    /// Channel 3, o2  (occipital lobe = o, right
    O2 = 3,
}
impl ChannelType {
    pub fn new(channel_number: u8) -> BBitResult<Self> {
        match channel_number {
            0 => Ok(ChannelType::O1),
            1 => Ok(ChannelType::T3),
            2 => Ok(ChannelType::T4),
            3 => Ok(ChannelType::O2),
            _ => Err(Error::InvalidData(
                "Incorrect channel type/number (correct value: 0-3)".to_string(),
            )),
        }
    }
}
impl Into<u8> for ChannelType {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Internal constants to assign for Resistance commands
#[derive(Debug, Copy, Clone)]
enum ADS1294ChannelInput {
    PowerDownGain6 = 0x00,
    PowerDownGain3 = 0x91,
    PowerUpGain1 = 0x48,
}

impl Into<u8> for ADS1294ChannelInput {
    fn into(self) -> u8 {
        self as u8
    }
}
