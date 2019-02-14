use std::fs;
use std::path::{Path, PathBuf};

use clap::{App, AppSettings, Arg, SubCommand};

use crate::dotfile::Store;
use crate::error::Result;

pub struct Args {
    pub store: Store,
    pub command: Command,
}

pub enum Command {
    List,
    Install(PathBuf),
    Uninstall(PathBuf),
    Manage(PathBuf),
    Unmanage(PathBuf),
}

pub fn init() -> Result<Args> {
    let store = Arg::with_name("store")
        .help("Path of dotfile store [default: ~/.dotfiles]")
        .long("store")
        .value_name("PATH")
        .global(true);

    let dotfiles = Arg::with_name("dotfile")
        .help("Dotfile(s) to manage")
        .value_name("DOTFILE")
        .multiple(true)
        .required(true);

    let list =
        SubCommand::with_name("list").about("List all installed dotfiles in the given store");

    let install = SubCommand::with_name("install")
        .about("Link dotfile(s) to target in home directory")
        .arg(&dotfiles);

    let uninstall = SubCommand::with_name("uninstall")
        .about("Unlink dotfile(s) to target in home directory")
        .arg(&dotfiles);

    let manage = SubCommand::with_name("manage")
        .about("Move dotfile(s) to the store and link them back to their target")
        .arg(&dotfiles);

    let unmanage = SubCommand::with_name("unmanage")
        .about("Move dotfile(s) out of the store and back to their target")
        .arg(&dotfiles);

    let matches = App::new("dots - Dotfile management made less toilesome.")
        .arg(&store)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::DisableVersion)
        .subcommands(vec![list, install, uninstall, manage, unmanage])
        .get_matches();

    let store = resolve_store(&matches)?;

    let command = match matches.subcommand() {
        ("list", _) => Command::List,
        ("install", Some(sub_m)) => Command::Install(resolve_dotfile(sub_m)?),
        ("uninstall", Some(sub_m)) => Command::Uninstall(resolve_dotfile(sub_m)?),
        ("manage", Some(sub_m)) => Command::Manage(resolve_dotfile(sub_m)?),
        ("unmanage", Some(sub_m)) => Command::Unmanage(resolve_dotfile(sub_m)?),
        _ => unreachable!(),
    };

    Ok(Args { store, command })
}

fn resolve_store(matches: &clap::ArgMatches) -> Result<Store> {
    let store_path = matches
        .value_of("store")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".dotfiles"));

    let store_path = resolve_path(&store_path)?;

    if store_path.is_dir() {
        Ok(Store::new(&store_path))
    } else {
        Err(format!("Path `{}` is not a directory.", store_path.display()).into())
    }
}

fn resolve_dotfile(matches: &clap::ArgMatches) -> Result<PathBuf> {
    let dotfile_path = matches.value_of("dotfile").map(PathBuf::from).unwrap();
    let dotfile_path = resolve_path(&dotfile_path)?;

    let is_reg_file = dotfile_path
        .metadata()
        .map(|m| m.is_file())
        .unwrap_or(false);

    if is_reg_file {
        Ok(dotfile_path)
    } else {
        Err(format!("Path `{}` is not a regular file.", dotfile_path.display()).into())
    }
}

fn resolve_path(path: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(path)
        .map_err(|_| format!("Path `{}` could not be resolved.", path.display()))?;

    Ok(path)
}
