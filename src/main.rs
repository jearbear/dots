extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate walkdir;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use walkdir::{DirEntry, WalkDir};

use std::path::PathBuf;
use std::{env, fs};

static INFO: &str = "dots - Dotfile management made less toilesome.";

lazy_static! {
    static ref HOME_DIR: PathBuf = env::home_dir().unwrap();
    static ref DOT_ROOT: PathBuf = HOME_DIR.join(".dotfiles");
}

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() -> Result<()> {
    let list_command =
        SubCommand::with_name("list").about("List all installed dotfiles in the current directory");

    let add_command = SubCommand::with_name("add")
        .about("Link dotfile to home directory")
        .arg(
            Arg::with_name("dotfile")
                .help("Dotfile to install")
                .required(true)
                .index(1),
        );

    let matches = App::new(INFO)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(list_command)
        .subcommand(add_command)
        .get_matches();

    match matches.subcommand_name() {
        Some("add") => add(matches.subcommand_matches("add").unwrap()),
        Some("list") => list(),
        _ => Ok(()),
    }
}

fn add(_matches: &ArgMatches) -> Result<()> {
    Ok(())
}

fn list() -> Result<()> {
    println!("Installed dotfiles from {}:", DOT_ROOT.to_string_lossy());

    let entries = WalkDir::new(DOT_ROOT.to_path_buf())
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
        .filter_map(|e| e.ok());

    for e in entries {
        let dotfile = Dotfile::new(e.path())?;
        if dotfile.installed() {
            println!("- {}", dotfile.path_str());
        }
    }

    Ok(())
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}

struct Dotfile {
    path: PathBuf,
}

impl Dotfile {
    fn new<P: Into<PathBuf>>(path: P) -> Result<Dotfile> {
        Ok(Dotfile {
            path: path.into()
                .strip_prefix(DOT_ROOT.to_path_buf())?
                .to_path_buf(),
        })
    }

    fn path_str(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    fn source(&self) -> PathBuf {
        DOT_ROOT.join(&self.path)
    }

    fn target(&self) -> PathBuf {
        HOME_DIR.join(PathBuf::from(format!(".{}", self.path.to_string_lossy())))
    }

    fn installed(&self) -> bool {
        match fs::read_link(self.target()) {
            Ok(linked) => linked == self.source(),
            _ => false,
        }
    }
}
