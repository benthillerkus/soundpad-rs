use core::time::Duration;
use derivative::Derivative;
use eyre::eyre;
use std::{borrow::Cow, fmt::Debug};
use thiserror::Error;
use tokio::{io, sync::mpsc};
use tracing::{info, instrument};

use crate::{
    response_code::{ErrorCode, ResponseCode, SuccessCode},
    soundlist::Sound,
    SoundList,
};

mod builder;
mod command;
mod connection;

pub use builder::ClientBuilder;
pub(crate) use command::Command;
pub(crate) use connection::Connection;

// FIXME: Not all of these can happen on each function
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Connection(#[from] io::Error),
    #[error("{0} is not a valid command")]
    InvalidCommand(String),
    #[error("{missing} does not exist")]
    NotFound { missing: String },
    #[error("Soundpad sent a bad response")]
    BadResponse(#[source] eyre::Report),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

impl From<command::Error> for Error {
    fn from(e: command::Error) -> Self {
        match e {
            command::Error::Connection(e) => Self::Connection(e),
            command::Error::Other(e) => Self::Other(e),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// A client for communicating with Soundpad
///
/// You can create a client using the [`ClientBuilder`].
///
/// # Examples
///
/// ```ignore
/// let client = ClientBuilder::new().connect()?;
/// let sounds = client.get_sound_list()?;
/// client.play_sound(&sounds[0])?;
/// ```
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Client {
    #[derivative(Debug = "ignore")]
    pub(crate) tx: mpsc::Sender<Command>,
    pub debounce: Duration,
}

impl Client {
    #[instrument]
    pub async fn custom_command(
        &self,
        command: impl ToString + Debug,
        cooldown: Duration,
    ) -> Result<ResponseCode> {
        match Command::new(command.to_string())
            .with_cooldown(cooldown)
            .issue(self)
            .await?
        {
            Ok(c) => Ok(Ok(c)),
            Err(e) => Ok(Err(e)),
        }
    }

    #[instrument]
    pub async fn get_sound_list(&self) -> Result<Vec<Sound>> {
        match Command::new("GetSoundList()").issue(self).await? {
            Ok(SoundList { sounds }) => Ok(sounds),
            Err(e) => Err(Error::BadResponse(
                eyre::Error::from(e).wrap_err("Could not deserialize XML"),
            )),
        }
    }

    #[instrument]
    pub async fn play_sound(&self, sound: &Sound) -> Result<()> {
        use ErrorCode::*;

        let msg = format!("DoPlaySound({})", sound.index);

        match Command::new(msg.clone())
            .with_cooldown(self.debounce)
            .issue(self)
            .await?
        {
            Ok(SuccessCode::Ok) => {
                info!("{}", sound.title);
                Ok(())
            }
            Err(e) => Err(match e {
                // FIXME: This cannot ever happen
                CommandNotFound(_) | BadRequest => Error::InvalidCommand(msg),
                NoContent => Error::NotFound {
                    missing: format!("{} at index {}", sound.title, sound.index),
                },
                NotFound(s) | Unknown(s) => eyre!("Soundpad says: {s}").into(),
            }),
        }
    }
}
