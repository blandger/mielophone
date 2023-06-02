use lib::bbit::device::BleSensor;
use lib::bbit::eeg_uuids::PERIPHERAL_NAME_MATCH_FILTER;
use std::error::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mainapp=DEBUG,bbit=DEBUG".into()),
        )
        .init();

    let connected = BleSensor::new()
        .await
        .unwrap()
        .block_connect(PERIPHERAL_NAME_MATCH_FILTER)
        .await
        .unwrap();

    tracing::info!("connected !");

    // Following error is possible on Linux !
    // thread 'main' panicked at 'Can't get Battery Level due to error...: BleError(Other(DbusError(
    // D-Bus error: Read not permitted (org.bluez.Error.NotPermitted))))', mainapp/src/main.rs
    /*    tracing::info!(
        "battery level is: {}%",
        connected
            .battery()
            .await
            .expect("Can't get Battery Level due to error...")
    );*/

    let device_info = connected.device_info().await.unwrap();
    tracing::info!("{:?}", device_info);

    Ok(())
}
