extern crate bluez;
extern crate btleplug;

use btleplug::api::{BDAddr, Central, CentralEvent, Peripheral};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use std::convert::TryInto;
use std::str::FromStr;

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    println!("Found {:?} adapters", adapters.len());
    let adapter = adapters.into_iter().nth(0).unwrap();
    println!("Connecting to adapter {:?}", adapter.addr);
    adapter.connect().expect("Failed to connect to adapter")
}

fn print_adapter_info(adapter: &ConnectedAdapter) {
    println!(
        "Connected adapter {:?} is UP: {:?}",
        adapter.adapter.name,
        adapter.adapter.is_up()
    );
}

fn u16_bytes(arr: &[u8]) -> [u8; 2] {
    arr[0..=1].try_into().expect("Wrong sizes array")
}

struct RuuviDataPoint {
    data_format: u8,
    temperature: f32,
    humidity: f32,
    pressure: u32,
    acc_x: i16,
    acc_y: i16,
    acc_z: i16,
    power_info: u16,
    movement_counter: u8,
    sequence_number: u16,
    mac_address: u64,
}

fn parse_advertisement_message(msg: &[u8]) -> RuuviDataPoint {
    if msg.len() != 26 {
        println!("Got unexpected length");
    }
    let data_format = *msg.get(2).unwrap();
    if data_format != 0x05 {
        println!("Got unexpected data format: {:x?}", data_format);
    }
    let temperature = i16::from_be_bytes(u16_bytes(&msg[3..=4]));
    let temperature = 0.005f32 * temperature as f32;

    let humidity = i16::from_be_bytes(u16_bytes(&msg[5..=6]));
    let humidity = 0.0025f32 * humidity as f32;

    let pressure = u16::from_be_bytes(u16_bytes(&msg[7..=8])) as u32;
    let pressure = pressure - 50_000;

    let acc_x = i16::from_be_bytes(u16_bytes(&msg[9..=10]));
    let acc_y = i16::from_be_bytes(u16_bytes(&msg[11..=12]));
    let acc_z = i16::from_be_bytes(u16_bytes(&msg[13..=14]));

    let power_info = u16::from_be_bytes(u16_bytes(&msg[15..=16]));

    let movement_counter = *msg.get(17).unwrap();
    let sequence_number = u16::from_be_bytes(u16_bytes(&msg[18..=19])) as u16;

    let mut v: Vec<u8> = vec![0x0, 0x0];
    v.extend_from_slice(&msg[20..=25]);
    let mac_address = u64::from_be_bytes(v.as_slice().try_into().unwrap());

    RuuviDataPoint {
        data_format,
        temperature,
        pressure,
        humidity,
        acc_x,
        acc_y,
        acc_z,
        power_info,
        movement_counter,
        sequence_number,
        mac_address,
    }
}

fn main() {
    let test_addr = BDAddr::from_str(&"D2:4C:92:E8:F4:3F").unwrap();
    let data_key = 0x0499;

    let manager = Manager::new().unwrap();
    let central = get_central(&manager);
    print_adapter_info(&central);

    let e = central.event_receiver().unwrap();
    central.start_scan().unwrap();
    println!("Scan enabled: {:?}", central.scan_enabled);

    while let Ok(event) = e.recv() {
        match event {
            CentralEvent::DeviceDiscovered(bd_addr) => println!("DeviceDiscovered {:?}", bd_addr),
            CentralEvent::DeviceConnected(bd_addr) => println!("DeviceConnected {:?}", bd_addr),
            CentralEvent::DeviceDisconnected(bd_addr) => {
                println!("DeviceDisconnected {:?}", bd_addr)
            }
            CentralEvent::DeviceUpdated(bd_addr) => {
                let sender = central.peripheral(bd_addr).unwrap();
                if sender.address() == test_addr {
                    println!("{:?} sent an update", sender.properties().address);
                    let data = sender.properties().manufacturer_data.expect("No data");
                    let msg = parse_advertisement_message(&data);
                }
            }
            s => println!("{:?}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_message() {
        let data = [
            0x99, 0x04, 0x05, 0x12, 0xFC, 0x53, 0x94, 0xC3, 0x7C, 0x00, 0x04, 0xFF, 0xfc, 0x04,
            0x0c, 0xac, 0x36, 0x42, 0x00, 0xcd, 0xcB, 0xb8, 0x33, 0x4c, 0x88, 0x4F,
        ];
    }
}
