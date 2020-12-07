use bm_grainfather::{btleplug::Client as GrainfatherClient, Command, Notification, Recipe};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

#[derive(Debug)]
pub struct NotConnected;

#[derive(Clone)]
pub struct GrainfatherManager(Arc<RwLock<GrainfatherInternal>>);

impl GrainfatherManager {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(GrainfatherInternal::new())))
    }

    pub fn set_client(&self, client: GrainfatherClient) {
        self.write().set_client(client);
    }

    pub fn command(&self, command: &Command) -> Result<(), NotConnected> {
        self.write().command(command)
    }

    pub fn send_recipe(&self, recipe: &Recipe) -> Result<(), NotConnected> {
        self.write().send_recipe(recipe)
    }

    fn read(&self) -> RwLockReadGuard<GrainfatherInternal> {
        self.0.read().expect("The grainfather manager lock has been poisoned")
    }

    fn write(&self) -> RwLockWriteGuard<GrainfatherInternal> {
        self.0.write().expect("The grainfather manager lock has been poisoned")
    }
}

struct GrainfatherInternal {
    client: Option<GrainfatherClient>,
    handlers: Vec<Sender<Notification>>,
}

impl GrainfatherInternal {
    const INITIAL_HANDLER_CAPACITY: usize = 16;

    fn new() -> Self {
        Self {
            client: None,
            handlers: Vec::with_capacity(Self::INITIAL_HANDLER_CAPACITY),
        }
    }

    fn set_client(&mut self, client: GrainfatherClient) {
        client.subscribe(Box::new(|notification| {
            //
        }));

        self.client = Some(client);
    }

    pub fn command(&mut self, command: &Command) -> Result<(), NotConnected> {
        let client = self.client.as_ref().ok_or(NotConnected)?;
        client.command(command).unwrap();
        Ok(())
    }

    pub fn send_recipe(&mut self, recipe: &Recipe) -> Result<(), NotConnected> {
        let client = self.client.as_ref().ok_or(NotConnected)?;
        client.send_recipe(recipe).unwrap();
        Ok(())
    }
}
