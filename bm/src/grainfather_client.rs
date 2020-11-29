use bm_grainfather::*;

use btleplug::api::{Characteristic, Peripheral, UUID};
use btleplug::Error;

use std::{
    convert::TryFrom,
    sync::{Arc, RwLock},
};

type NotificationHandler = Box<dyn FnMut(GrainfatherNotification) + Send>;

#[derive(Debug)]
pub enum GrainfatherClientError {
    Connect(Error),
    DiscoverCharacteristics(Error),
    WriteCharacteristic,
    ReadCharacteristic,
}

pub trait GrainfatherClientImpl: Send + Sync + std::fmt::Debug {
    fn is_connected(&self) -> bool;
    fn connect(&self) -> btleplug::Result<()>;
    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> btleplug::Result<()>;
    fn discover_characteristics(&self) -> btleplug::Result<Vec<Characteristic>>;
    fn on_notification(&self, handler: btleplug::api::NotificationHandler);
    fn subscribe(&self, characteristic: &Characteristic) -> btleplug::Result<()>;
}

#[derive(Debug)]
pub struct BtleplugGrainfatherClientImpl<P>
where
    P: Peripheral,
{
    p: P,
}

impl<P> BtleplugGrainfatherClientImpl<P>
where
    P: Peripheral,
{
    pub fn new(peripheral: P) -> Self {
        Self {
            p: peripheral,
        }
    }
}

impl<P> GrainfatherClientImpl for BtleplugGrainfatherClientImpl<P>
where
    P: Peripheral,
{
    fn is_connected(&self) -> bool {
        self.p.is_connected()
    }

    fn connect(&self) -> btleplug::Result<()> {
        self.p.connect()
    }

    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> btleplug::Result<()> {
        self.p.command(characteristic, data)
    }

    fn discover_characteristics(&self) -> btleplug::Result<Vec<Characteristic>> {
        self.p.discover_characteristics()
    }

    fn on_notification(&self, handler: btleplug::api::NotificationHandler) {
        self.p.on_notification(handler)
    }

    fn subscribe(&self, characteristic: &Characteristic) -> btleplug::Result<()> {
        self.p.subscribe(characteristic)
    }
}

#[derive(Debug, Default)]
struct GrainfatherState {
    interaction_code: InteractionCode,
    step_number: StepNumber,
}

#[derive(Debug)]
pub struct GrainfatherClient {
    gf: Box<dyn GrainfatherClientImpl>,
    read: Characteristic,
    write: Characteristic,
    state: Arc<RwLock<GrainfatherState>>,
}

impl GrainfatherClient {
    pub fn try_from(gf: Box<dyn GrainfatherClientImpl>) -> Result<Self, GrainfatherClientError> {
        if !gf.is_connected() {
            gf.connect().map_err(GrainfatherClientError::Connect)?
        }

        let cs = gf.discover_characteristics().map_err(GrainfatherClientError::DiscoverCharacteristics)?;

        let rc_id = UUID::B128(CHARACTERISTIC_ID_READ.to_le_bytes());
        let rc = cs.iter().find(|c| c.uuid == rc_id).ok_or(GrainfatherClientError::ReadCharacteristic)?;

        let wc_id = UUID::B128(CHARACTERISTIC_ID_WRITE.to_le_bytes());
        let wc = cs.iter().find(|c| c.uuid == wc_id).ok_or(GrainfatherClientError::WriteCharacteristic)?;

        let state = Arc::new(RwLock::new(GrainfatherState::default()));

        let result = Self {
            gf,
            read: rc.clone(),
            write: wc.clone(),
            state: state.clone(),
        };

        result
            .subscribe(Box::new(move |notification| match notification {
                GrainfatherNotification::Status1 {
                    interaction_code,
                    step_number,
                    ..
                } => {
                    let mut state = state.write().unwrap();

                    if state.interaction_code != interaction_code {
                        println!("Interaction code changed from {} to {}", state.interaction_code, interaction_code);
                        state.interaction_code = interaction_code;
                    }

                    if state.step_number != step_number {
                        println!("Step number changed from {} to {}", state.step_number, step_number);
                        state.step_number = step_number;
                    }
                }
                GrainfatherNotification::Status2 {
                    ..
                } => {}
                GrainfatherNotification::Temp {
                    ..
                } => {}
                GrainfatherNotification::DelayedHeatTimer {
                    ..
                } => {}
                GrainfatherNotification::Interaction {
                    interaction_code,
                } => {
                    println!("Received interaction with code {}", interaction_code);
                }
                GrainfatherNotification::VoltageAndUnits {
                    ..
                } => {}
                GrainfatherNotification::Boil {
                    ..
                } => {}
                GrainfatherNotification::FirmwareVersion {
                    ..
                } => {}
                GrainfatherNotification::Other {
                    ..
                } => {}
            }))
            .unwrap();

        Ok(result)
    }

    pub fn command(&self, command: &GrainfatherCommand) -> Result<(), Error> {
        println!("Sending command {:?}", command);
        self.gf.command(&self.write, command.to_vec().as_ref())
    }

    pub fn send_recipe(&self, recipe: &Recipe) -> Result<(), Error> {
        for command in recipe.to_commands().iter() {
            self.gf.command(&self.write, command.as_ref())?
        }

        Ok(())
    }

    pub fn subscribe(&self, mut handler: NotificationHandler) -> Result<(), Error> {
        const NOTIFICATION_LEN: usize = 17;
        const NOTIFICATION_BUF_COUNT: usize = NOTIFICATION_LEN * 8;
        let mut gf_notification_buf = Vec::<u8>::with_capacity(NOTIFICATION_BUF_COUNT);

        self.gf.on_notification(Box::new(move |mut value_notification| {
            gf_notification_buf.append(&mut value_notification.value);

            let notification_count = gf_notification_buf.len() / NOTIFICATION_LEN;
            let notifications_len = notification_count * NOTIFICATION_LEN;

            for notification in gf_notification_buf.drain(..notifications_len).as_slice().chunks_exact(NOTIFICATION_LEN)
            {
                let notification = GrainfatherNotification::try_from(notification).unwrap();
                handler(notification);
            }
        }));

        self.gf.subscribe(&self.read)
    }
}
