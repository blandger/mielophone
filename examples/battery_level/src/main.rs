use std::error::Error;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::time;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use brainbit::bbit::device::BBitSensor;
use brainbit::bbit::device_status::DeviceStatus;
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

    // Global Ctrl+C handler: cancels the app token at ANY moment — while
    // connecting, running the event loop or reconnecting — and that triggers
    // graceful shutdown of the loop and the device.
    let shutdown_token = CancellationToken::new();
    tokio::spawn({
        let token = shutdown_token.clone();
        async move {
            loop {
                if tokio::signal::ctrl_c().await.is_ok() {
                    println!("\nCtrl+C received, stopping gracefully...");
                    token.cancel();
                }
            }
        }
    });

    let mut sensor = BBitSensor::new(PERIPHERAL_NAME_MATCH_FILTER.to_string())
        .await?;

    debug!("Attempting connection");
    while !sensor.is_connected().await {
        if shutdown_token.is_cancelled() {
            info!("Ctrl+C received, exiting");
            return Ok(());
        }
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

    sensor.listen(EventType::State);

    let paused_loop = Arc::new(AtomicBool::new(false));

    loop {
        // BLE subscriptions die with the connection, so (re)subscribe them
        // before every run (after a possible reconnect).
        sensor.build().await?;

        // The event loop is run as a future we control directly, so we can
        // stop it gracefully at any moment (see the `select!` below).
        //
        // NOTE: a CancellationToken is one-shot — once cancelled it stays
        // cancelled forever, so every loop run gets a brand-new CHILD token
        // (which is also cancelled by Ctrl+C via the parent app token).
        let user_requested_stop = {
            let handler = Handler::new().await?;
            let run_token = shutdown_token.child_token();
            let loop_fut =
                sensor.event_loop(handler, Arc::clone(&paused_loop), run_token.clone());
            tokio::pin!(loop_fut);

            let user_requested_stop = tokio::select! {
                biased;
                _ = wait_for_stop() => true,               // 'y' pressed
                _ = shutdown_token.cancelled() => true,    // Ctrl+C pressed
                _ = &mut loop_fut => false,                // loop exited on its own
            };

            if user_requested_stop {
                // Graceful shutdown: cancel the token, the loop finishes and
                // returns LoopExit::Shutdown.
                info!("stopping the event loop gracefully...");
                run_token.cancel();
                let exit = (&mut loop_fut).await?;
                info!("event loop exited: {exit:?}");
            } else {
                // The loop exited on its own (e.g. the headset went away).
                let exit = (&mut loop_fut).await?;
                warn!("event loop exited on its own: {exit:?}");
            }
            user_requested_stop
        };

        if user_requested_stop {
            // Stop the device: stop any measurement, unsubscribe, disconnect.
            sensor.stop().await?;
            info!("device stopped, finished");
            return Ok(());
        }

        // The headset is gone: reconnect and restart the event loop.
        warn!("reconnecting...");
        while !sensor.is_connected().await {
            if shutdown_token.is_cancelled() {
                info!("Ctrl+C received, exiting");
                return Ok(());
            }
            match sensor.connect().await {
                Err(brainbit::bbit::errors::Error::NoBleAdaptor) => {
                    error!("No Bluetooth adapter found");
                    return Ok(());
                }
                Err(why) => error!("Could not connect: {:?}", why),
                _ => {}
            }
        }
    }
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
    async fn device_status_update(&self, status_data: DeviceStatus) {
        debug!("received Status: {status_data}");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}

/// Wait until the user presses 'y' + Enter to stop gracefully.
/// (Ctrl+C is handled globally by the Ctrl+C handler task in `main`.)
/// While waiting, periodically prints how many events were received.
async fn wait_for_stop() {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buf = String::new();
    let mut ticker = time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            biased;
            _ = ticker.tick() => {
                print!(
                    "\r({} events received) press 'y' or Ctrl+C to stop gracefully ",
                    COUNTER.load(Ordering::SeqCst)
                );
                let _ = io::stdout().flush();
            }
            res = stdin.read_line(&mut buf) => {
                println!();
                match res {
                    // EOF (Ctrl+D) or read error: stop as well
                    Ok(0) | Err(_) => return,
                    // 'y' + Enter: graceful stop
                    Ok(_) if buf.trim().to_ascii_lowercase() == "y" => return,
                    // anything else: keep waiting
                    Ok(_) => buf.clear(),
                }
            }
        }
    }
}
