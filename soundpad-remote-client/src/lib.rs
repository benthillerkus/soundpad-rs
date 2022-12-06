#![feature(error_generic_member_access)]
#![feature(provide_any)]

mod client;
mod error;
mod response_code;

pub use client::ClientBuilder;
pub use soundpad_xml::{Sound, SoundList};
