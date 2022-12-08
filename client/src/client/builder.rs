use std::borrow::Cow;

use thiserror::Error;
use tokio::{net::windows::named_pipe::ClientOptions, sync::mpsc};
use tracing::instrument;
use windows_sys::Win32::Foundation::ERROR_PIPE_BUSY;

use super::{connection, Client, Connection};

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
    pipe_name: Cow<'static, str>,
    max_queue_len: usize,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            pipe_name: PIPE_NAME.into(),
            max_queue_len: 8,
            ..Default::default()
        }
    }

    pub fn debounce(mut self, duration: core::time::Duration) -> Self {
        self.debounce = duration;
        self
    }

    #[instrument]
    pub fn connect(self) -> Result<Client, ConnectError> {
        match ClientOptions::new().open(self.pipe_name.as_ref()) {
            Ok(pipe) => {
                let (tx, rx) = mpsc::channel(self.max_queue_len);

                let connection = Connection { rx, pipe };

                tokio::spawn(connection::run_actor(connection));

                Ok(Client {
                    tx,
                    debounce: self.debounce,
                })
            }
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) => Err(ConnectError::Busy),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ConnectError::NotFound { source: e })
            }
            Err(e) => Err(ConnectError::Other(e)),
        }
    }
}
