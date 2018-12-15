use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct AppError {
    msg: String,
}

impl AppError {
    pub fn new(msg: &str) -> AppError {
        AppError {
            msg: msg.to_owned(),
        }
    }

    pub fn result<T>(msg: &str) -> Result<T> {
        Err(Box::new(AppError::new(msg)))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for AppError {}
