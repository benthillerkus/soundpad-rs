use thiserror::Error;
use tokio::{io, sync::oneshot};
use tracing::{info, instrument};

use crate::SoundList;

use super::Connection;

#[derive(Debug)]
pub(crate) enum Command {
    GetSoundList {
        respond_to: oneshot::Sender<SoundList>,
    },
    PlaySound {
        index: u64,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Issue communicating with Soundpad")]
    Connection(#[from] io::Error),
    #[error("Could not parse response")]
    ParseXML(#[from] serde_xml_rs::Error),
    #[error("Could not respond to Handle. It was dropped")]
    ClientDropped,
}

impl Command {
    #[instrument]
    pub(crate) async fn do_work(self, connection: &mut Connection) -> Result<(), Error> {
        match self {
            Command::GetSoundList { respond_to } => {
                connection.send(b"GetSoundList()").await?;
                let response = connection.receive().await?;
                let sounds: SoundList = serde_xml_rs::from_str(&response)?;
                respond_to.send(sounds).map_err(|_| Error::ClientDropped)?;
            }
            Command::PlaySound { index } => {
                connection.send(format!("DoPlaySound({index})")).await?;
                let response = connection.receive().await?;
                tokio::time::sleep(connection.debounce).await;
            }
        }
        Ok(())
    }
}
