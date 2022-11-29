use derivative::Derivative;
use eyre::eyre;
use thiserror::Error;
use tokio::{io, sync::mpsc};
use tracing::{info, instrument};

use crate::{
    response_code::{ErrorCode, ResponseCode},
    soundlist::Sound,
    SoundList,
};

mod builder;
mod command;
mod connection;

pub use builder::ClientBuilder;
pub(crate) use command::Command;
pub(crate) use connection::Connection;

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

type Result<T> = std::result::Result<T, Error>;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Client {
    #[derivative(Debug = "ignore")]
    pub(crate) tx: mpsc::Sender<Command>,
    pub debounce: core::time::Duration,
}

impl Client {
    #[instrument]
    pub async fn get_sound_list(&self) -> Result<Vec<Sound>> {
        use command::Error::*;

        match Command::new("GetSoundList()").issue(self).await {
            Ok(SoundList { sounds }) => Ok(sounds),
            Err(e) => Err(match e {
                Pipe(e) => Error::Connection(e),
                Dropped => panic!("Actor & Handles lost connection"),
                Parse(e) => {
                    Error::BadResponse(eyre::Error::from(e).wrap_err("Could not deserialize XML"))
                }
            }),
        }
    }

    #[instrument]
    pub async fn play_sound(&self, sound: &Sound) -> Result<()> {
        use command::Error::*;
        use ErrorCode::*;

        let msg = format!("DoPlaySound({})", sound.index);

        match Command::new(msg.clone())
            .with_cooldown(self.debounce)
            .issue(self)
            .await
        {
            Ok(ResponseCode::Ok) => {
                info!("{}", sound.title);
                Ok(())
            }
            Err(e) => Err(match e {
                Pipe(e) => Error::Connection(e),
                Dropped => panic!("Actor & Handles lost connection"),
                Parse(e) => match e {
                    CommandNotFound | BadRequest => Error::InvalidCommand(msg),
                    NoContent => Error::NotFound {
                        missing: format!("{} at index {}", sound.title, sound.index),
                    },
                    NotFound(s) | Unknown(s) => eyre!("Soundpad says: {s}").into(),
                },
            }),
        }
    }
}
