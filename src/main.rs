extern crate btleplug;

use btleplug::api::{Central, CentralEvent};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    println!("Found {:?} adapters", adapters.len());
    let adapter = adapters.into_iter().nth(0).unwrap();
    println!("Connecting to adapter {:?}", adapter.addr);
    adapter.connect().expect("Failed to connect to adapter")
}

fn print_adapter_info(adapter: &ConnectedAdapter) {
    println!("Connected adapter {:?} is UP: {:?}",
    adapter.adapter.name,
    adapter.adapter.is_up());
}


fn main() {
    let manager = Manager::new().unwrap();
    let central = get_central(&manager);
    print_adapter_info(&central);

    let e = central.event_receiver().unwrap();
    central.start_scan().unwrap();
    println!("Scan enabled: {:?}", central.scan_enabled);

    
    while let Ok(event) = e.recv() {
        match event {
            CentralEvent::DeviceDiscovered(bd_addr) => {
                println!("DeviceDiscovered {:?}", bd_addr)
            }
            CentralEvent::DeviceConnected(bd_addr) => {
                println!("DeviceConnected {:?}", bd_addr)
            }
            CentralEvent::DeviceDisconnected(bd_addr) => {
                println!("DeviceDisconnected {:?}", bd_addr)
            }
            s => {println!("{:?}", s)}
        }
    }
}
