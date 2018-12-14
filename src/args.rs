use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;

use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "dots - Dotfile management made less toilesome.",
    about = "",
    author = "",
    version = "",
    raw(global_setting = "AppSettings::DisableVersion")
)]
pub struct Opt {
    #[structopt(
        name = "store",
        help = "Path of dotfile store [default: ~/.dotfiles]",
        long = "store",
        raw(global = "true"),
        value_name = "PATH",
        parse(try_from_os_str = "resolve_dir")
    )]
    pub store: Option<PathBuf>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(
        name = "install",
        about = "Link dotfile(s) to target in home directory",
        author = "",
        version = ""
    )]
    Install {
        #[structopt(
            help = "Dotfile(s) to install",
            parse(try_from_os_str = "resolve_file"),
            raw(required = "true")
        )]
        dotfiles: Vec<PathBuf>,
    },

    #[structopt(
        name = "uninstall",
        about = "Unlink dotfile(s) from target in home directory",
        author = "",
        version = ""
    )]
    Uninstall {
        #[structopt(
            help = "Dotfile(s) to uninstall",
            parse(try_from_os_str = "resolve_file"),
            raw(required = "true")
        )]
        dotfiles: Vec<PathBuf>,
    },

    #[structopt(
        name = "manage",
        about = "Move dotfile(s) to the store and link them back to their target",
        author = "",
        version = ""
    )]
    Manage {
        #[structopt(
            help = "Dotfile(s) to manage",
            parse(try_from_os_str = "resolve_file"),
            raw(required = "true")
        )]
        dotfiles: Vec<PathBuf>,
    },

    #[structopt(
        name = "unmanage",
        about = "Move dotfile(s) out of the store and back to their target",
        author = "",
        version = ""
    )]
    Unmanage {
        #[structopt(
            help = "Dotfile(s) to unmanage",
            parse(try_from_os_str = "resolve_file"),
            raw(required = "true")
        )]
        dotfiles: Vec<PathBuf>,
    },

    #[structopt(
        name = "list",
        about = "List all installed dotfiles in the given store",
        author = "",
        version = ""
    )]
    List {},
}

fn resolve_path(path: &OsStr) -> Result<PathBuf, OsString> {
    let path = fs::canonicalize(path).map_err(|_| {
        OsString::from(format!(
            "Path `{}` could not be resolved.",
            path.to_string_lossy()
        ))
    })?;

    Ok(path)
}

fn resolve_dir(path: &OsStr) -> Result<PathBuf, OsString> {
    let path = resolve_path(path)?;

    if path.is_dir() {
        Ok(path)
    } else {
        Err(OsString::from(format!(
            "Path `{}` must be a directory.",
            path.display()
        )))
    }
}

fn resolve_file(path: &OsStr) -> Result<PathBuf, OsString> {
    let path = resolve_path(path)?;

    if path.is_file() {
        Ok(path)
    } else {
        Err(OsString::from(format!(
            "Path `{}` must be a file.",
            path.display()
        )))
    }
}
