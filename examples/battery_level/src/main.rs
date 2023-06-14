use lib::bbit::device::BBiteSensor;
use lib::bbit::eeg_uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};
use lib::bbit::responses::DeviceStatusData;
use lib::EventHandler;
use std::error::Error;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tokio::sync::oneshot;
use tracing::instrument;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "battery_level=DEBUG,lib=DEBUG".into()),
        )
        .init();

    let connected = BBiteSensor::new()
        .await?
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await?
        .listen(EventType::State) // subscribe to device status changes
        .build()
        .await?
        .event_loop(Handler::new().await?)
        .await;
    tracing::info!("BrainBit is connected, event loop is started");

    get_finish().await?;
    connected.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Handler {}

impl Handler {
    async fn new() -> color_eyre::Result<Self> {
        Ok(Self {})
    }
}

#[lib::async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatusData) {
        let level = status_data.battery_level;
        tracing::debug!("received Status: {status_data:?}, battery = '{level:?}'");
        COUNTER.fetch_add(1, Ordering::SeqCst);
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
