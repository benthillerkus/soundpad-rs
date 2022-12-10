use std::{borrow::Cow, str::FromStr};

use core::time::Duration;
use tokio::{io, sync::oneshot};
use tracing::{error, info, instrument, warn};

use super::{Client, Connection};
use crate::error::{eyre, Context, Error};

#[derive(Debug)]
pub struct Command {
    pub message: Cow<'static, str>,
    pub callback: Option<oneshot::Sender<io::Result<String>>>,
    pub cooldown: Duration,
}

impl Command {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            callback: None,
            cooldown: Duration::default(),
        }
    }

    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }

    #[instrument]
    pub(crate) async fn do_work(self, connection: &mut Connection) {
        let response = connection.send_and_receive(self.message.as_bytes()).await;

        if let Some(callback) = self.callback {
            if callback.send(response).is_err() {
                error!("Client is no longer listening for responses");
            }
        } else {
            warn!("No callback provided for command");
            match response {
                Ok(s) => info!("Command succeeded - Response: {s}"),
                Err(e) => error!("Command failed: {:?}", e),
            }
        }
    }

    #[instrument]
    pub async fn issue<R, T>(mut self, client: &Client) -> Result<R, Error>
    where
        R: FromStr<Err = T>,
        T: std::error::Error + Send + Sync + 'static,
    {
        let (respond_to, rx) = oneshot::channel();
        self.callback = Some(respond_to);
        client.tx.send(self).wrap_err(eyre!(
            "Couldn't submit Command, the actor was probably dropped"
        ))?;
        let response = rx.await.wrap_err(eyre!(
            "Couldn't receive a response, the actor was probably dropped"
        ))??;
        Ok(R::from_str(&response).wrap_err("Couldn't convert response")?)
    }
}
