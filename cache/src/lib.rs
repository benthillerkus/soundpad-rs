mod pool;
mod cache;
mod message;
mod sync;
pub mod builder;

pub use pool::Pool;
pub use builder::CacheBuilder;

const FILE_NAME: &str = "cache.db3";
const APP_DIR: &str = "soundpad-rs";