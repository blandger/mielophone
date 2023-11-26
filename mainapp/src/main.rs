use std::sync::Arc;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use tokio::sync::oneshot;
use tracing::instrument;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use lib::bbit::device::BBitSensor;
use lib::bbit::eeg_uuids::{EventType, PERIPHERAL_NAME_MATCH_FILTER};

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

    let connected = BBitSensor::new()
        .await?
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await?
        .listen(EventType::State)
        .listen(EventType::EegOrResistance)
        .build()
        .await?;

    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let handler = connected
        .event_loop(handler::main_handler::BBitHandler::new(Arc::clone(&counter)).await?)
        .await;
    tracing::info!("BrainBit is connected, event loop is started");
    handler.start().await;

    get_finish(counter).await?;
    handler.stop().await;

    tracing::info!("stopped the event loop, finishing");

    Ok(())
}

async fn get_finish(counter: Arc<AtomicUsize>) -> color_eyre::Result<()> {
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
            tracing::debug!("entered letter: {control_letter:?}");
            let _ = tx.send(());
            task.await?;
            return Ok(());
        }
    }
}
