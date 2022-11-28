use derivative::Derivative;
use thiserror::Error;
use tokio::{net::windows::named_pipe::ClientOptions, sync::mpsc, sync::oneshot};
use tracing::{info, instrument};
use winapi::shared::winerror;

use crate::{soundlist::Sound, SoundList};

mod command;
mod connection;

pub(crate) use command::Command;
pub(crate) use connection::Connection;

const PIPE_NAME: &str = r"\\.\pipe\sp_remote_control";

#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("Could not connect to soundpad. Is it running?")]
    NotFound { source: std::io::Error },
    #[error("Soundpad is not accepting connections")]
    Busy,
    #[error(transparent)]
    Other(#[from] std::io::Error),
}

#[derive(Debug, Default, Clone)]
pub struct ClientBuilder {
    debounce: core::time::Duration,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn debounce(mut self, duration: core::time::Duration) -> Self {
        self.debounce = duration;
        self
    }

    pub fn connect(self) -> Result<Client, ConnectError> {
        match ClientOptions::new().open(PIPE_NAME) {
            Ok(pipe) => {
                let (tx, rx) = mpsc::channel(8);

                let connection = Connection {
                    rx,
                    pipe,
                    debounce: self.debounce,
                };

                tokio::spawn(connection::run_actor(connection));

                Ok(Client { tx })
            }
            Err(e) if e.raw_os_error() == Some(winerror::ERROR_PIPE_BUSY as i32) => {
                Err(ConnectError::Busy)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ConnectError::NotFound { source: e })
            }
            Err(e) => Err(ConnectError::Other(e)),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Client {
    #[derivative(Debug = "ignore")]
    tx: mpsc::Sender<Command>,
}

impl Client {
    #[instrument]
    pub async fn get_sound_list(&self) -> Vec<Sound> {
        let (respond_to, rx) = oneshot::channel();
        self.tx
            .send(Command::GetSoundList { respond_to })
            .await
            .expect("the sender to be still alive");
        if let Ok(SoundList { sounds }) = rx.await {
            sounds
        } else {
            Vec::new()
        }
    }

    #[instrument]
    pub async fn play_sound(&self, sound: &Sound) {
        self.tx
            .send(Command::PlaySound { index: sound.index })
            .await
            .expect("the sender to be still alive");
    }
}
