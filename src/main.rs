extern crate clap;
extern crate walkdir;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use walkdir::{DirEntry, WalkDir};

use std::path::PathBuf;
use std::{env, fs};

static INFO: &str = "dots - Dotfile management made less toilesome.";

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() -> Result<()> {
    let list_command = SubCommand::with_name("list")
        .about("List all installed dotfiles in the current directory")
        .arg(
            Arg::with_name("source")
                .help("Directory to list")
                .long("source")
                .takes_value(true),
        );

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
        Some("list") => list(matches.subcommand_matches("list").unwrap()),
        _ => Ok(()),
    }
}

fn add(matches: &ArgMatches) -> Result<()> {
    Ok(())
}

fn list(matches: &ArgMatches) -> Result<()> {
    let home_dir = env::home_dir().unwrap();
    let home_dir_str = home_dir.to_str().unwrap();

    let source_dir = match matches.value_of("source") {
        Some(dir) => PathBuf::from(dir),
        None => home_dir.join(".dotfiles"),
    };
    let source_dir_str = source_dir.to_str().unwrap();

    println!("Installed dotfiles from {}:", source_dir_str);

    WalkDir::new(&source_dir)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
        .filter_map(|e| e.ok())
        .for_each(|e| {
            let stripped_source_path = e.path()
                .strip_prefix(&source_dir)
                .unwrap()
                .to_str()
                .unwrap();

            let installed_path_str = format!("{}/.{}", home_dir_str, stripped_source_path);
            let installed_path = PathBuf::from(installed_path_str);

            if let Ok(source_path) = fs::read_link(installed_path) {
                if source_path == e.path() {
                    println!("- {}", stripped_source_path);
                }
            }
        });

    Ok(())
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}
