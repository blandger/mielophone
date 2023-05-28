pub mod constants;

/// A list device's state/mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevMode {
    /// device is not initialized
    Invalid,
    /// device is turned on and sends advertisements
    Advertising,
    /// sensor is connected
    Connected,
    /// accum has discharged, turning off device
    PowerDown,
    /// DFU loader mode
    Dfu,
}
