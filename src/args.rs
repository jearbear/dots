use std::path::PathBuf;
use structopt::clap::AppSettings;

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
        parse(from_os_str)
    )]
    pub store: Option<PathBuf>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(
        name = "add", about = "Link dotfile to target in home directory", author = "", version = ""
    )]
    Add {
        #[structopt(help = "Dotfile to install", parse(from_os_str))]
        dotfile: PathBuf,
    },

    #[structopt(
        name = "remove",
        about = "Unlink dotfile from target in home directory",
        author = "",
        version = ""
    )]
    Remove {
        #[structopt(help = "Dotfile to uninstall", parse(from_os_str))]
        dotfile: PathBuf,
    },

    #[structopt(
        name = "list",
        about = "List all installed dotfiles in the given store",
        author = "",
        version = ""
    )]
    List {},
}