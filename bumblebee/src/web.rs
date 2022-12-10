use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Form, Router,
};
use serde::Deserialize;
use soundpad_remote_client::{Client, Sound};
use tracing::{info, instrument};

use crate::{play, Args};

pub fn run(args: &Args, client: Client, sounds: Vec<Sound>) {
    let shared_state = AppState { client, sounds };
    let app = Router::new()
        .route("/", get(show_form).post(accept_form))
        .with_state(shared_state);
    info!("Opened web interface on http://{}", args.address);

    tokio::spawn(axum::Server::bind(&args.address).serve(app.into_make_service()));

    tokio::task::spawn_blocking({
        let address = args.address;
        move || webbrowser::open(&format!("http://{}", address))
    });
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
