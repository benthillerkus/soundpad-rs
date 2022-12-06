#![allow(unused)]

use color_eyre::eyre;
use std::backtrace::Backtrace;
use thiserror::Error;
use tokio::io;
use tracing_error::TracedError;

pub use color_eyre::eyre::{eyre, Context, Report};

#[derive(Debug, Error)]
pub enum Kind {
    #[error("Soundpad received a syntactically wrong command")]
    BadRequest,
    #[error("Connection error")]
    Connection(#[from] io::Error),
    #[error("Couldn't convert Soundpad's response into the desired type")]
    Conversion(#[source] eyre::Report),
    #[error("Soundpad doesn't recognize this command")]
    CommandNotFound,
    #[error("Couldn't find {0}")]
    NotFound(String),
    #[error("Soundpad understood the command, but there is no content to act on")]
    NoContent,
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

#[derive(Debug)]
pub struct Error {
    source: TracedError<Kind>,
    backtrace: Backtrace,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.source, f)
    }
}

impl<E> From<E> for Error
where
    Kind: From<E>,
{
    fn from(source: E) -> Self {
        Self {
            source: Kind::from(source).into(),
            backtrace: Backtrace::capture(),
        }
    }
}
