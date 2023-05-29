pub mod constants;

/// A device's state/mode type
// Probably it's returned from UUID = 6E400002-B534-F393-68A9-E50E24DCCA9E (READ / NOTIFY)
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
