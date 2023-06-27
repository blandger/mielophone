pub(crate) mod control;
pub mod device;
pub mod eeg_uuids;
pub mod responses;
pub(crate) mod sealed;

/// List of measurement types you can request.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MeasurementType {
    /// Resistance
    Resistance,
    /// EEG
    Eeg,
}

/// Internal constants to assign for Resistance commands
#[derive(Debug, Copy, Clone)]
enum ADS1294ChannelInput {
    PowerDownGain3 = 0x91,
    PowerUpGain1 = 0x48,
}

impl Into<u8> for ADS1294ChannelInput {
    fn into(self) -> u8 {
        self as u8
    }
}
