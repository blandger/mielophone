use chrono::Utc;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use tracing::{debug, instrument};

use async_trait::async_trait;
use brainbit::bbit::device::BBitSensor;
use brainbit::bbit::device_status::{DeviceStatus, Nss2Status};
use brainbit::bbit::traits::EventHandler;

#[derive(Debug)]
pub struct FileWriteHandler {
    /// internal device status
    device_status: Mutex<DeviceStatus>,
    /// data file written with device data
    output: Mutex<File>,
}

#[async_trait]
impl EventHandler for FileWriteHandler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatus) {
        let time = Utc::now();
        let formatted: String = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // formatted = formatted.replace("\'", "");
        let msg = format!("{formatted} - {status_data}\n");
        debug!(msg);
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
    }

    #[instrument(skip_all)]
    async fn eeg_update(&self, _ctx: &BBitSensor, eeg_data: Vec<u8>) {
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
                debug!(msg);
            }
            Nss2Status::EegTransmission => {
                debug!(msg);
            }
            Nss2Status::Stopped => {
                debug!("Stopped device in main");
            }
            _ => {
                debug!("{:?}", nss2status);
            }
        }
    }
}

impl FileWriteHandler {
    pub async fn new(log_file_name: &str) -> color_eyre::Result<Self> {
        Ok(Self {
            device_status: Mutex::new(DeviceStatus::default()),
            output: Mutex::new(File::create(log_file_name)?),
        })
    }
}
