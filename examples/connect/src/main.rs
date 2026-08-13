use tracing::{debug, error};
use brainbit::bbit::device::BBitSensor;
use brainbit::bbit::uuids::PERIPHERAL_NAME_MATCH_FILTER;
use brainbit::bbit::errors::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "connect=DEBUG,brainbit=DEBUG,btleplug=TRACE".into()),
        )
        .init();

    let mut sensor = BBitSensor::new(PERIPHERAL_NAME_MATCH_FILTER.to_string())
        .await
        .expect("Invalid BBit name");

    debug!("Attempting connection");
    while !sensor.is_connected().await {
        match sensor.connect().await {
            Err(Error::NoBleAdaptor) => {
                error!("No Bluetooth adapter found");
                return Ok(());
            }
            Err(why) => error!("Could not connect: {:?}", why),
            _ => {}
        }
    }
    debug!("Connected");

    tracing::info!("BrainBit is connected");

    let characteristics = sensor.characteristics();

    // list all characteristics
    for char in characteristics? {
        tracing::info!("characteristic: {char:?}");
    }
    // get device information
    let device_info = sensor.device_info().await.unwrap();
    tracing::info!("{:?}", device_info);

    sensor.stop().await?;

    tracing::info!("finished");

    Ok(())
}
