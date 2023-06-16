use lib::bbit::device::BBitSensor;
use lib::bbit::eeg_uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};
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
        .await?
        .event_loop(Handler::new().await?)
        .await;
    tracing::info!("started event loop");

    get_finish().await?;
    connected.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

#[derive(Debug)]
struct Handler {
    output: Mutex<File>,
}

impl Handler {
    async fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            output: Mutex::new(
                File::create(format!("main_app_output.txt")).await?,
                // .map_err(|error| lib::Error::HandlerError),
            ),
        })
    }
}

#[lib::async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatusData) {
        let time = Utc::now();
        let formatted = time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        tracing::debug!("received Status: {status_data:?}");
        let msg = format!("{formatted:?} - DS = {status_data:?}\n");
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[instrument(skip_all)]
    async fn eeg_update(&self, eeg_data: Vec<u8>) {
        tracing::debug!("received EEG: {eeg_data:?}");
        let msg = format!("EGG={eeg_data:?}\n");
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
        }
    }

    #[instrument(skip_all)]
    async fn resistance_update(&self, resists_data: Vec<u8>) {
        tracing::debug!("received RESIST: {resists_data:?}");
        let msg = format!("R={resists_data:?}\n");
        {
            let mut lock = self.output.lock().await;
            lock.write_all(msg.as_bytes()).await.unwrap();
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
        if buf.trim().to_ascii_lowercase() == "y" {
            let _ = tx.send(());
            task.await?;
            return Ok(());
        }
    }
}
