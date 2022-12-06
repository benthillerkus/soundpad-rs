# soundpad-rs

> Libraries for interacting with Soundpad

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
