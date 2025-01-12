use chrono::Utc;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tracing::instrument;

use async_trait::async_trait;
use brainbit::bbit::resist::ResistState;
use brainbit::bbit::responses::{DeviceStatusData, Nss2Status};
use brainbit::bbit::traits::EventHandler;

const SKIP_FIRST_RESIST_RECORDS_NUMBER: usize = 20;
const STORE_RESIST_RECORDS_NUMBER: usize = 20;

#[derive(Debug)]
pub struct BBitHandler {
    /// count packets from device during measurement on one channel, then it switches to the next and starts again from Zero
    current_chanel_counter: AtomicUsize,
    /// internal device status
    device_status: Mutex<DeviceStatusData>,
    /// data file written with device data
    output: Mutex<File>,
    /// we skip 'SKIP_FIRST_RESIST_RECORDS_NUMBER' resist records on every channel
    skipped_resist_records_number: AtomicUsize,
    /// we have 4 channels, so we need to know which channel is being measured
    current_chanel_number_resist_measure: AtomicUsize,
    /// keep read records per channel measurement
    resist_measure_records: Vec<u8>,
    /// final measurement result on device after all channels are measured
    final_resist_results: Mutex<ResistState>,
}

#[async_trait]
impl EventHandler for BBitHandler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatusData) {
        let time = Utc::now();
        let formatted: String = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // formatted = formatted.replace("\'", "");
        let msg = format!("{formatted:?} - {status_data}\n");
        tracing::debug!(msg);
        {
            // write eeg data to file
            let mut lock = self.output.lock().unwrap();
            lock.write_all(msg.as_bytes()).unwrap();
        }
        {
            // read and update local Device Status
            let mut lock = self.device_status.lock().unwrap();
            lock.status_nss2 = status_data.status_nss2;
            lock.battery_level = status_data.battery_level;
            lock.cmd_error = status_data.cmd_error;
        }
        self.current_chanel_counter.fetch_add(1, Ordering::SeqCst);
    }

    #[instrument(skip_all)]
    async fn eeg_update(self: &mut BBitHandler, eeg_data: Vec<u8>) {
        let time = Utc::now();
        let mut _formatted: String = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        _formatted = _formatted.replace("\'", "");
        // let msg = format!("{_formatted:?} - EEG={:>3?}\n", eeg_data);
        let msg = format!("{:>3?}\n", eeg_data);
        {
            let mut lock = self.output.lock().unwrap();
            lock.write_all(msg.as_bytes()).expect("Can't write log...");
        }
        let nss2status = self.device_status.lock().unwrap().status_nss2;
        match nss2status {
            Nss2Status::ResistTransmission => {
                tracing::debug!(msg);
                let skipped_number = self.skipped_resist_records_number.load(Ordering::Relaxed);
                if skipped_number > 0 {
                    // skip 'SKIP_FIRST_RESIST_RECORDS_NUMBER' records
                    tracing::debug!("Skipping = {:?} packet", skipped_number);
                    self.decrease_skipped_resist_records_number();
                    return;
                }
                let gathered_records_number = self.get_resist_measure_records_len();
                if gathered_records_number >= STORE_RESIST_RECORDS_NUMBER {
                    tracing::debug!(
                        "Gathered = {:?} records for ch='{}'",
                        gathered_records_number,
                        self.current_chanel_number_resist_measure
                            .load(Ordering::Relaxed)
                    );
                    // go to next channel
                }
            }
            Nss2Status::EegTransmission => {
                tracing::debug!(msg);
            }
            Nss2Status::Stopped => {
                tracing::debug!("Stopped device in main");
            }
            _ => {}
        }
    }
}

impl BBitHandler {
    pub async fn new(log_file_name: &str) -> color_eyre::Result<Self> {
        Ok(Self {
            current_chanel_counter: AtomicUsize::new(0),
            device_status: Mutex::new(DeviceStatusData::default()),
            output: Mutex::new(File::create(log_file_name)?),
            skipped_resist_records_number: AtomicUsize::new(
                SKIP_FIRST_RESIST_RECORDS_NUMBER,
            ),
            current_chanel_number_resist_measure: AtomicUsize::new(0),
            resist_measure_records: Vec::with_capacity(STORE_RESIST_RECORDS_NUMBER),
            final_resist_results: Mutex::new(ResistState::default()),
        })
    }

    fn decrease_skipped_resist_records_number(&mut self) {
        self.current_chanel_number_resist_measure
            .fetch_sub(1, Ordering::Relaxed);
    }

    fn push_resist_data(&mut self, new_data: u8) {
        self.resist_measure_records.push(new_data);
    }

    fn get_resist_measure_records_len(&self) -> usize {
        self.resist_measure_records.len()
    }
}
