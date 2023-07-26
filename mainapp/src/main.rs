use lib::bbit::device::BBitSensor;
use lib::bbit::eeg_uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};
use lib::bbit::resist::{ResistState, ResistsMeasureResult};
use lib::bbit::responses::{DeviceStatusData, Nss2Status};
use lib::EventHandler;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tracing::instrument;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use chrono::Utc;
use tokio::io::AsyncWriteExt;
use tokio::{
    fs::File,
    sync::{oneshot, Mutex},
};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
#[instrument]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .compact()
                // Display source code file paths
                .with_file(true)
                // Display source code line numbers
                .with_line_number(true)
                // Display the thread ID an event was recorded on
                .with_thread_ids(true)
                // Don't display the event's target (module path)
                .with_target(false),
            // Build the subscriber
            // .finish(),
        )
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mainapp=DEBUG,lib=DEBUG".into()),
        )
        .init();

    let connected = BBitSensor::new()
        .await?
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await?
        .listen(EventType::State)
        .listen(EventType::EegOrResistance)
        .build()
        .await?;

    let handler = connected.event_loop(Handler::new().await?).await;
    tracing::info!("BrainBit is connected, event loop is started");
    handler.start().await;

    get_finish().await?;
    handler.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

const SKIP_FIRST_RESIST_RECORDS_NUMBER: u8 = 20;
const STORE_RESIST_RECORDS_NUMBER: u8 = 20;

#[derive(Debug)]
struct Handler {
    device_status: Mutex<DeviceStatusData>,
    output: Mutex<File>,
    skipped_resist_records_number: AtomicUsize, // AtomicUsize = AtomicUsize::new(0);
    current_chanel_number_resist_measure: AtomicUsize,
    resist_measure_records: Vec<u8>,
    resist_results: Mutex<ResistState>,
}

impl Handler {
    async fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            device_status: Mutex::new(DeviceStatusData::default()),
            output: Mutex::new(File::create(format!("main_app_output.txt")).await?),
            skipped_resist_records_number: AtomicUsize::new(
                SKIP_FIRST_RESIST_RECORDS_NUMBER as usize,
            ),
            current_chanel_number_resist_measure: AtomicUsize::new(0),
            resist_measure_records: Vec::with_capacity(STORE_RESIST_RECORDS_NUMBER as usize),
            resist_results: Mutex::new(ResistState::default()),
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

#[lib::async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatusData) {
        let time = Utc::now();
        let formatted: String = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // formatted = formatted.replace("\'", "");
        let msg = format!("{formatted:?} - {status_data}\n");
        tracing::debug!(msg);
        {
            // write eeg data to file
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
        {
            // read and update local Device Status
            let mut lock = self.device_status.lock().await;
            lock.status_nss2 = status_data.status_nss2;
            lock.battery_level = status_data.battery_level;
            lock.cmd_error = status_data.cmd_error;
        }
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[instrument(skip_all)]
    async fn eeg_update(self: &mut Handler, eeg_data: Vec<u8>) {
        let time = Utc::now();
        let mut formatted: String = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        formatted = formatted.replace("\'", "");
        // let msg = format!("{formatted:?} - EEG={:>3?}\n", eeg_data);
        let msg = format!("{:>3?}\n", eeg_data);
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
        let nss2status = self.device_status.lock().await.status_nss2;
        match nss2status {
            Nss2Status::ResistTransmission => {
                tracing::debug!(msg);
                let skipped_number = self.skipped_resist_records_number.load(Ordering::Relaxed);
                if skipped_number > 0 {
                    // skip 'SKIP_FIRST_RESIST_RECORDS_NUMBER' records
                    tracing::debug!("Skipping = {:?}", skipped_number);
                    self.decrease_skipped_resist_records_number();
                    return;
                }
                let gathered_records_number = self.get_resist_measure_records_len();
                if gathered_records_number >= STORE_RESIST_RECORDS_NUMBER as usize {
                    tracing::debug!(
                        "Gathered = {:?} records for ch='{}'",
                        gathered_records_number,
                        self.current_chanel_number_resist_measure
                            .load(Ordering::Relaxed)
                    );
                    // got to next channel
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

async fn get_finish() -> color_eyre::Result<()> {
    let mut buf = String::new();
    let (tx, mut rx) = oneshot::channel();

    println!();
    print!(
        "\r({} events received) Would you like to stop? (y/N) ",
        COUNTER.load(Ordering::SeqCst)
    );
    let task = tokio::task::spawn(async move {
        loop {
            if let Ok(_) = rx.try_recv() {
                return;
            }
            io::stdout().flush().unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    loop {
        io::stdin().read_line(&mut buf)?;
        let control_letter = buf.trim().to_ascii_lowercase();
        if control_letter == "y" {
            tracing::debug!("entered letter: {control_letter:?}");
            let _ = tx.send(());
            task.await?;
            return Ok(());
        }
    }
}
