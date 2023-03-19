use uuid::Uuid;

pub const PERIPHERAL_NAME_MATCH_FILTER: &'static str = "BrainBit";
// string to match with BLE name
// pub const SUBSCRIBE_TO_CHARACTERISTIC: UUID = UUID::B128([
//     0x1B, 0xC5, 0xD5, 0xA5, 0x02, 0x00, 0xCF, 0x88, 0xE4, 0x11, 0xB9, 0xD6, 0x03, 0x00, 0x2F, 0x3D,
// ]); // reversed

pub const NOTIFY_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x6e400004_b534_f393_68a9_e50e24dcca9e);
