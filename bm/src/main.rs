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

                    println!("Connecting");
                    gf.connect().unwrap();

                    println!("Obtaining characteristics");
                    // discover characteristics
                    gf.discover_characteristics().unwrap();
                    let cs = gf.characteristics();
                    let rcid = btleplug::api::UUID::B128(CHARACTERISTIC_ID_READ.to_le_bytes());
                    let rc = cs.iter().find(|c| c.uuid == rcid).unwrap();
                    let wcid = btleplug::api::UUID::B128(CHARACTERISTIC_ID_WRITE.to_le_bytes());
                    let wc = cs.iter().find(|c| c.uuid == wcid).unwrap();

                    println!("Reset");
                    let cmd = GrainfatherCommand::Reset;
                    gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    std::thread::sleep(Duration::from_millis(5000));

                    {
                        const NOTIFICATION_LEN: usize = 17;
                        const NOTIFICATION_BUF_COUNT: usize = NOTIFICATION_LEN * 8;
                        let mut gf_notification_buf = Vec::<u8>::with_capacity(NOTIFICATION_BUF_COUNT);

                        gf.on_notification(Box::new(move |mut value_notification| {
                            gf_notification_buf.append(&mut value_notification.value);

                            let notification_count = gf_notification_buf.len() / NOTIFICATION_LEN;
                            let notifications_len = notification_count * NOTIFICATION_LEN;

                            for notification in gf_notification_buf.drain(..notifications_len).as_slice().chunks_exact(NOTIFICATION_LEN) {
                                let notification = GrainfatherNotification::try_from(notification).unwrap();
                                println!("\treceived {:?}", notification);
                            }
                        }));
                    }

                    gf.subscribe(rc).unwrap();

                    println!("Requesting Firmware Version");

                    let cmd = GrainfatherCommand::GetFirmwareVersion;
                    gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    std::thread::sleep(Duration::from_millis(100));

                    println!("Requesting Voltage and Units");
                    let cmd = GrainfatherCommand::GetVoltageAndUnits;
                    gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    std::thread::sleep(Duration::from_millis(5000));

                    // println!("Requesting Boil Temp");
                    // let cmd = GrainfatherCommand::GetBoilTemperature;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Set Local Boil temp");
                    // let cmd = GrainfatherCommand::SetLocalBoilTemperature(98.5);
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Requesting Boil Temp Again");
                    // let cmd = GrainfatherCommand::GetBoilTemperature;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Pump On");
                    // let cmd = GrainfatherCommand::SetPumpActive(true);
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Pump Off");
                    // let cmd = GrainfatherCommand::SetPumpActive(false);
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Delayed Heat");
                    // let cmd = GrainfatherCommand::EnableDelayedHeatTimer { minutes: 2, seconds: 0 };
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5_000));

                    // println!("Pause Timer");
                    // let cmd = GrainfatherCommand::PauseOrResumeActiveTimer;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5_000));

                    // println!("Resume Timer");
                    // let cmd = GrainfatherCommand::PauseOrResumeActiveTimer;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5_000));

                    // println!("Update Timer");
                    // let cmd = GrainfatherCommand::UpdateActiveTimer(Delay::MinutesSeconds(3, 30));
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5_000));

                    // println!("Cancel Delayed Heat");
                    // let cmd = GrainfatherCommand::CancelActiveTimer;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();

                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Increment Temp");
                    // let cmd = GrainfatherCommand::IncrementTargetTemperature;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();
                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Decrement Temp");
                    // let cmd = GrainfatherCommand::DecrementTargetTemperature;
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();
                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Set to 75");
                    // let cmd = GrainfatherCommand::SetTargetTemperature(75.0);
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();
                    // std::thread::sleep(Duration::from_millis(5000));

                    // println!("Set to 60");
                    // let cmd = GrainfatherCommand::SetTargetTemperature(60.0);
                    // gf.command(&wc, cmd.to_vec().as_ref()).unwrap();
                    // std::thread::sleep(Duration::from_millis(5000));

                    println!("Send recipe");
                    let mut recipe = Recipe::default();
                    recipe.name = "TEST".to_string();
                    recipe.delay = RecipeDelay::None; // RecipeDelay::MinutesSeconds(60, 0);
                    recipe.mash_steps.push(MashStep { temperature: 65, minutes: 60 });
                    recipe.mash_steps.push(MashStep { temperature: 75, minutes: 10 });
                    recipe.boil_steps.push(60); // Hop addition 1
                    recipe.boil_steps.push(30); // Hop addition 2
                    recipe.boil_steps.push(5);  // Yeast nutrient

                    for command in recipe.to_commands().iter() {
                        gf.command(&wc, command.as_ref()).unwrap();
                    }
                    println!("Recipe sent");

                    loop {
                        std::thread::sleep(Duration::from_millis(1000));
                    }

                    // for gf_char in gf.characteristics().iter() {
                    //     println!("\t{:?}", gf_char);
                    // }
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
