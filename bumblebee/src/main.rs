use clap::Parser;
use color_eyre::eyre::Result;
use play::play;
use soundpad_cache::CacheBuilder;
use soundpad_remote_client::ClientBuilder;
use std::net::SocketAddr;
use tracing::{info, metadata::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format, prelude::*};

mod play;
#[cfg(feature = "web")]
mod web;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    message: Vec<String>,

    #[clap(short, long, default_value = "127.0.0.1:5338")]
    address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let registry = tracing_subscriber::registry();
    #[cfg(feature = "console")]
    let registry = registry.with(console_subscriber::spawn());

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

    let client = ClientBuilder::new().connect()?;

    info!("Connected to Soundpad and ready!");

    let cache = CacheBuilder::new().client(client.clone()).init().await?;

    let sounds = client.get_sound_list().await?;
    if !args.message.is_empty() {
        play(args.message, sounds, client).await?;
    } else {
        #[cfg(feature = "web")]
        crate::web::run(&args, client, sounds);
    }

    tokio::signal::ctrl_c().await?;

    Ok(())
}
