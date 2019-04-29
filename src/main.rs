#[macro_use]
mod error;
mod args;
mod dotfile;

use std::path::{Path, PathBuf};

use crate::args::Command;
use crate::dotfile::{DFState, Dotfile, Store};
use crate::error::Result;

fn main() {
    let res = args::init().map(|args| match args.command {
        Command::List => list_dotfiles(&args.store),
        Command::Install(df) => install_dotfile(&args.store, &df),
        Command::Uninstall(df) => uninstall_dotfile(&args.store, &df),
        Command::Manage(df) => manage_dotfile(&args.store, &df),
        Command::Unmanage(df) => unmanage_dotfile(&args.store, &df),
    });

    if let Err(err) = res {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn install_dotfile(store: &Store, path: &PathBuf) -> Result<()> {
    let dotfile = fetch_dotfile(store, path)?;

    match dotfile.state() {
        DFState::Installed => Ok(()),
        DFState::Blocked => err!("Dotfile target `{}` is blocked.", dotfile.target.display()),
        DFState::Uninstalled => dotfile.install(),
    }?;

    Ok(())
}

fn uninstall_dotfile(store: &Store, path: &PathBuf) -> Result<()> {
    let dotfile = fetch_dotfile(store, path)?;

    match dotfile.state() {
        DFState::Installed => dotfile.uninstall(),
        DFState::Blocked | DFState::Uninstalled => Ok(()),
    }?;

    Ok(())
}

fn manage_dotfile(store: &Store, target: &PathBuf) -> Result<()> {
    if store.get(target).is_some() {
        return err!(
            "Dotfile with target `{}` already exists in the store.",
            target.display()
        );
    }

    Dotfile::from_target(&store.path, target)?.store()?;

    Ok(())
}

fn unmanage_dotfile(store: &Store, path: &PathBuf) -> Result<()> {
    fetch_dotfile(store, path)?.unstore()?;

    Ok(())
}

fn list_dotfiles(store: &Store) -> Result<()> {
    eprintln!("Printing dotfiles from {}", store.path.display());
    eprintln!("Legend: [x] installed, [-] blocked, [ ] uninstalled\n");

    for dotfile in store.all() {
        println!(
            "[{}] {}",
            match dotfile.state() {
                DFState::Installed => "x",
                DFState::Blocked => "-",
                DFState::Uninstalled => " ",
            },
            dotfile.name.display(),
        );
    }

    Ok(())
}

fn fetch_dotfile<'a>(store: &'a Store, path: &Path) -> Result<&'a Dotfile> {
    match store.get(path) {
        Some(dotfile) => Ok(dotfile),
        None => err!("Dotfile not found with reference `{}`.", path.display()),
    }
}
