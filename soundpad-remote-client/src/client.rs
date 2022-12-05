use color_eyre::eyre::{self, eyre};
use core::time::Duration;
use derivative::Derivative;
use std::fmt::Debug;
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
#[derive(Debug)]
pub enum Error {
    Connection(io::Error),
    InvalidCommand(String),
    NotFound { missing: String },
    BadResponse { source: eyre::Report },
    Other(eyre::Report),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "Connection error: {}", e),
            Self::InvalidCommand(s) => write!(f, "Soundpad cannot handle this command: {}", s),
            Self::NotFound { missing } => write!(f, "Not found: {}", missing),
            Self::BadResponse { source: e } => write!(f, "Bad response: {}", e),
            Self::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::InvalidCommand(_) => None,
            Self::NotFound { .. } => None,
            Self::BadResponse { source: e } => e.source(),
            Self::Other(e) => e.source(),
        }
    }
}

impl From<eyre::Report> for Error {
    fn from(e: eyre::Report) -> Self {
        Self::Other(e)
    }
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
            Err(e) => Err(Error::BadResponse {
                source: eyre::Error::from(e).wrap_err("Could not deserialize XML"),
            }),
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
                CommandNotFound(_) | BadRequest => {
                    panic!("{msg} is not a valid command. If you encounter this, please file a 🐞.")
                }
                NoContent => Error::NotFound {
                    missing: format!("{} at index {}", sound.title, sound.index),
                },
                NotFound(s) | Unknown(s) => eyre!("Soundpad says: {s}").into(),
            }),
        }
    }
}
