use std::vec::Vec;

#[derive(Copy, Clone, Debug)]
pub enum DeviceGattType {
    BrainBit,
    ColibriRed,
    ColibriBlue,
    ColibriYellow,
    ColibriWhite,
}

pub trait DeviceGattInfo {
    fn device_service_uuid() -> String;
    fn rx_characteristic_uuid() -> String;
    fn tx_characteristic_uuid() -> String;
    fn status_characteristic_uuid() -> String;
    fn get_valid_bt_names() -> Vec<String>;
}

struct BrainbitGattInfo;

impl DeviceGattInfo for BrainbitGattInfo {
    fn device_service_uuid() -> String {
        String::from("6E400001-B534-F393-68A9-E50E24DCCA9E")
    }

    fn rx_characteristic_uuid() -> String {
        String::from("6E400004-B534-F393-68A9-E50E24DCCA9E")
    }

    fn tx_characteristic_uuid() -> String {
        String::from("6E400003-B534-F393-68A9-E50E24DCCA9E")
    }

    fn status_characteristic_uuid() -> String {
        String::from("6E400002-B534-F393-68A9-E50E24DCCA9E")
    }

    fn get_valid_bt_names() -> Vec<String> {
        vec!["NeuroBLE", "BrainBit"]
    }
}
