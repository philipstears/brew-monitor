use std::error::Error;
use std::time::Duration;
use std::convert::TryFrom;
use std::convert::TryInto;

use async_std::task::block_on;

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;

use btleplug::api::{Central, Peripheral, UUID};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};

use bm_bluetooth::*;
use bm_tilt::*;
use bm_grainfather::*;

use chrono::prelude::*;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut bluez_client = BlueZClient::new().unwrap();

    let version = bluez_client.get_mgmt_version().await?;
    eprintln!("management version: {}.{}", version.version, version.revision);

    let bluez_controllers = bluez_client.get_controller_list().await?;

    // find the first controller we can power on
    let (bluez_controller, bluez_info) = bluez_controllers
        .into_iter()
        .filter_map(|controller| {
            let info = block_on(bluez_client.get_controller_info(controller)).ok()?;

            if info.supported_settings.contains(ControllerSetting::Powered) {
                Some((controller, info))
            } else {
                None
            }
        })
    .nth(0)
        .expect("no usable controllers found");

    if !bluez_info.current_settings.contains(ControllerSetting::Powered) {
        eprintln!("powering on bluetooth controller {}", bluez_controller);
        bluez_client.set_powered(bluez_controller, true).await?;
    }

    let btle_manager = Manager::new().unwrap();
    let btle_adapters = btle_manager.adapters().unwrap();
    let btle_adapter = btle_adapters.into_iter().filter(|adapter| adapter.addr.address == bluez_info.address.as_ref()).nth(0).unwrap();
    let btle_central = btle_adapter.connect().unwrap();

    // NOTE: could filter here to just GF if needed
    let service_ids = vec![];

    bluez_client
        .start_service_discovery(
            bluez_controller,
            AddressTypeFlag::LEPublic |
            AddressTypeFlag::LERandom,
            127,
            service_ids.clone(),
            )
        .await?;

    // just wait for discovery forever
    loop {
        // process() blocks until there is a response to be had
        let response = bluez_client.process().await?;

        match response.event {
            Event::DeviceFound {
                address,
                address_type,
                flags,
                rssi,
                eir_data,
                ..
            } => {
                let report1 = EIRData::from(eir_data.as_ref());
                let report2 = EIRData::from(eir_data.as_ref());
                let now = Utc::now();

                if let Ok(Tilt { color, fahrenheit, gravity, .. } ) = Tilt::try_from(report1) {
                    let centi_celsius = ((i32::from(fahrenheit) - 32) * 500) / 9;
                    println!("at={:?} which={:?} celsius={:?} gravity={:?}", now, color, centi_celsius, gravity);
                }
                else if let Ok(gf_info) = Grainfather::try_from(report2) {
                    println!("at={:?} grainfather={:?} address={} ({:?})", now, gf_info, address, address_type);

                    let gf = btle_central
                        .peripherals()
                        .into_iter()
                        .find(|p| p.address().address == address.as_ref())
                        .unwrap();

                    gf.connect().unwrap();

                    // discover characteristics
                    gf.discover_characteristics().unwrap();

                    for gf_char in gf.characteristics().iter() {
                        println!("\t{:?}", gf_char);
                    }
                }

                ()
            }
            Event::Discovering {
                discovering,
                address_type,
            } => {
                eprintln!("discovering: {} {:?}", discovering, address_type);

                // if discovery ended, turn it back on
                if !discovering {
                    bluez_client
                        .start_service_discovery(
                            bluez_controller,
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
                    bluez_controller, address, address_type, flags
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
