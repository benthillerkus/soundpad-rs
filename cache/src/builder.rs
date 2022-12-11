use color_eyre::eyre::{Context, Result};
use rusqlite::Connection;
use soundpad_remote_client::Client;
use std::path::PathBuf;
use tokio::{sync::mpsc, task};

use crate::{
    cache::{run_actor, Cache},
    sync::sync_task,
    Pool, APP_DIR, FILE_NAME,
};

pub struct CacheBuilder {
    path: Option<PathBuf>,
    client: Option<Client>,
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            client: None,
        }
    }
}

impl CacheBuilder {
    /// Set a custom file path for the database file.
    ///
    /// This will not create any *directories* for you.
    /// SQLite will create the *file* if it doesn't exist.
    ///
    /// This function is just a setter, actually doing stuff is deferred to [`init()`].
    ///
    /// If you don't set this, the default path is `C:\Users\{username}\AppData\Roaming\soundpad-rs\cache.db3`.
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn client(self, client: Client) -> CacheBuilder {
        CacheBuilder {
            path: self.path,
            client: Some(client),
        }
    }

    pub async fn init(self) -> Result<Pool> {
        // get or insert
        let path = if let Some(path) = self.path {
            path
        } else {
            let mut path = app_data_dir()?;
            path.push(APP_DIR);
            match tokio::fs::create_dir(&path).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => Err(e).wrap_err_with(|| {
                    format!("Couldn't create the app directory at {}", path.display())
                })?,
            }
            path.push(FILE_NAME);
            path
        };

        let conn = task::block_in_place(|| Connection::open(path))?;
        let (tx, rx) = mpsc::unbounded_channel();
        let cache = Cache { conn, rx };
        tokio::spawn(run_actor(cache));

        let pool = Pool { tx };

        dbg!(pool.version().await);

        if let Some(client) = self.client {
            tokio::spawn(sync_task(pool.clone(), client));
        }

        Ok(pool)
    }
}

/// Get the AppData directory, typically located at `C:\Users\{username}\AppData\Roaming`.
///
/// ```rust
/// # use soundpad_cache::builder::*;
/// let app_data = app_data_dir()?;
/// assert!(app_data.display().to_string().contains("Roaming"));
/// # Ok::<(), Error>(())
/// ```
pub fn app_data_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(std::env::var("AppData").wrap_err(
        "Couldn't get the AppData directory. The %AppData% environment variable may be not set.",
    )?))
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn init() {
        let builder = CacheBuilder::new();
        builder.init().await.unwrap();
    }
}
