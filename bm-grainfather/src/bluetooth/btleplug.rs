//! Provides a client to make it easy to work with a Grainfather controller.

use crate::bluetooth::*;
use crate::{Command, Notification, Recipe};

use ::btleplug::{
    api::{Characteristic, NotificationHandler as BtlePlugNotificationHandler, Peripheral, UUID},
    Error, Result as BtlePlugResult,
};

use std::convert::TryFrom;

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

#[derive(Debug)]
pub struct Client {
    gf: Box<dyn ClientImpl>,
    read: Characteristic,
    write: Characteristic,
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

        let result = Self {
            gf,
            read: rc.clone(),
            write: wc.clone(),
        };

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

        self.gf.command(&self.write, &Command::SetLocalBoilTemperature(recipe.boil_temperature).to_vec())?;

        // Otherwise the controller becomes unresponsive, there's got to be
        // a better approach than this though.
        std::thread::sleep(std::time::Duration::from_millis(32));

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
