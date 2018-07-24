mod args;
mod dotfile;
mod error;

extern crate clap;
extern crate dirs;
extern crate failure;
extern crate walkdir;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use args::{Command, Opt};
use dotfile::{DotfileState, Store};
use error::{pretty_err, AppError, Result};

fn main() {
    let matches = Opt::from_args();

    let dot_root = matches.store.unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap();
        home.join(".dotfiles")
    });
    let store = Store::new(&dot_root);

    let res = match matches.cmd {
        Command::Add { dotfile } => run_add_command(&store, dotfile),
        Command::Remove { dotfile } => run_remove_command(&store, dotfile),
        Command::List {} => run_list_command(&store),
    };

    if let Err(err) = res {
        println!("Error: {}", pretty_err(&err));
    }
}

fn run_add_command<P: Into<PathBuf>>(store: &Store, name: P) -> Result<()> {
    let name = name.into();
    let dotfile = store
        .get(&name)
        .ok_or_else(|| AppError::DotfileNotFound(name))?;

    match dotfile.state() {
        DotfileState::Installed => Ok(()),
        DotfileState::Blocked => Err(AppError::DotfileBlocked(dotfile.target()))?,
        DotfileState::Uninstalled => {
            if let Some(parent) = dotfile.target().parent() {
                fs::create_dir_all(parent).map_err(AppError::IOError)?;
            }
            symlink(dotfile.source(), dotfile.target()).map_err(AppError::IOError)?;
            Ok(())
        }
    }
}

fn run_remove_command<P: Into<PathBuf>>(store: &Store, name: P) -> Result<()> {
    let name = name.into();
    let dotfile = store
        .get(&name)
        .ok_or_else(|| AppError::DotfileNotFound(name))?;

    match dotfile.state() {
        DotfileState::Installed => {
            fs::remove_file(dotfile.target())?;
            Ok(())
        }
        DotfileState::Blocked => Err(AppError::DotfileBlocked(dotfile.target()))?,
        DotfileState::Uninstalled => Ok(()),
    }
}

fn run_list_command(store: &Store) -> Result<()> {
    println!("Dotfiles from {:?}:", store.root());

    for dotfile in store.all() {
        println!(
            "[{}] {}",
            match dotfile.state() {
                DotfileState::Installed => "x",
                DotfileState::Blocked => "-",
                DotfileState::Uninstalled => " ",
            },
            dotfile.name().to_string_lossy(),
        );
    }

    Ok(())
}
