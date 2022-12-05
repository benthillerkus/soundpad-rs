use std::str::FromStr;

use crate::error::{eyre, Error, Kind};
use tracing::instrument;

#[derive(Debug, Clone)]
pub enum SuccessCode {
    Ok,
}

impl FromStr for SuccessCode {
    type Err = Error;

    #[instrument]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[.."R-200".len()] {
            "R-200" => Ok(Self::Ok),
            "R-204" => Err(Kind::NoContent)?,
            "R-400" => Err(Kind::BadRequest)?,
            "R-404" => match &s["R-404: ".len()..] {
                "Command not found." => Err(Kind::CommandNotFound)?,
                s => Err(Kind::NotFound(s.into()))?,
            },
            s => Err(eyre!("Unknown response code: {}", s))?,
        }
    }
}
