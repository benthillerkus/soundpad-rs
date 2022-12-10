use super::Command;
use derivative::Derivative;
use tokio::{io, net::windows::named_pipe::NamedPipeClient, sync::mpsc, time::Sleep};
use tracing::instrument;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Connection {
    #[derivative(Debug = "ignore")]
    pub(crate) rx: mpsc::UnboundedReceiver<Command>,
    pub(crate) pipe: NamedPipeClient,
}

#[instrument]
pub(crate) async fn run_actor(mut connection: Connection) {
    let mut sleep: Option<Sleep> = None;

    while let Some(command) = connection.rx.recv().await {
        if !command.cooldown.is_zero() {
            if let Some(sleep) = sleep.take() {
                sleep.await;
            }
            sleep = Some(tokio::time::sleep(command.cooldown));
        }
        command.do_work(&mut connection).await;
    }
}

impl Connection {
    pub(crate) async fn send_and_receive(
        &mut self,
        message: impl AsRef<[u8]>,
    ) -> io::Result<String> {
        self.send(message).await?;
        self.receive().await
    }

    pub(crate) async fn send(&self, command: impl AsRef<[u8]>) -> io::Result<()> {
        let bytes = command.as_ref();
        let mut written = 0;

        while written < bytes.len() {
            self.pipe.writable().await?;
            match self.pipe.try_write(&bytes[written..]) {
                Ok(amount) => written += amount,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn receive(&self) -> io::Result<String> {
        let mut response = String::new();
        let mut buf = [0u8; 4096];

        loop {
            // Wait for the pipe to be readable
            self.pipe.readable().await?;

            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match self.pipe.try_read(&mut buf) {
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if n < buf.len() {
                        return Ok(response);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
