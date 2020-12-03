use crate::{GrainfatherClient, BtleplugGrainfatherClientImpl, };

use std::{
    error::Error,
    sync::mpsc::{Sender},
    convert::{TryFrom}
};

use bm_bluetooth::*;
use bm_grainfather::bluetooth::*;
use bm_tilt::*;

use async_std::task::block_on;
use bluez::{
    client::*,
    interface::{controller::*, event::Event},
};
use btleplug::{
    api::{Central, Peripheral},
};

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::ConnectedAdapter as CentralImpl, manager::Manager};

#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter as CentralImpl, manager::Manager};

#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter as CentralImpl, manager::Manager};

pub enum BluetoothDiscoveryEvent
{
    DiscoveredTilt(Tilt),
    DiscoveredGrainfather(GrainfatherClient),
}

pub struct BluetoothDiscovery<'z>
{
    sender: Sender<BluetoothDiscoveryEvent>,
    bluez_client: BlueZClient<'z>,
    bluez_controller: Controller,
    btle_central: CentralImpl
}

impl<'z> BluetoothDiscovery<'z>
{
    pub async fn run(sender: Sender<BluetoothDiscoveryEvent>) -> Result<(), Box<dyn Error>> {
        let mut bluez_client = BlueZClient::new().unwrap();

        let bluez_controllers = bluez_client.get_controller_list().await.unwrap();

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
        let btle_adapter =
            btle_adapters.into_iter().filter(|adapter| adapter.addr.address == bluez_info.address.as_ref()).nth(0).unwrap();
        let btle_central = btle_adapter.connect().unwrap();

        let state = Self {
            sender,
            bluez_client,
            bluez_controller,
            btle_central,
        };

        state.run_prime().await
    }

    async fn run_prime(mut self) -> Result<(), Box<dyn Error>> {
        self.start_discovery().await?;

        loop {
            let response = self.bluez_client.process().await?;

            match response.event {
                Event::Discovering {
                    discovering,
                    address_type: _,
                } => {
                    // eprintln!("discovering: {} {:?}", discovering, address_type);

                    // if discovery ended, turn it back on
                    if !discovering {
                        self.start_discovery().await?
                    }
                }

                Event::DeviceFound {
                    address,
                    // address_type,
                    // flags,
                    // rssi,
                    eir_data,
                    ..
                } => {
                    let report1 = EIRData::from(eir_data.as_ref());
                    let report2 = EIRData::from(eir_data.as_ref());

                    if let Ok(tilt) = Tilt::try_from(report1) {
                        self.sender.send(BluetoothDiscoveryEvent::DiscoveredTilt(tilt)).unwrap();
                    }
                    else if let Ok(_gf_info) = Grainfather::try_from(report2) {
                        let gf_peripheral = self.btle_central
                            .peripherals()
                            .into_iter()
                            .find(|p| p.address().address == address.as_ref())
                            .unwrap();

                        let gf = BtleplugGrainfatherClientImpl::new(gf_peripheral);
                        let gf = GrainfatherClient::try_from(Box::new(gf)).unwrap();

                        self.sender.send(BluetoothDiscoveryEvent::DiscoveredGrainfather(gf)).unwrap();
                    }
                }
                other => {
                    eprintln!("got: {:?}", other);
                }
            }
        }
    }

    async fn start_discovery(&mut self) -> Result<(), Box<dyn Error>> {
        const TX_LEVEL: i8 = 127;

        self.bluez_client
            .start_service_discovery(
                self.bluez_controller,
                AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
                TX_LEVEL,
                vec![],
                )
            .await?;

        Ok(())
    }
}
