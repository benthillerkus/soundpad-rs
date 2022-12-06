#![feature(error_generic_member_access)]
#![feature(provide_any)]

mod parse_or;
mod soundlist;
mod client;
mod response_code;
mod error;

pub use soundlist::SoundList;
pub use client::ClientBuilder;