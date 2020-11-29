use async_std::task::block_on;
use chrono::prelude::*;
use warp::{http, Filter, Rejection, Reply};

use std::{collections::HashMap, sync::mpsc, thread, time::Duration};

use bm_grainfather::*;
use bm_tilt::*;

use crate::*;

pub struct DeviceInfo<T> {
    when: DateTime<Utc>,
    device: T,
}

impl<T> DeviceInfo<T> {
    pub fn new(when: DateTime<Utc>, device: T) -> Self {
        Self {
            when,
            device,
        }
    }
}

pub struct Server {
    tilts: HashMap<TiltColor, DeviceInfo<Tilt>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            tilts: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        let (discovery_sender, discovery_receiver) = mpsc::channel();

        let _ = thread::Builder::new().name("ble-discovery".into()).spawn(move || {
            let discovery_future = BluetoothDiscovery::run(discovery_sender);
            block_on(discovery_future).unwrap();
        });

        loop {
            match discovery_receiver.recv().unwrap() {
                BluetoothDiscoveryEvent::DiscoveredTilt(tilt) => {
                    let now = Utc::now();
                    let centi_celsius = ((i32::from(tilt.fahrenheit) - 32) * 500) / 9;
                    println!(
                        "at={:?} which={:?} celsius={:?} gravity={:?}",
                        now, tilt.color, centi_celsius, tilt.gravity
                    );

                    self.tilts.insert(tilt.color, DeviceInfo::new(now, tilt));
                }

                BluetoothDiscoveryEvent::DiscoveredGrainfather(gf) => {
                    let now = Utc::now();
                    println!("at={:?} which=grainfather", now);

                    println!("Reset");
                    gf.command(&GrainfatherCommand::Reset).unwrap();

                    std::thread::sleep(Duration::from_millis(5000));

                    gf.subscribe(Box::new(move |notification| {
                        println!("\treceived {:?}", notification);
                    }))
                    .unwrap();

                    println!("Requesting Firmware Version");

                    gf.command(&GrainfatherCommand::GetFirmwareVersion).unwrap();
                    std::thread::sleep(Duration::from_millis(100));

                    println!("Requesting Voltage and Units");
                    gf.command(&GrainfatherCommand::GetVoltageAndUnits).unwrap();
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
                    recipe.mash_steps.push(MashStep {
                        temperature: 65,
                        minutes: 60,
                    });
                    recipe.mash_steps.push(MashStep {
                        temperature: 75,
                        minutes: 10,
                    });
                    recipe.boil_steps.push(60); // Hop addition 1
                    recipe.boil_steps.push(30); // Hop addition 2
                    recipe.boil_steps.push(5); // Yeast nutrient

                    gf.send_recipe(&recipe).unwrap();
                    println!("Recipe sent");

                    loop {
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                }
            }
        }
    }
}
