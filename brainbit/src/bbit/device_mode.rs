use crate::bbit::channel::ChannelType;

/// List of measurement types you can request.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceMode {
    /// Resistance
    Resistance(ChannelType),
    /// EEG
    Eeg,
}

#[derive(Debug)]
pub enum LoopExit {
    /// User called shutdown_token.cancel()
    Shutdown,
    /// Notification stream has cloaes OR is_connected() returned 'false'
    Disconnected,
}