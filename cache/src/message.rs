use std::fmt::Display;

use tokio::sync::oneshot;

#[derive(Debug)]
pub(crate) enum Message {
    Version {
        callback: oneshot::Sender<Version>,
    },
    AddSongs,
    Search {
        query: String,
        // callback: oneshot::Sender<Vec<String>>,
    },
}

#[derive(Debug)]
pub struct Version {
    pub library: String,
    pub cache: String,
    pub app: String,
}
