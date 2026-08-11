use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::io::AsyncBufReadExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use brainbit::bbit::device::BBitSensor;
use brainbit::bbit::uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};

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
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mainapp=DEBUG,brainbit=DEBUG".into()),
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

    sensor.listen(EventType::EegOrResistance);

    let paused_loop = Arc::new(AtomicBool::new(false));
    let log_file_name = "main_app_output.txt";

    loop {
        // BLE subscriptions die with the connection, so (re)subscribe them
        // before every run (after a possible reconnect).
        sensor.build().await?;

        // The event loop is run as a future we control directly, so we can
        // stop it gracefully at any moment (see the `select!` below).
        //
        // IMPORTANT: CancellationToken is one-shot — once cancelled it stays
        // cancelled forever, so every loop run needs a brand-new token.
        let user_requested_stop = {
            let handler = handler::main_handler::BBitHandler::new(log_file_name).await?;
            let shutdown_token = CancellationToken::new();
            let loop_fut =
                sensor.event_loop(handler, Arc::clone(&paused_loop), shutdown_token.clone());
            tokio::pin!(loop_fut);

            let user_requested_stop = tokio::select! {
                biased;
                _ = wait_for_stop() => true,
                _ = &mut loop_fut => false,
            };

            if user_requested_stop {
                // Graceful shutdown: cancel the token, the loop finishes and
                // returns LoopExit::Shutdown.
                info!("stopping the event loop gracefully...");
                shutdown_token.cancel();
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

/// Wait until the user presses 'y' + Enter or Ctrl+C to stop gracefully.
async fn wait_for_stop() {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buf = String::new();
    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                println!();
                return;
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
