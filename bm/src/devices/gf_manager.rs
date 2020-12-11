use bm_grainfather::{btleplug::Client as GrainfatherClient, Command, Notification, Recipe};
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
}

impl GrainfatherInternal {
    const INITIAL_HANDLER_CAPACITY: usize = 16;

    fn new() -> Self {
        Self {
            client: None,
            subscribers: Arc::new(Mutex::new(Vec::with_capacity(Self::INITIAL_HANDLER_CAPACITY))),
        }
    }

    fn set_client(&mut self, client: GrainfatherClient) {
        let have_valid_client = self.client.as_ref().map(|client| client.is_connected()).unwrap_or(false);

        if !have_valid_client {
            println!("Setting grainfather");

            let subscribers = self.subscribers.clone();

            client
                .subscribe(Box::new(move |notification| {
                    subscribers.lock().unwrap().retain(|subscriber| {
                        let keep_subscriber = subscriber.send(notification.clone()).is_ok();
                        keep_subscriber
                    });
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
