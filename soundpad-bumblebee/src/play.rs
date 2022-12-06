use color_eyre::eyre::Result;
use soundpad_remote_client::{Client, Sound};
use tracing::warn;

pub async fn play(message: Vec<String>, sounds: Vec<Sound>, client: Client) -> Result<()> {
    for word in message {
        match sounds
            .iter()
            .find(|&s| s.title.to_lowercase().contains(&word.to_lowercase()))
        {
            Some(sound) => {
                client.play_sound(sound).await?;
            }
            None => {
                warn!("No sound found for {word}");
            }
        }
    }
    Ok(())
}
