// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use lib::bbit::constants::{EEG_DATA_NOTIFY_CHARACTERISTIC_UUID, PERIPHERAL_NAME_MATCH_FILTER};
use std::error::Error;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    Ok(())
}
