use lib::bbit::device::BBitSensor;
use lib::bbit::eeg_uuids::PERIPHERAL_NAME_MATCH_FILTER;
use std::error::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "connect=DEBUG,lib=DEBUG".into()),
        )
        .init();

    let connected = BBitSensor::new()
        .await
        .unwrap()
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await
        .unwrap();

    tracing::info!("BrainBit is connected");

    let characteristics = connected.characteristics();

    // list all characteristics
    for char in characteristics {
        tracing::info!("characteristic: {char:?}");
    }
    // get device information
    let device_info = connected.device_info().await.unwrap();
    tracing::info!("{:?}", device_info);

    tracing::info!("finished");

    Ok(())
}
