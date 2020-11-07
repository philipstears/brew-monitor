use btleplug::api::{Central, Peripheral, UUID};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use std::thread;
use std::time::Duration;

pub fn main() {
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    //
    // connect to the adapter
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    let central = adapter.connect().unwrap();

    // start scanning for devices
    central.start_scan().unwrap();

    // instead of waiting, you can use central.event_receiver() to fetch a channel and
    // be notified of new devices
    thread::sleep(Duration::from_secs(2));

    // find the device we're interested in
    let light = central
        .peripherals()
        .into_iter()
        .find(|p| p.properties().local_name.iter().any(|name| name.contains("LEDBlue")))
        .unwrap();

    // connect to the device
    light.connect().unwrap();

    // discover characteristics
    light.discover_characteristics().unwrap();

    // find the characteristic we want
    let chars = light.characteristics();
    let cmd_char = chars.iter().find(|c| c.uuid == UUID::B16(0xFFE9)).unwrap();

    for _ in 0..20 {
        let color_cmd = vec![0x56, 3, 4, 5, 0x00, 0xF0, 0xAA];
        light.command(&cmd_char, &color_cmd).unwrap();
        thread::sleep(Duration::from_millis(200));
    }
}
