mod error;

extern crate clap;
extern crate dirs;
extern crate walkdir;

use clap::{App, AppSettings, Arg, SubCommand};
use walkdir::{DirEntry, WalkDir};

use std::fs;
use std::path::PathBuf;

use error::{Error, Result};

static INFO: &str = "dots - Dotfile management made less toilesome.";

fn main() {
    let list_command =
        SubCommand::with_name("list").about("List all installed dotfiles in the given store");

    let add_command = SubCommand::with_name("add")
        .about("Link dotfile to home directory")
        .arg(
            Arg::with_name("dotfile")
                .help("Dotfile to install")
                .required(true)
                .index(1),
        );

    let remove_command = SubCommand::with_name("remove")
        .about("Unlink dotfile from home directory")
        .arg(
            Arg::with_name("dotfile")
                .help("Dotfile to uninstall")
                .required(true)
                .index(1),
        );

    let matches = App::new(INFO)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(list_command)
        .subcommand(add_command)
        .subcommand(remove_command)
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

    let res = match matches.subcommand() {
        ("add", Some(sub_m)) => run_add_command(&store, sub_m),
        ("remove", Some(sub_m)) => run_remove_command(&store, sub_m),
        ("list", _) => run_list_command(&store),
        _ => unreachable!(),
    };

    if let Err(err) = res {
        println!("Error: {}.", err);
    }
}

fn run_add_command(store: &Store, matches: &clap::ArgMatches) -> Result<()> {
    let name = matches.value_of("dotfile").unwrap();
    let dotfile = store.get(name).ok_or(Error::DotfileNotFound)?;

    match dotfile.state() {
        DotfileState::Installed => Ok(()),
        DotfileState::Blocked => Err(Error::DotfileBlocked)?,
        DotfileState::Uninstalled => {
            if let Some(parent) = dotfile.target().parent() {
                fs::create_dir_all(parent)?;
            }
            std::os::unix::fs::symlink(dotfile.source(), dotfile.target())?;
            Ok(())
        }
    }
}

fn run_remove_command(store: &Store, matches: &clap::ArgMatches) -> Result<()> {
    let name = matches.value_of("dotfile").unwrap();
    let dotfile = store.get(name).ok_or(Error::DotfileNotFound)?;

    match dotfile.state() {
        DotfileState::Installed => {
            fs::remove_file(dotfile.target())?;
            Ok(())
        }
        DotfileState::Blocked => Err(Error::DotfileBlocked)?,
        DotfileState::Uninstalled => Ok(()),
    }
}

fn run_list_command(store: &Store) -> Result<()> {
    println!("Dotfiles from `{}`:", store.root_str());

    for dotfile in store.all() {
        println!(
            "[{}] {}",
            match dotfile.state() {
                DotfileState::Installed => "x",
                DotfileState::Blocked => "-",
                DotfileState::Uninstalled => " ",
            },
            dotfile.name_str(),
        );
    }

    Ok(())
}

struct Store {
    root: PathBuf,
    dotfiles: Vec<Dotfile>,
}

impl Store {
    fn new<P: Into<PathBuf>>(path: P) -> Store {
        let dot_root = path.into();
        let mut dotfiles: Vec<Dotfile> = WalkDir::new(dot_root.to_path_buf())
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .flat_map(|e| e)
            .filter(|e| e.file_type().is_file())
            .flat_map(|e| e.path().strip_prefix(&dot_root).map(|p| p.to_path_buf()))
            .map(|p| Dotfile::new(&dot_root, &p))
            .collect();
        dotfiles.sort();

        Store {
            root: dot_root,
            dotfiles,
        }
    }

    fn root(&self) -> PathBuf {
        self.root.clone()
    }

    fn root_str(&self) -> String {
        self.root().to_string_lossy().into_owned()
    }

    fn get<P: Into<PathBuf>>(&self, path: P) -> Option<&Dotfile> {
        let path = path.into();
        self.dotfiles.iter().find(|d| d.has_name(&path))
    }

    fn all(&self) -> impl Iterator<Item = &Dotfile> {
        self.dotfiles.iter()
    }
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "README.md")
        .unwrap_or(false)
}

#[derive(Debug)]
enum DotfileState {
    Installed,
    Uninstalled,
    Blocked,
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
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

    fn has_name<P: Into<PathBuf>>(&self, path: P) -> bool {
        let path = path.into();
        path == self.name() || path == self.source() || path == self.target()
    }

    fn name(&self) -> PathBuf {
        self.path.clone()
    }

    fn name_str(&self) -> String {
        self.name().to_string_lossy().into_owned()
    }

    fn source(&self) -> PathBuf {
        self.store_path.join(&self.path)
    }

    // fn source_str(&self) -> String {
    //     self.source().to_string_lossy().into_owned()
    // }

    fn target(&self) -> PathBuf {
        home_dir().join(PathBuf::from(format!(".{}", self.path.to_string_lossy())))
    }

    // fn target_str(&self) -> String {
    //     self.target().to_string_lossy().into_owned()
    // }

    fn state(&self) -> DotfileState {
        if !self.target().exists() {
            return DotfileState::Uninstalled;
        }

        match fs::read_link(self.target()) {
            Ok(linked) => if linked == self.source() {
                DotfileState::Installed
            } else {
                DotfileState::Blocked
            },
            _ => DotfileState::Blocked,
        }
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap()
}
