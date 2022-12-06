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

## Crates

- [`soundpad-remote-client`](https://crates.io/crates/soundpad-remote-client)
- [`soundpad-xml`](https://crates.io/crates/soundpad-xml)
- [`soundpad-bumblebee`](https://crates.io/crates/soundpad-bumblebee)