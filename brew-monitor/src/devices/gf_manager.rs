use bm_grainfather::{
    btleplug::Client as GrainfatherClient, notifications::*, Command, InteractionCode, Notification, Recipe, StepNumber,
};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex, MutexGuard,
};

#[derive(Clone, Debug)]
pub enum ManagerOrClientNotification {
    ClientNotification(Notification),
    ManagerNotification(ManagerNotification),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ManagerNotification {
    BoilAlertState(BoilAlertState),
    HeatSpargeWaterAlertState(HeatSpargeWaterAlertState),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BoilAlertState {
    pub visible: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HeatSpargeWaterAlertState {
    pub visible: bool,
}

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

    pub fn subscribe(&mut self) -> Receiver<ManagerOrClientNotification> {
        self.lock().subscribe()
    }

    fn lock(&self) -> MutexGuard<GrainfatherInternal> {
        self.0.lock().expect("The grainfather manager lock has been poisoned")
    }
}

struct GrainfatherInternal {
    client: Option<GrainfatherClient>,
    subscribers: Arc<Mutex<Vec<Sender<ManagerOrClientNotification>>>>,
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
                    send_notification_to_subscribers(
                        subscribers.as_ref(),
                        &ManagerOrClientNotification::ClientNotification(notification.clone()),
                    );

                    // Sometimes this will generate an synthetic notification, e.g.
                    // for boil additions
                    if let Some(synthetic_notifications) = state.lock().unwrap().handle_notification(&notification) {
                        for synthetic_notification in synthetic_notifications.iter() {
                            send_notification_to_subscribers(subscribers.as_ref(), &synthetic_notification);
                        }
                    }
                }))
                .unwrap();

            self.client = Some(client);
        }
    }

    pub fn command(&mut self, command: &Command) -> Result<(), btleplug::Error> {
        let client = self.client.as_ref().ok_or(btleplug::Error::NotConnected)?;
        let result = client.command(command);

        if let Ok(()) = result {
            self.state.lock().unwrap().handle_command(&command);
        }

        result
    }

    pub fn send_recipe(&mut self, recipe: &Recipe) -> Result<(), btleplug::Error> {
        let client = self.client.as_ref().ok_or(btleplug::Error::NotConnected)?;
        client.send_recipe(recipe)
    }

    pub fn subscribe(&mut self) -> Receiver<ManagerOrClientNotification> {
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
    // Boil Alert Visible
    boil_alert_active: bool,
    // Heat Sparge Water Alert Visible
    sparge_water_alert_active: bool,
}

impl State {
    fn handle_command(&mut self, command: &Command) -> Option<ManagerOrClientNotification> {
        let maybe_dismiss_alert = match command {
            Command::DismissAlert => true,
            Command::PressSet => true,
            _ => false,
        };

        if maybe_dismiss_alert {
            if self.sparge_water_alert_active {
                return Some(self.update_sparge_water_alert_status(false));
            }

            if self.boil_alert_active {
                return Some(self.update_boil_alert_status(false));
            }
        }

        None
    }

    fn update_boil_alert_status(&mut self, boil_alert_active: bool) -> ManagerOrClientNotification {
        maybe_update("boil_alert_visible", &mut self.boil_alert_active, &boil_alert_active);
        self.build_boil_status()
    }

    fn update_sparge_water_alert_status(&mut self, sparge_water_alert_active: bool) -> ManagerOrClientNotification {
        maybe_update("sparge_water_alert_visible", &mut self.sparge_water_alert_active, &sparge_water_alert_active);
        self.build_sparge_water_status()
    }

    fn handle_notification(&mut self, notification: &Notification) -> Option<Vec<ManagerOrClientNotification>> {
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

                // Send out boil addition alerts with each status alert
                // TODO: if we stored the recipe, we could work out whether we were in the boil,
                // and only send them then
                return Some(vec![self.build_boil_status(), self.build_sparge_water_status()]);
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

            Notification::Timer(Timer {
                active,
                ..
            }) => {
                maybe_update("timer_active", &mut self.timer_active, &active);
            }

            Notification::Interaction(Interaction {
                interaction_code,
            }) => {
                println!("[R]: interaction with code {:?}", interaction_code);

                if let InteractionCode::Dismiss = interaction_code {
                    if self.boil_alert_active {
                        return Some(vec![self.update_boil_alert_status(false)]);
                    }

                    if self.sparge_water_alert_active {
                        return Some(vec![self.update_sparge_water_alert_status(false)]);
                    }
                }
            }

            Notification::PromptBoilAddition(PromptBoilAddition) => {
                return Some(vec![self.update_boil_alert_status(true)]);
            }

            Notification::PromptSpargeWater(PromptSpargeWater) => {
                return Some(vec![self.update_sparge_water_alert_status(true)]);
            }

            other => {
                println!("[R]: {:?}", other);
            }
        }

        None
    }

    fn build_boil_status(&self) -> ManagerOrClientNotification {
        ManagerOrClientNotification::ManagerNotification(ManagerNotification::BoilAlertState(BoilAlertState {
            visible: self.boil_alert_active,
        }))
    }

    fn build_sparge_water_status(&self) -> ManagerOrClientNotification {
        ManagerOrClientNotification::ManagerNotification(ManagerNotification::HeatSpargeWaterAlertState(
            HeatSpargeWaterAlertState {
                visible: self.sparge_water_alert_active,
            },
        ))
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

fn send_notification_to_subscribers(
    subscribers: &Mutex<Vec<Sender<ManagerOrClientNotification>>>,
    notification: &ManagerOrClientNotification,
) {
    subscribers.lock().unwrap().retain(|subscriber| {
        let keep_subscriber = subscriber.send(notification.clone()).is_ok();
        keep_subscriber
    });
}
