use std::error::Error;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::oneshot;
use tracing::{debug, error, instrument};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use brainbit::bbit::device::BBitSensor;
use brainbit::bbit::responses::DeviceStatusData;
use brainbit::bbit::traits::EventHandler;
use brainbit::bbit::uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "battery_level=DEBUG,brainbit=DEBUG".into()),
        )
        .init();

    let mut sensor = BBitSensor::new(PERIPHERAL_NAME_MATCH_FILTER.to_string())
        .await
        .expect("Invalid BBit name");

    debug!("Attempting connection");
    while !sensor.is_connected().await {
        match sensor.connect().await {
            Err(brainbit::bbit::errors::Error::NoBleAdaptor) => {
                error!("No Bluetooth adapter found");
                return Ok(());
            }
            Err(why) => error!("Could not connect: {:?}", why),
            _ => {}
        }
    }
    debug!("Connected");

    sensor
        .event_handler(Handler::new().await?);
    tracing::info!("BrainBit is connected, event loop is started");
    // connected.start();

    get_finish().await?;
    // sensor.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

#[derive(Debug)]
struct Handler {}

impl Handler {
    async fn new() -> color_eyre::Result<Self> {
        Ok(Self {})
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self))]
    async fn device_status_update(&self, status_data: DeviceStatusData) {
        debug!("received Status: {status_data}");
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
