use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Box<error::Error>>;

#[derive(Debug)]
pub enum Error {
    DotfileNotFound,
    DotfileBlocked,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DotfileNotFound => write!(f, "dotfile not found"),
            Error::DotfileBlocked => write!(f, "target path differs from dotfile target"),
        }
    }
}

impl error::Error for Error {}
