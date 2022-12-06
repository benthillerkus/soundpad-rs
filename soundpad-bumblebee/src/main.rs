use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use soundpad_remote_client::ClientBuilder;
use std::time::Duration;
use tracing::{info, metadata::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format, prelude::*};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    message: String,
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
        .debounce(Duration::from_millis(800))
        .connect()?;

    info!("Connected to soundpad and ready!");

    let sounds = client.get_sound_list().await?;

    let sound = sounds
        .iter()
        .find(|&s| s.title.contains(&args.message))
        .ok_or(eyre!("Could not find a sound containing {}", args.message))?;

    client.play_sound(sound).await?;
    Ok(())
}
