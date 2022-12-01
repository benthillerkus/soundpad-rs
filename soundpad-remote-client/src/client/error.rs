use color_eyre::eyre;
use thiserror::Error;
use tokio::io;

#[derive(Debug, Error)]
pub enum CriticalError {
    #[error("Issue communicating with Soundpad")]
    Connection(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}