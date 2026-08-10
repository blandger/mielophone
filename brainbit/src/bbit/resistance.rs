use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResistanceConfig {
    /// How many packets to accumulate per channel before averaging
    pub packets_per_channel: u32,
    /// Resistance threshold below which the channel is considered “good” or "bad"
    pub good_threshold_ohm: f32,
    /// Should I do a resistance scan automatically at the beginning of a session?
    pub run_resistance_measure_at_session_start: bool,
    /// Do periodic resistance check period during operation (None = do not do)
    pub resistance_check_interval: Option<Duration>,
}

/// Structure for storing result of resistance measurement on every electrode
/// Data is computed and quality of electrode's contact
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResistState {
    /// Left occipital region, back of the head
    pub ch_o1: ChannelQuality,
    /// Right occipital region, back of the head
    pub ch_o2: ChannelQuality,
    /// Left temporal lobe electrode
    pub ch_t3: ChannelQuality,
    /// Right temporal lobe electrode
    pub ch_t4: ChannelQuality,
}
impl Default for ResistState {
    fn default() -> Self {
        ResistState {
            ch_o1: ChannelQuality::NONE,
            ch_o2: ChannelQuality::NONE,
            ch_t3: ChannelQuality::NONE,
            ch_t4: ChannelQuality::NONE,
        }
    }
}
impl Display for ResistState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T3={}--T4={}\n
 O1={}-O2={}",
            self.ch_t3,
            self.ch_t4,
            self.ch_o1,
            self.ch_o2,
        )
    }
}

/// Result of measurement and computation received data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelQuality {
    /// Result is not computed yet
    NONE,
    /// Good electrode's to head contact
    GOOD,
    /// Bad electrode's to head contact
    BAD,
}
impl Display for ChannelQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}'",
            match self {
                ChannelQuality::NONE => "0",
                ChannelQuality::GOOD => "✅",
                ChannelQuality::BAD => "❌",
            }
        )
    }
}