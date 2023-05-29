pub mod bbit;

/// Error type for general lib errors and internal btleplug Ble errors
#[derive(Debug, Error)]
pub enum Error {
    /// Bluetooth adapter is not found on attempt to scan it
    #[error("No BLE adaptor")]
    NoBleAdaptor,
    /// Could not connect to a device by filter
    #[error("No BLE device")]
    NoDevice,
    /// Device looks as it's not connected, but command was called
    #[error("Not connected")]
    NotConnected,
    /// UUID device's characteristic is missing
    #[error("Characteristic not found")]
    CharacteristicNotFound,
    /// EEG Data packets received from device is not parsed
    #[error("Invalid data")]
    InvalidData,
    /// The command did not return a response
    #[error("No command response")]
    NoControlPointResponse,
    /// An error occurred in the underlying BLE library.
    #[error("BLE error: {0}")]
    BleError(#[from] btleplug::Error),
}

/// Base trait for handling events coming from a BrainBit device.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Dispatched when a battery update is received.
    ///
    /// Contains the current battery level.
    async fn battery_update(&self, _battery_level: u8) {}

    /// Dispatched when an eeg data is received.
    ///
    /// Contains information about the O1, O2, T3, T4 + interval.
    async fn eeg_update(&self, _ctx: &BleSensor, _eeg_data: EegData) {}

    /// Dispatched when measurement data is received over the PMD data UUID.
    ///
    /// Contains data in a [`PmdRead`].
    async fn send_command(&self, _ctx: &BleSensor, _data: CommandData) {}

    /// Checked at start of each event loop.
    ///
    /// Returns [`false`] if the event loop should be terminated and close connection.
    async fn should_continue(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 3, 5);
    }
}
