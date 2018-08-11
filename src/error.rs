use failure::Fallible;

use std::io;
use std::path::PathBuf;

pub type Result<T> = Fallible<T>;

#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "Dotfile not found at path {:?}.", _0)]
    DotfileNotFound(PathBuf),

    #[fail(display = "Dotfile target {:?} is blocked by another file.", _0)]
    DotfileBlocked(PathBuf),

    #[fail(display = "IO error occurred.")]
    IOError(#[cause] io::Error),
}
