mod args;
mod dotfile;
mod error;

use std::fs;
use std::path::{Path, PathBuf};

use failure::{bail, ensure, format_err, ResultExt};
use lazy_static::lazy_static;
use structopt::StructOpt;

use crate::args::{Command, Opt};
use crate::dotfile::{DFState, Dotfile, Store};
use crate::error::Result;

lazy_static! {
    static ref HOME_DIR: PathBuf = dirs::home_dir().unwrap();
}

fn do_main(args: Opt) -> Result<()> {
    let dot_root = args.store.unwrap_or_else(|| HOME_DIR.join(".dotfiles"));

    ensure!(
        fs::metadata(&dot_root)?.is_dir(),
        "Given store path `{}` is not a directory.",
        dot_root.display()
    );

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
            DFState::Blocked => Err(format_err!(
                "Dotfile target `{}` is blocked.",
                dotfile.target.display()
            )),
            DFState::Uninstalled => dotfile.install(),
        }
        .context(format!("Failed to install `{}`.", dotfile.name.display()))?;
    }

    Ok(())
}

fn uninstall_dotfiles(store: &Store, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let dotfile = fetch_dotfile(store, path)?;

        match dotfile.state() {
            DFState::Installed => dotfile.uninstall(),
            DFState::Blocked | DFState::Uninstalled => Ok(()),
        }
        .context(format!("Failed to uninstall `{}`.", dotfile.name.display()))?;
    }

    Ok(())
}

fn manage_dotfiles(store: &mut Store, targets: &[PathBuf]) -> Result<()> {
    for target in targets {
        ensure!(
            store.get(target).is_none(),
            "Dotfile with target `{}` already exists in the store.",
            target.display()
        );

        Dotfile::from_target(&store.path, target)
            .and_then(|df| df.store().map(|_| df))
            .map(|df| store.add(df))
            .context(format!("Failed to manage `{}`.", target.display()))?;
    }

    Ok(())
}

fn unmanage_dotfiles(store: &mut Store, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        {
            let dotfile = fetch_dotfile(store, path)?;

            dotfile
                .unstore()
                .context(format!("Failed to unmanage `{}`", dotfile.name.display()))?;
        }

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
    store
        .get(path)
        .ok_or_else(|| format_err!("Dotfile not found with reference `{}`.", path.display()))
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
