use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use clap::Parser;
use color_eyre::eyre::Result;
use play::play;
use serde::Deserialize;
use soundpad_remote_client::{Client, ClientBuilder, Sound};
use tracing::{info, instrument, metadata::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format, prelude::*};

mod play;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    message: Vec<String>,

    #[clap(short, long, default_value = "127.0.0.1:3003")]
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

    let sounds = client.get_sound_list().await?;
    if !args.message.is_empty() {
        play(args.message, sounds, client).await?;
    } else {
        let shared_state = AppState { client, sounds };
        let app = Router::new()
            .route("/", get(show_form).post(accept_form))
            .with_state(shared_state);
        info!("Opened web interface on http://{}", args.address);
        axum::Server::bind(&args.address)
            .serve(app.into_make_service())
            .await?;
    }
    Ok(())
}

async fn show_form() -> Html<&'static str> {
    let page = include_str!("./index.html");
    Html(page)
}

#[derive(Debug, Clone)]
struct AppState {
    client: Client,
    sounds: Vec<Sound>,
}

#[derive(Deserialize, Debug)]
struct Input {
    message: String,
}

#[instrument]
async fn accept_form(State(state): State<AppState>, Form(input): Form<Input>) -> impl IntoResponse {
    info!("Got a request!");

    let message = input
        .message
        .clone()
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    let sounds = state.sounds.clone();
    let client = state.client.clone();
    tokio::spawn(play(message, sounds, client));
    show_form().await
}
