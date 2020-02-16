#[allow(unused_imports)]
#[allow(dead_code)]

use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
//#[cfg(target_os = "windows")]
//use btleplug::winrtble::{adapter::Adapter, manager::Manager};
//#[cfg(target_os = "macos")]
//use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
use btleplug::api::{UUID, Central, Peripheral, Characteristic};



fn main() {
    let manager = Manager::new().unwrap();
    let adapter_list = manager.adapters().unwrap();
    if adapter_list.len() <= 0 {
        eprint!("Bluetooth adapter(s) were NOT found, sorry...\n");
    } else {
        for adapter in adapter_list.iter() {
            println!("connecting to BLE adapter: {:?}...", adapter.name);
            let connected_adapter: ConnectedAdapter = adapter.connect().expect("Error connecting to BLE Adapter....");
            println!("connected adapter {:?} is UP: {:?}", connected_adapter.adapter.name, connected_adapter.adapter.is_up());
            println!("adapter states : {:?}", connected_adapter.adapter.states);
            thread::sleep(Duration::from_secs(2));
            connected_adapter.start_scan().expect("Can't scan BLE adapter for connected devices...");
//            let ble_device_list = connected_adapter.peripherals().iter()
//                .find(|p| p.properties().local_name.iter().all(|name| name.contains("Neuro")));
            if connected_adapter.peripherals().is_empty() {
                eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
            } else {
                for peripheral in connected_adapter.peripherals().iter() {
                    println!("peripheral : {:?} is connected: {:?}", peripheral.properties().local_name, peripheral.is_connected());
                    if !peripheral.is_connected() {
                        println!("start connect to peripheral : {:?}...", peripheral.properties().local_name);
                        peripheral.connect().expect("Can't connect to peripheral...");
                        println!("now connected (\'{:?}\') to peripheral : {:?}...", peripheral.is_connected(), peripheral.properties().local_name);
                        let chars = peripheral.discover_characteristics();
                        if peripheral.is_connected() {
                            println!("Discover peripheral : \'{:?}\' characteristics...", peripheral.properties().local_name);
                            for chars_vector in chars.into_iter() {
                                for char_item in chars_vector.iter() {
                                    println!("{:?}", char_item);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/*
#[cfg(target_os = "linux")]
fn get_adapter_list(manager: &Manager) -> Vec<ConnectedAdapter> {
    manager.adapters().unwrap()
}
*/
