use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidToken,
    Jwt(jsonwebtoken::errors::Error),
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::Jwt(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidToken => f.write_str("invalid token"),
            Error::Jwt(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
