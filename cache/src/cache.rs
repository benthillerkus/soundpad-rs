use rusqlite::Connection;
use tokio::{sync::mpsc::UnboundedReceiver, task::block_in_place};

use crate::message::{Message, Version};

pub(crate) struct Cache {
    pub(crate) conn: Connection,
    pub(crate) rx: UnboundedReceiver<Message>,
}

pub(crate) async fn run_actor(mut cache: Cache) {
    while let Some(message) = cache.rx.recv().await {
        match message {
            Message::Version { callback } => {
                let version = block_in_place(|| version(&cache.conn));
                callback.send(version).unwrap();
            }
            Message::Search { query } => todo!(),
            Message::AddSongs => todo!(),
        }
    }
}

fn version(conn: &Connection) -> Version {
    let library = conn
        .prepare_cached("SELECT sqlite_version() AS version;")
        .unwrap()
        .query_row((), |row| row.get::<_, String>(0))
        .unwrap();

    Version {
        library,
        cache: "0.1.0".into(),
        app: "0.1.0".into(),
    }
}
