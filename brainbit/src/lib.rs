pub mod bbit;

use bbit::errors::Error;
use bbit::results::BBitResult;
use btleplug::api::{Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;
use thiserror::Error;
use uuid::Uuid;

/// Private helper to find characteristics from a [`Uuid`].
async fn find_characteristic(device: &Peripheral, uuid: Uuid) -> BBitResult<Characteristic> {
    device
        .characteristics()
        .iter()
        .find(|c| c.uuid == uuid)
        .ok_or(Error::CharacteristicNotFound)
        .cloned()
}
