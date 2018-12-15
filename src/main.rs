mod args;
mod dotfile;
mod error;

use std::fs;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::args::{Command, Opt};
use crate::dotfile::{DFState, Dotfile, Store};
use crate::error::{AppError, Result};

fn do_main(args: Opt) -> Result<()> {
    let home_dir = dirs::home_dir().unwrap();
    let dot_root = args.store.unwrap_or_else(|| home_dir.join(".dotfiles"));

    if !fs::metadata(&dot_root)?.is_dir() {
        return AppError::result(&format!(
            "Given store path `{} is not a directory.",
            dot_root.display()
        ));
    }

    let mut store = Store::new(&dot_root);

    match args.cmd {
        Command::Install { dotfiles } => install_dotfiles(&store, &dotfiles),
        Command::Uninstall { dotfiles } => uninstall_dotfiles(&store, &dotfiles),
        Command::Manage { dotfiles } => manage_dotfiles(&mut store, &dotfiles),
        Command::Unmanage { dotfiles } => unmanage_dotfiles(&mut store, &dotfiles),
        Command::List {} => list_dotfiles(&store),
    }
}

fn install_dotfiles(store: &Store, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let dotfile = fetch_dotfile(store, path)?;

        match dotfile.state() {
            DFState::Installed => Ok(()),
            DFState::Blocked => AppError::result(&format!(
                "Dotfile target `{}` is blocked.",
                dotfile.target.display()
            )),
            DFState::Uninstalled => dotfile.install(),
        }?;
    }

    Ok(())
}

fn uninstall_dotfiles(store: &Store, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let dotfile = fetch_dotfile(store, path)?;

        match dotfile.state() {
            DFState::Installed => dotfile.uninstall(),
            DFState::Blocked | DFState::Uninstalled => Ok(()),
        }?;
    }

    Ok(())
}

fn manage_dotfiles(store: &mut Store, targets: &[PathBuf]) -> Result<()> {
    for target in targets {
        if store.get(target).is_some() {
            return AppError::result(&format!(
                "Dotfile with target `{}` already exists in the store.",
                target.display()
            ));
        }

        Dotfile::from_target(&store.path, target)
            .and_then(|df| df.store().map(|_| df))
            .map(|df| store.add(df))?;
    }

    Ok(())
}

fn unmanage_dotfiles(store: &mut Store, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let dotfile = fetch_dotfile(store, path)?;
        dotfile.unstore()?;
        store.remove(path);
    }

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
        None => AppError::result(&format!(
            "Dotfile not found with reference `{}`.",
            path.display()
        )),
    }
}

fn main() {
    let args = Opt::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
