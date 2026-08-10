use std::{
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, instrument};
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
    sensor.build().await?;

    let paused_loop = Arc::new(AtomicBool::new(false));
    let shutdown_token = CancellationToken::new();

    let log_file_name = "main_app_output.txt";
    let loop_result = sensor
        .event_loop(handler::main_handler::BBitHandler::new(log_file_name).await?, paused_loop, shutdown_token).await?;
    tracing::info!("BrainBit is connected, event loop is started");

    get_finish(&AtomicUsize::default()).await?;
    // handler.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

async fn get_finish(counter: &AtomicUsize) -> color_eyre::Result<()> {
    let mut buf = String::new();
    let (tx, mut rx) = oneshot::channel();

    println!();
    print!(
        "\r({} events received) Would you like to stop? (y/N) ",
        counter.load(Ordering::SeqCst)
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
            debug!("entered letter: {control_letter:?}");
            let _ = tx.send(());
            task.await?;
            return Ok(());
        }
    }
}
