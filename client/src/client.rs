use core::time::Duration;
use derivative::Derivative;
use std::fmt::Debug;
use tokio::sync::mpsc;
use tracing::{info, instrument};

use crate::{error::Error, response_code::SuccessCode, Sound, SoundList};

mod builder;
mod command;
mod connection;

pub use builder::ClientBuilder;
pub(crate) use command::Command;
pub(crate) use connection::Connection;

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
    pub(crate) tx: mpsc::UnboundedSender<Command>,
    pub debounce: Duration,
}

impl Client {
    #[instrument]
    pub async fn custom_command(
        &self,
        command: impl ToString + Debug,
        cooldown: Duration,
    ) -> Result<SuccessCode> {
        Command::new(command.to_string())
            .with_cooldown(cooldown)
            .issue::<SuccessCode, _>(self)
            .await
    }

    #[instrument]
    pub async fn get_sound_list(&self) -> Result<Vec<Sound>> {
        Command::new("GetSoundList()")
            .issue::<SoundList, _>(self)
            .await
            .map(|sl| sl.sounds)
    }

    #[instrument]
    pub async fn play_sound(&self, sound: &Sound) -> Result<()> {
        let msg = format!("DoPlaySound({})", sound.index);

        let _ = Command::new(msg.clone())
            .with_cooldown(self.debounce + sound.duration)
            .issue::<SuccessCode, _>(self)
            .await?;

        info!("{}", sound.title);

        Ok(())
    }
}
