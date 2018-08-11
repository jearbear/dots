#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate structopt;

extern crate clap;
extern crate dirs;
extern crate walkdir;

mod args;
mod dotfile;
mod error;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use structopt::StructOpt;

use args::{Command, Opt};
use dotfile::{DotfileState, Store};
use error::*;

lazy_static! {
    static ref HOME_DIR: PathBuf = dirs::home_dir().unwrap();
}

fn do_main(args: Opt) -> Result<()> {
    let dot_root = args.store.unwrap_or_else(|| HOME_DIR.join(".dotfiles"));
    let store = Store::new(&dot_root);

    match args.cmd {
        Command::Add { dotfiles } => run_add_commands(&store, dotfiles),
        Command::Remove { dotfiles } => run_remove_commands(&store, dotfiles),
        Command::List {} => run_list_command(&store),
    }?;

    Ok(())
}

fn run_add_commands<P: Into<PathBuf>>(store: &Store, names: Vec<P>) -> Result<()> {
    names
        .into_iter()
        .map(|name| name.into())
        .map(|name| run_add_command(store, name))
        .collect::<Result<_>>()?;

    Ok(())
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

fn run_remove_commands<P: Into<PathBuf>>(store: &Store, names: Vec<P>) -> Result<()> {
    names
        .into_iter()
        .map(|name| name.into())
        .map(|name| run_remove_command(store, name))
        .collect::<Result<_>>()?;

    Ok(())
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
    eprintln!("Printing dotfiles from {:?}", store.root());
    eprintln!("Legend: [x] installed, [-] blocked, [ ] uninstalled\n");

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

fn main() {
    let args = Opt::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}
