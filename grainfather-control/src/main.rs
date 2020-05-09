use btleplug::api::{Central, Peripheral, UUID};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};

#[cfg(target_os = "linux")]
fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

pub fn main() {
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    //
    // connect to the adapter
    let central = get_central(&manager);

    // start scanning for devices
    central.start_scan().unwrap();

    // instead of waiting, you can use central.on_event to be notified of
    // new devices
    thread::sleep(Duration::from_secs(2));

    // find the device we're interested in
      let gf = central
        .peripherals()
        .into_iter()
        .find(|p| p.properties().local_name.iter().any(|name| name.contains("LEDBlue")))
        .unwrap();

    // connect to the device
    gf.connect().unwrap();

    // discover characteristics
    gf.discover_characteristics().unwrap();

    // // find the characteristic we want
    // let chars = light.characteristics();
    // let cmd_char = chars.iter().find(|c| c.uuid == UUID::B16(0xFFE9)).unwrap();
}
