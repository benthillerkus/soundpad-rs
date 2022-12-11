# soundpad-rs

> Libraries for interacting with Soundpad

## Usage

```rust
#[tokio::main]
async fn main -> Result<()> {
  // Create a new API client
  let client = ClientBuilder::new().connect()?;
  // Retrieve a list with all available sounds from Soundpad
  let sounds = client.get_sound_list().await?;
  // Play the first sound
  client.play_sound(sounds[0])
}
```

## Overview

|crate|description|details|type|
|-|-|-|-|
|[`soundpad-remote-client`](https://crates.io/crates/soundpad-remote-client)|An asynchronous client for the Named Pipes based remote control API.|tokio, actor pattern|lib|
|[`soundpad-xml`](https://crates.io/crates/soundpad-xml)|`FromStr` implementations for the various XML based formats Soundpad uses.|serde, serde-xml-derive|lib|
|[`soundpad-bumblebee`](https://crates.io/crates/soundpad-bumblebee)|An app for playing sentences consisting of different soundbites (word mixing) |tokio, clap, axum|binary|
|[`soundpad-cache`](https://crates.io/crates/soundpad-cache)|A cache for soundfiles known to Soundpad.<br>Actually checks out the files to determine their exact length|rusqlite, lofty, actor pattern|lib|
