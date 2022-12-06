use clap::Parser;
use color_eyre::eyre::Result;
use play::play;
use soundpad_remote_client::ClientBuilder;
use std::{
    io::{self, Write},
    sync::mpsc,
    thread,
};
use tracing::{info, metadata::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format, prelude::*};

mod play;

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

    let client = ClientBuilder::new().connect()?;

    info!("Connected to soundpad and ready!");

    let sounds = client.get_sound_list().await?;
    if !args.message.is_empty() {
        play(&args.message, &sounds, &client).await?;
    } else {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut input = String::new();
            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut input).unwrap();
                sender.send(input.clone()).unwrap();
                input.clear();
            }
        });
        while let Ok(input) = receiver.recv() {
            let message = input.split_whitespace().collect::<Vec<_>>();
            play(&message, &sounds, &client).await?;
        }
    }
    Ok(())
}
