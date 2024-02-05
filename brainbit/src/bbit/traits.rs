use crate::bbit::device::CommandData;
use crate::bbit::responses::DeviceStatusData;
use async_trait::async_trait;

/// Base trait for handling events coming from a BrainBit device.
#[async_trait]
pub trait EventHandler {
    /// Dispatched when an internal device status update is received.
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
