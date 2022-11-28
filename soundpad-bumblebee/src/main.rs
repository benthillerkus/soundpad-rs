use color_eyre::eyre::Result;
use soundpad_remote_client::ClientBuilder;
use std::time::Duration;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
    .with(console_subscriber::spawn())
    .with(
        tracing_subscriber::fmt::layer()
        .event_format(format().pretty())
        .with_filter(LevelFilter::INFO),
    )
    .init();
    color_eyre::install()?;
    info!("Starting up...");

    let client = ClientBuilder::new()
        .debounce(Duration::from_millis(800))
        .connect()?;

    info!("Connected to soundpad and ready!");

    let sounds = client.get_sound_list().await;
    // println!("{:#?}", sounds);

    let esel = sounds.iter().find(|&s| s.title.contains("Esel")).unwrap();

    client.play_sound(esel).await;
    Ok(())
}
