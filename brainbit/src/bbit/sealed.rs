// Sealed Traits
// So crate users will not implement [`Level`] on any type to make weird [`BLESensor`]s

/// Marker for the construction level of a [`BleSensor`]
pub trait Level: internal::Level {}
impl<L> Level for L where L: internal::Level {}

/// Marker for [`BleSensor`] connected over Bluetooth
pub trait Connected: internal::Level {}

mod internal {
    /// Marker for level of [`crate::bbit::device::BBitSensor`]
    pub trait Level {}
}

/// [`BleSensor`] level for connecting to your device
pub struct Bluetooth;

impl internal::Level for Bluetooth {}

/// [`BleSensor`] level for registering data types to listen for
#[derive(Default)]
pub struct Configure {
    /// Is subscribed to device status changes, cmd errors, battery
    pub device_status: bool,
    /// Is subscribed to EEG or Resistance stream
    pub eeg_rate: bool,
}

impl internal::Level for Configure {}
impl Connected for Configure {}

/// [`BleSensor`] level for starting the event loop
pub struct EventLoop;

impl internal::Level for EventLoop {}
impl Connected for EventLoop {}
