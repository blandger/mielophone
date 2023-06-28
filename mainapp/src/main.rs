use lib::bbit::device::BBitSensor;
use lib::bbit::eeg_uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};
use lib::bbit::resist::{ResistState, ResistsMeasureResult};
use lib::bbit::responses::DeviceStatusData;
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
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mainapp=DEBUG,lib=DEBUG".into()),
        )
        .init();

    let connected = BBitSensor::new()
        .await?
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await?
        .listen(EventType::State)
        .listen(EventType::Resistance)
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
const STORE_RESIST_RECORDS_NUMBER: u8 = 10;

#[derive(Debug)]
struct Handler {
    output: Mutex<File>,
    skipped_resist_records_number: u8,
    current_chanel_number_resist_measure: u8,
    resist_measure_records: Vec<u8>,
    resist_results: ResistState,
}

impl Handler {
    async fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            output: Mutex::new(
                File::create(format!("main_app_output.txt")).await?,
                // .map_err(|error| lib::Error::HandlerError),
            ),
            skipped_resist_records_number: SKIP_FIRST_RESIST_RECORDS_NUMBER,
            current_chanel_number_resist_measure: 0,
            resist_measure_records: Vec::with_capacity(STORE_RESIST_RECORDS_NUMBER as usize),
            resist_results: ResistState::default(),
        })
    }

    fn decrease_skipped_resist_records_number(&mut self) {
        self.current_chanel_number_resist_measure -= 1;
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
        let formatted = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let msg = format!("{formatted:?} - DS = {status_data:?}");
        tracing::debug!(msg);
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[instrument(skip_all)]
    async fn eeg_update(&self, eeg_data: Vec<u8>) {
        let msg = format!("EGG={eeg_data:?}\n");
        tracing::debug!("{msg:?}");
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
    }

    #[instrument(skip_all)]
    async fn resistance_update<'a>(self: &'a mut Handler, resists_data: Vec<u8>) {
        let time = Utc::now();
        let formatted = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let msg = format!("{formatted:?} - R={resists_data:?}");
        tracing::debug!(msg);
        if self.skipped_resist_records_number > 0 {
            // skip 'SKIP_FIRST_RESIST_RECORDS_NUMBER' records
            tracing::debug!("Skip = {:?}", self.skipped_resist_records_number);
            Self::decrease_skipped_resist_records_number(self);
            return;
        }
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
        let gethered_records_number = self.get_resist_measure_records_len();
        if gethered_records_number >= STORE_RESIST_RECORDS_NUMBER as usize {
            tracing::debug!(
                "Gathered = {:?} records for ch='{}'",
                gethered_records_number,
                self.current_chanel_number_resist_measure
            );
            // got to next channel
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
