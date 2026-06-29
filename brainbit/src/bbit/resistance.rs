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
    pub ch_o1: ResistsMeasureResult,
    /// Right occipital region, back of the head
    pub ch_o2: ResistsMeasureResult,
    /// Left temporal lobe electrode
    pub ch_t3: ResistsMeasureResult,
    /// Right temporal lobe electrode
    pub ch_t4: ResistsMeasureResult,
}
impl Default for ResistState {
    fn default() -> Self {
        ResistState {
            ch_o1: ResistsMeasureResult::NONE,
            ch_o2: ResistsMeasureResult::NONE,
            ch_t3: ResistsMeasureResult::NONE,
            ch_t4: ResistsMeasureResult::NONE,
        }
    }
}

/// Result of measurement and computation received data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResistsMeasureResult {
    /// Result is not computed yet
    NONE,
    /// Good electrode's to head contact
    GOOD,
    /// Bad electrode's to head contact
    BAD,
}
