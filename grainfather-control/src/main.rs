use std::error::Error;
use bluez::client::*;
use std::time::Duration;
use async_std::task::block_on;
use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;
use grainfather_control::*;
use chrono::prelude::*;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = BlueZClient::new().unwrap();

    let version = client.get_mgmt_version().await?;
    eprintln!("management version: {}.{}", version.version, version.revision);

    let controllers = client.get_controller_list().await?;

    // find the first controller we can power on
    let (controller, info) = controllers
        .into_iter()
        .filter_map(|controller| {
            let info = block_on(client.get_controller_info(controller)).ok()?;

            if info.supported_settings.contains(ControllerSetting::Powered) {
                Some((controller, info))
            } else {
                None
            }
        })
    .nth(0)
        .expect("no usable controllers found");

    if !info.current_settings.contains(ControllerSetting::Powered) {
        eprintln!("powering on bluetooth controller {}", controller);
        client.set_powered(controller, true).await?;
    }

    // scan for some devices
    // to do this we'll need to listen for the Device Found event
    let _gf_1 = [
        0x00, 0x00, 0xcd, 0xd0, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb
    ];

    let _gf_2 = [
        0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0xd0, 0xcd, 0x00, 0x00
    ];

    let service_ids = vec![]; //gf_2];

    client
        .start_service_discovery(
            controller,
            AddressTypeFlag::LEPublic |
            AddressTypeFlag::LERandom,
            127,
            service_ids.clone(),
            )
        .await?;

    // just wait for discovery forever
    loop {
        // process() blocks until there is a response to be had
        let response = client.process().await?;

        match response.event {
            Event::DeviceFound {
                address,
                address_type,
                flags,
                rssi,
                eir_data,
                ..
            } => {
                for entry in EIRData::from(eir_data.as_ref()).into_iter() {
                    if let EIREntry::ManufacturerSpecific(ms) = entry {
                        if let ManufacturerSpecificEntry::Apple(apple) = ms {
                            if let AppleEntry::Beacon(beacon) = apple {
                                log_beacon(beacon);
                            }
                        }
                    }
                }
            }
            Event::Discovering {
                discovering,
                address_type,
            } => {
                eprintln!("discovering: {} {:?}", discovering, address_type);

                // if discovery ended, turn it back on
                if !discovering {
                    client
                        .start_service_discovery(
                            controller,
                            AddressTypeFlag::LEPublic
                            | AddressTypeFlag::LERandom,
                            127,
                            service_ids.clone(),
                            )
                        .await?;
                }
            }
            Event::DeviceConnected {
                address,
                address_type,
                flags,
                eir_data,
            } => {
                eprintln!(
                    "[{:?}] device connected {} ({:?}) with flags {:?}",
                    controller, address, address_type, flags
                    );
                let eir_entries = EIRData::from(eir_data.as_ref()).into_iter().collect::<Vec<_>>();
                eprintln!("Entries: {:?}", eir_entries);

            }
            other => {
                eprintln!("got: {:?}", other);
            },
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

fn log_beacon(beacon: Beacon) {
    if let Beacon::Tilt { color, celsius, gravity, .. }  = beacon {
        let now = Utc::now();
        println!("at={:?} which={:?} celsius={:?} gravity={:?}", now, color, celsius, gravity);
    }
}
