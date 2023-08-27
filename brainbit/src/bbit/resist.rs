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
