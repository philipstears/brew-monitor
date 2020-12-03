use bm_grainfather::bluetooth::*;
use bm_grainfather::proto::*;
use bm_grainfather::proto::command::*;
use bm_grainfather::proto::recipe::*;
use bm_grainfather::proto::notification::*;

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
    // Status 1
    heat_active: bool,
    pump_active: bool,
    auto_mode_active: bool,
    step_ramp_active: bool,
    interaction_mode_active: bool,
    interaction_code: InteractionCode,
    step_number: StepNumber,
    delayed_heat_mode_active: bool,
    // Status 2
    heat_power_output_percentage: u8,
    timer_paused: bool,
    step_mash_mode: bool,
    recipe_interrupted: bool,
    manual_power_mode: bool,
    sparge_water_alert_displayed: bool,
    // Temp
    temp_desired: i32,
    temp_current: i32,
    // Timer
    timer_active: bool,
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
                    heat_active,
                    pump_active,
                    auto_mode_active,
                    step_ramp_active,
                    interaction_mode_active,
                    interaction_code,
                    step_number,
                    delayed_heat_mode_active,
                } => {
                    let mut state = state.write().unwrap();

                    maybe_update("heat_active", &mut state.heat_active, &heat_active);
                    maybe_update("pump_active", &mut state.pump_active, &pump_active);
                    maybe_update("auto_mode_active", &mut state.auto_mode_active, &auto_mode_active);
                    maybe_update("step_ramp_active", &mut state.step_ramp_active, &step_ramp_active);
                    maybe_update(
                        "interaction_mode_active",
                        &mut state.interaction_mode_active,
                        &interaction_mode_active,
                    );
                    maybe_update("interaction_code", &mut state.interaction_code, &interaction_code);
                    maybe_update("step_number", &mut state.step_number, &step_number);
                    maybe_update(
                        "delayed_heat_mode_active",
                        &mut state.delayed_heat_mode_active,
                        &delayed_heat_mode_active,
                    );
                }
                GrainfatherNotification::Status2 {
                    heat_power_output_percentage,
                    timer_paused,
                    step_mash_mode,
                    recipe_interrupted,
                    manual_power_mode,
                    sparge_water_alert_displayed,
                } => {
                    let mut state = state.write().unwrap();

                    maybe_update(
                        "heat_power_output_percentage",
                        &mut state.heat_power_output_percentage,
                        &heat_power_output_percentage,
                    );
                    maybe_update("timer_paused", &mut state.timer_paused, &timer_paused);
                    maybe_update("step_mash_mode", &mut state.step_mash_mode, &step_mash_mode);
                    maybe_update("recipe_interrupted", &mut state.recipe_interrupted, &recipe_interrupted);
                    maybe_update("manual_power_mode", &mut state.manual_power_mode, &manual_power_mode);
                    maybe_update(
                        "sparge_water_alert_displayed",
                        &mut state.sparge_water_alert_displayed,
                        &sparge_water_alert_displayed,
                    );
                }
                GrainfatherNotification::Temp {
                    desired,
                    current,
                } => {
                    let mut state = state.write().unwrap();

                    // These frequently fluctuate by 0.1 of a degree, which is
                    // annoying to report, so don't
                    state.temp_desired = (desired * 100.0) as i32;
                    state.temp_current = (current * 100.0) as i32;
                }
                GrainfatherNotification::DelayedHeatTimer {
                    active,
                    ..
                } => {
                    let mut state = state.write().unwrap();
                    maybe_update("timer_active", &mut state.timer_active, &active);
                }
                GrainfatherNotification::Interaction {
                    interaction_code,
                } => {
                    println!("[R]: interaction with code {:?}", interaction_code);
                }
                other => {
                    println!("[R]: {:?}", other);
                }
            }))
            .unwrap();

        Ok(result)
    }

    pub fn command(&self, command: &GrainfatherCommand) -> Result<(), Error> {
        println!("[S]: {:?}", command);
        self.gf.command(&self.write, command.to_vec().as_ref())
    }

    pub fn send_recipe(&self, recipe: &Recipe) -> Result<(), Error> {
        println!("[S]: Recipe with name {}", recipe.name);

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

fn maybe_update<T>(field_name: &str, target: &mut T, new_value: &T)
where
    T: Eq + Clone + std::fmt::Debug,
{
    if target == new_value {
        return;
    }

    println!("[R]: {} changed from {:?} to {:?}", field_name, target, new_value);

    *target = new_value.clone();
}
