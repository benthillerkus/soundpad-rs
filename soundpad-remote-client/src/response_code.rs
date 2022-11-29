use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone)]
pub enum ResponseCode {
    Ok,
}

#[derive(Debug, Clone, Error)]
pub enum ErrorCode {
    #[error("Soundpad understood the command, but there is no content to act on")]
    NoContent,
    #[error("Soundpad received a syntactically wrong command")]
    BadRequest,
    #[error("Soundpad didn't find something: {0}")]
    NotFound(String),
    #[error("Soundpad doesn't know this command")]
    CommandNotFound,
    #[error("Soundpad had this problem: {0}")]
    Unknown(String),
}

impl FromStr for ResponseCode {
    type Err = ErrorCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[.."R-200".len()] {
            "R-200" => Ok(Self::Ok),
            "R-204" => Err(ErrorCode::NoContent),
            "R-400" => Err(ErrorCode::BadRequest),
            "R-404" => match &s["R-404: ".len()..] {
                "Command not found." => Err(ErrorCode::CommandNotFound),
                s => Err(ErrorCode::NotFound(s.into())),
            },
            s => Err(ErrorCode::Unknown(s.into())),
        }
    }
}
