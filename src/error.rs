use failure::Error;

use std::path::PathBuf;
use std::{io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "Dotfile not found at path: {:?}.", _0)]
    DotfileNotFound(PathBuf),

    #[fail(display = "Dotfile target {:?} is blocked by another file.", _0)]
    DotfileBlocked(PathBuf),

    #[fail(display = "IO error occurred.")]
    IOError(#[cause] io::Error),
}

pub fn pretty_err(err: &Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.cause();
    while let Some(next) = prev.cause() {
        pretty.push_str(" : ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}
