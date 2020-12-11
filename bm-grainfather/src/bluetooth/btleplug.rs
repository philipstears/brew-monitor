//! Provides a client to make it easy to work with a Grainfather controller.

use crate::bluetooth::*;
use crate::{notifications::*, Command, InteractionCode, Notification, Recipe, StepNumber};

use ::btleplug::{
    api::{Characteristic, NotificationHandler as BtlePlugNotificationHandler, Peripheral, UUID},
    Error, Result as BtlePlugResult,
};

use std::{
    convert::TryFrom,
    sync::{Arc, RwLock},
};

pub type NotificationHandler = Box<dyn FnMut(Notification) + Send>;

/// Possible errors encountered during the construction of a client.
#[derive(Debug)]
pub enum ClientError {
    /// The peripheral isn't connected, and failed to connect.
    Connect(Error),
    /// The characteristics of the peripheral could not be discovered.
    DiscoverCharacteristics(Error),
    /// The write characteristic (used for issuing commands) could not be found.
    WriteCharacteristic,
    /// The read characteristic (used for receiving notifications) could not be found.
    ReadCharacteristic,
}

trait ClientImpl: Send + Sync + std::fmt::Debug {
    fn is_connected(&self) -> bool;
    fn connect(&self) -> BtlePlugResult<()>;
    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> BtlePlugResult<()>;
    fn discover_characteristics(&self) -> BtlePlugResult<Vec<Characteristic>>;
    fn on_notification(&self, handler: BtlePlugNotificationHandler);
    fn subscribe(&self, characteristic: &Characteristic) -> BtlePlugResult<()>;
}

#[derive(Debug)]
struct BtleplugClientImpl<P>
where
    P: Peripheral,
{
    p: P,
}

impl<P> BtleplugClientImpl<P>
where
    P: Peripheral,
{
    pub fn new(peripheral: P) -> Self {
        Self {
            p: peripheral,
        }
    }
}

impl<P> ClientImpl for BtleplugClientImpl<P>
where
    P: Peripheral,
{
    fn is_connected(&self) -> bool {
        self.p.is_connected()
    }

    fn connect(&self) -> BtlePlugResult<()> {
        self.p.connect()
    }

    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> BtlePlugResult<()> {
        self.p.command(characteristic, data)
    }

    fn discover_characteristics(&self) -> BtlePlugResult<Vec<Characteristic>> {
        self.p.discover_characteristics()
    }

    fn on_notification(&self, handler: BtlePlugNotificationHandler) {
        self.p.on_notification(handler)
    }

    fn subscribe(&self, characteristic: &Characteristic) -> BtlePlugResult<()> {
        self.p.subscribe(characteristic)
    }
}

#[derive(Debug, Default)]
struct State {
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
pub struct Client {
    gf: Box<dyn ClientImpl>,
    read: Characteristic,
    write: Characteristic,
    state: Arc<RwLock<State>>,
}

impl Client {
    /// Tries to construct a client from the given [`btleplug::api::Peripheral`](::btleplug::api::Peripheral). This
    /// will connect the peripheral if it isn't already connected.
    ///
    /// This will fail if the peripheral isn't connected and can't be connected, and if the
    /// peripheral doesn't support the necessary read/write characteristics.
    pub fn try_from<P>(peripheral: P) -> Result<Self, ClientError>
    where
        P: Peripheral + 'static,
    {
        let gf = Box::new(BtleplugClientImpl::new(peripheral));

        if !gf.is_connected() {
            gf.connect().map_err(ClientError::Connect)?
        }

        let cs = gf.discover_characteristics().map_err(ClientError::DiscoverCharacteristics)?;

        let rc_id = UUID::B128(CHARACTERISTIC_ID_READ.to_le_bytes());
        let rc = cs.iter().find(|c| c.uuid == rc_id).ok_or(ClientError::ReadCharacteristic)?;

        let wc_id = UUID::B128(CHARACTERISTIC_ID_WRITE.to_le_bytes());
        let wc = cs.iter().find(|c| c.uuid == wc_id).ok_or(ClientError::WriteCharacteristic)?;

        let state = Arc::new(RwLock::new(State::default()));

        let result = Self {
            gf,
            read: rc.clone(),
            write: wc.clone(),
            state: state.clone(),
        };

        result
            .subscribe(Box::new(move |notification| match notification {
                Notification::Status1(Status1 {
                    heat_active,
                    pump_active,
                    auto_mode_active,
                    step_ramp_active,
                    interaction_mode_active,
                    interaction_code,
                    step_number,
                    delayed_heat_mode_active,
                }) => {
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
                Notification::Status2(Status2 {
                    heat_power_output_percentage,
                    timer_paused,
                    step_mash_mode,
                    recipe_interrupted,
                    manual_power_mode,
                    sparge_water_alert_displayed,
                }) => {
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
                Notification::Temp(Temp {
                    desired,
                    current,
                }) => {
                    let mut state = state.write().unwrap();

                    // These frequently fluctuate by 0.1 of a degree, which is
                    // annoying to report, so don't
                    state.temp_desired = (desired * 100.0) as i32;
                    state.temp_current = (current * 100.0) as i32;
                }
                Notification::DelayedHeatTimer(Timer {
                    active,
                    ..
                }) => {
                    let mut state = state.write().unwrap();
                    maybe_update("timer_active", &mut state.timer_active, &active);
                }
                Notification::Interaction(Interaction {
                    interaction_code,
                }) => {
                    println!("[R]: interaction with code {:?}", interaction_code);
                }
                other => {
                    println!("[R]: {:?}", other);
                }
            }))
            .unwrap();

        Ok(result)
    }

    /// Determines whether the client is connected.
    pub fn is_connected(&self) -> bool {
        self.gf.is_connected()
    }

    /// Despatches a command to the grainfather controller.
    pub fn command(&self, command: &Command) -> Result<(), Error> {
        println!("[S]: {:?}", command);
        self.gf.command(&self.write, command.to_vec().as_ref())
    }

    /// Sends a recipe to the to the grainfather controller.
    pub fn send_recipe(&self, recipe: &Recipe) -> Result<(), Error> {
        println!("[S]: Recipe with name {}", recipe.name);

        for command in recipe.to_commands().iter() {
            self.gf.command(&self.write, command.as_ref())?
        }

        Ok(())
    }

    /// Subscribes to notifications issued by the grainfather controller.
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
                let notification = Notification::try_from(notification).unwrap();
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
