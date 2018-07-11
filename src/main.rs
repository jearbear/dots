extern crate clap;
extern crate dirs;
extern crate walkdir;

use clap::{App, AppSettings, Arg, SubCommand};
use walkdir::{DirEntry, WalkDir};

use std::fs;
use std::path::PathBuf;

static INFO: &str = "dots - Dotfile management made less toilesome.";

type _Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() {
    let list_command =
        SubCommand::with_name("list").about("List all installed dotfiles in the given store");

    let _add_command = SubCommand::with_name("add")
        .about("Link dotfile to home directory")
        .arg(
            Arg::with_name("dotfile")
                .help("Dotfile to install")
                .required(true)
                .index(1),
        );

    let matches = App::new(INFO)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(list_command)
        // .subcommand(add_command)
        .arg(
            Arg::with_name("store")
                .help("Path of dotfile store [default: ~/.dotfiles]")
                .global(true)
                .long("store")
                .value_name("PATH"),
        )
        .get_matches();

    let dot_root = match matches.value_of("store") {
        Some(store_path) => PathBuf::from(store_path),
        None => home_dir().join(".dotfiles"),
    };

    let store = Store::new(&dot_root);

    match matches.subcommand_name() {
        Some("list") => {
            println!("Installed dotfiles from `{}`:", dot_root.to_string_lossy());

            for dotfile in store.installed() {
                println!("- {}", dotfile.path.to_string_lossy());
            }
        }

        _ => {}
    }
}

struct Store {
    dotfiles: Vec<Dotfile>,
}

impl Store {
    fn new<P: Into<PathBuf>>(path: P) -> Store {
        let dot_root = path.into();
        let dotfiles = WalkDir::new(dot_root.to_path_buf())
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .flat_map(|e| e)
            .flat_map(|e| e.path().strip_prefix(&dot_root).map(|p| p.to_path_buf()))
            .map(|p| Dotfile::new(&dot_root, &p))
            .collect();

        Store { dotfiles }
    }

    fn installed(&self) -> impl Iterator<Item = &Dotfile> {
        self.dotfiles.iter().filter(|d| d.installed())
    }
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}

struct Dotfile {
    store_path: PathBuf,
    path: PathBuf,
}

impl Dotfile {
    fn new<P: Into<PathBuf>>(store_path: P, path: P) -> Dotfile {
        Dotfile {
            store_path: store_path.into(),
            path: path.into(),
        }
    }

    fn source(&self) -> PathBuf {
        self.store_path.join(&self.path)
    }

    fn target(&self) -> PathBuf {
        home_dir().join(PathBuf::from(format!(".{}", self.path.to_string_lossy())))
    }

    fn installed(&self) -> bool {
        match fs::read_link(self.target()) {
            Ok(linked) => linked == self.source(),
            _ => false,
        }
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap()
}
