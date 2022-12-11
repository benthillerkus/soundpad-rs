use tokio::sync::mpsc::UnboundedSender;

use crate::message::{Message, Version};

#[derive(Clone)]
pub struct Pool {
    pub(crate) tx: UnboundedSender<Message>,
}

impl Pool {
    pub async fn version(&self) -> Version {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Message::Version { callback: tx }).unwrap();
        rx.await.unwrap()
    }
}
