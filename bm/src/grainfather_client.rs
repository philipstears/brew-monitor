use bm_grainfather::*;

use btleplug::api::{Characteristic, Peripheral, UUID};
use btleplug::Error;

use std::convert::TryFrom;

type NotificationHandler = Box<dyn FnMut(GrainfatherNotification) + Send>;

#[derive(Debug)]
pub enum GrainfatherClientError {
    Connect(Error),
    DiscoverCharacteristics(Error),
    WriteCharacteristic,
    ReadCharacteristic,
}

pub struct GrainfatherClient {
    gf: Peripheral,
    read: Characteristic,
    write: Characteristic,
}

impl GrainfatherClient {
    pub fn try_from(gf: Peripheral) -> Result<Self, GrainfatherClientError> {
        if !gf.is_connected() {
            gf.connect().map_err(GrainfatherClientError::Connect)?
        }

        let cs = gf.discover_characteristics().map_err(GrainfatherClientError::DiscoverCharacteristics)?;

        let rc_id = UUID::B128(CHARACTERISTIC_ID_READ.to_le_bytes());
        let rc = cs.iter().find(|c| c.uuid == rc_id).ok_or(GrainfatherClientError::ReadCharacteristic)?;

        let wc_id = UUID::B128(CHARACTERISTIC_ID_WRITE.to_le_bytes());
        let wc = cs.iter().find(|c| c.uuid == wc_id).ok_or(GrainfatherClientError::WriteCharacteristic)?;

        Ok(Self {
            gf,
            read: rc.clone(),
            write: wc.clone(),
        })
    }

    pub fn command(&self, command: &GrainfatherCommand) -> Result<(), Error> {
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
