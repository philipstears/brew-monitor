use bm_grainfather::{
    btleplug::Client as GrainfatherClient, notifications::*, Command, InteractionCode, Notification, Recipe, StepNumber,
};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex, MutexGuard,
};

#[derive(Clone)]
pub struct GrainfatherManager(Arc<Mutex<GrainfatherInternal>>);

impl GrainfatherManager {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(GrainfatherInternal::new())))
    }

    pub fn set_client(&self, client: GrainfatherClient) {
        self.lock().set_client(client);
    }

    pub fn command(&self, command: &Command) -> Result<(), btleplug::Error> {
        self.lock().command(command)
    }

    pub fn send_recipe(&self, recipe: &Recipe) -> Result<(), btleplug::Error> {
        self.lock().send_recipe(recipe)
    }

    pub fn subscribe(&mut self) -> Receiver<Notification> {
        self.lock().subscribe()
    }

    fn lock(&self) -> MutexGuard<GrainfatherInternal> {
        self.0.lock().expect("The grainfather manager lock has been poisoned")
    }
}

struct GrainfatherInternal {
    client: Option<GrainfatherClient>,
    subscribers: Arc<Mutex<Vec<Sender<Notification>>>>,
    state: Arc<Mutex<State>>,
}

impl GrainfatherInternal {
    const INITIAL_HANDLER_CAPACITY: usize = 16;

    fn new() -> Self {
        Self {
            client: None,
            subscribers: Arc::new(Mutex::new(Vec::with_capacity(Self::INITIAL_HANDLER_CAPACITY))),
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    fn set_client(&mut self, client: GrainfatherClient) {
        let have_valid_client = self.client.as_ref().map(|client| client.is_connected()).unwrap_or(false);

        if !have_valid_client {
            println!("Setting grainfather");

            let subscribers = self.subscribers.clone();
            let state = self.state.clone();

            client
                .subscribe(Box::new(move |notification| {
                    subscribers.lock().unwrap().retain(|subscriber| {
                        let keep_subscriber = subscriber.send(notification.clone()).is_ok();
                        keep_subscriber
                    });

                    state.lock().unwrap().update(notification);
                }))
                .unwrap();

            self.client = Some(client);
        }
    }

    pub fn command(&mut self, command: &Command) -> Result<(), btleplug::Error> {
        let client = self.client.as_ref().ok_or(btleplug::Error::NotConnected)?;
        client.command(command)
    }

    pub fn send_recipe(&mut self, recipe: &Recipe) -> Result<(), btleplug::Error> {
        let client = self.client.as_ref().ok_or(btleplug::Error::NotConnected)?;
        client.send_recipe(recipe)
    }

    pub fn subscribe(&mut self) -> Receiver<Notification> {
        let (sender, receiver) = mpsc::channel();
        self.subscribers.lock().unwrap().push(sender);
        receiver
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

impl State {
    fn update(&mut self, notification: Notification) {
        match notification {
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
                maybe_update("heat_active", &mut self.heat_active, &heat_active);
                maybe_update("pump_active", &mut self.pump_active, &pump_active);
                maybe_update("auto_mode_active", &mut self.auto_mode_active, &auto_mode_active);
                maybe_update("step_ramp_active", &mut self.step_ramp_active, &step_ramp_active);
                maybe_update("interaction_mode_active", &mut self.interaction_mode_active, &interaction_mode_active);
                maybe_update("interaction_code", &mut self.interaction_code, &interaction_code);
                maybe_update("step_number", &mut self.step_number, &step_number);
                maybe_update("delayed_heat_mode_active", &mut self.delayed_heat_mode_active, &delayed_heat_mode_active);
            }
            Notification::Status2(Status2 {
                heat_power_output_percentage,
                timer_paused,
                step_mash_mode,
                recipe_interrupted,
                manual_power_mode,
                sparge_water_alert_displayed,
            }) => {
                maybe_update(
                    "heat_power_output_percentage",
                    &mut self.heat_power_output_percentage,
                    &heat_power_output_percentage,
                );
                maybe_update("timer_paused", &mut self.timer_paused, &timer_paused);
                maybe_update("step_mash_mode", &mut self.step_mash_mode, &step_mash_mode);
                maybe_update("recipe_interrupted", &mut self.recipe_interrupted, &recipe_interrupted);
                maybe_update("manual_power_mode", &mut self.manual_power_mode, &manual_power_mode);
                maybe_update(
                    "sparge_water_alert_displayed",
                    &mut self.sparge_water_alert_displayed,
                    &sparge_water_alert_displayed,
                );
            }
            Notification::Temp(Temp {
                desired,
                current,
            }) => {
                // These frequently fluctuate by 0.1 of a degree, which is
                // annoying to report, so don't
                self.temp_desired = (desired * 100.0) as i32;
                self.temp_current = (current * 100.0) as i32;
            }
            Notification::DelayedHeatTimer(Timer {
                active,
                ..
            }) => {
                maybe_update("timer_active", &mut self.timer_active, &active);
            }
            Notification::Interaction(Interaction {
                interaction_code,
            }) => {
                println!("[R]: interaction with code {:?}", interaction_code);
            }
            other => {
                println!("[R]: {:?}", other);
            }
        }
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
