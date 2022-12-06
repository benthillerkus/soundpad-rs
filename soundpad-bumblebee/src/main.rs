use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use soundpad_remote_client::ClientBuilder;
use tracing::{info, metadata::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format, prelude::*};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    message: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let registry = tracing_subscriber::registry();
    #[cfg(feature = "console")]
    {
        let registry = registry.with(console_subscriber::spawn());
    }

    registry
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(format().pretty())
                .with_filter(LevelFilter::INFO),
        )
        .with(ErrorLayer::default())
        .init();
    color_eyre::install()?;
    info!("Starting up...");

    let client = ClientBuilder::new()
        .connect()?;

    info!("Connected to soundpad and ready!");

    let sounds = client.get_sound_list().await?;

    for word in args.message {
        let sound = sounds
            .iter()
            .find(|&s| s.title.to_lowercase().contains(&word.to_lowercase()))
            .ok_or_else(|| eyre!("Could not find a sound containing {}", word))?;

        client.play_sound(sound).await?;
    }
    Ok(())
}
