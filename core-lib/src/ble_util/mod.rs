#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, adapter::ConnectedAdapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

pub const PERIPHERAL_NAME_MATCH_FILTER:&'static str = "Neuro"; // string to match with BLE name
pub const SUBSCRIBE_TO_CHARACTERISTIC: UUID = UUID::B128(
        [0x1B, 0xC5, 0xD5, 0xA5, 0x02, 0x00, 0xCF, 0x88, 0xE4, 0x11, 0xB9, 0xD6, 0x03, 0x00, 0x2F, 0x3D]); // reversed
        //3D:2F:00:03:D6:B9:11:E4:88:CF:00:02:A5:D5:C5:1B

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn connect_to(adapter: &Adapter) -> ConnectedAdapter {
    adapter.connect().expect("Error connecting to BLE Adapter....") //linux
}
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn print_adapter_info(adapter: &ConnectedAdapter) {
    println!("connected adapter {:?} is UP: {:?}", adapter.adapter.name, adapter.adapter.is_up());
    println!("adapter states : {:?}", adapter.adapter.states);
}

/*
#[cfg(target_os = "windows")]
pub fn connect_to(adapter: &Adapter) -> &Adapter {
    adapter //windows 10
}
#[cfg(target_os = "windows")]
pub fn print_adapter_info(_adapter: &Adapter) {
    println!("adapter info can't be printed on Windows 10");
}*/
