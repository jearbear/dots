use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use clap::Clap;
use lazy_static::lazy_static;
use path_clean::PathClean;
use walkdir::{DirEntry, WalkDir};

lazy_static! {
    static ref HOME_DIR: PathBuf = dirs::home_dir().expect("Couldn't obtain home directory");
    static ref DEFAULT_DIR: PathBuf = HOME_DIR.join(".dotfiles");
    static ref CUR_DIR: PathBuf = env::current_dir().expect("Couldn't obtain current directory");
}

/// A small program that helps you manage your dotfiles.
#[derive(Clap, Debug)]
struct Opts {
    /// Directory to use as the dotfile store.
    #[clap(short, long, default_value = DEFAULT_DIR.to_str().unwrap(), parse(try_from_os_str = parse_path))]
    store_dir: PathBuf,

    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// List the status of all dotfiles in the store.
    List,

    /// Given a path to a file in the home directory, add it to your dotfile store. This will move
    /// the original file and replace the old location with a symlink into its new location in the
    /// dotfile store.
    Add {
        #[clap(parse(try_from_os_str = parse_path))]
        target: PathBuf,
    },

    /// Given a path to a file in the home directory, remove it from your dotfile store. This will
    /// replace the symlink in the home directory with the original file from the dotfile store.
    Remove {
        #[clap(parse(try_from_os_str = parse_path))]
        target: PathBuf,
    },

    /// Given a path to a file in the dotfile store, create a symlink to it in the home directory.
    Link {
        #[clap(parse(try_from_os_str = parse_path))]
        source: PathBuf,
    },

    /// If given a path to a file in the dotfile store, remove the file in the home directory that
    /// links to it. If given a path to a file in the home directory, remove that file, assuming it
    /// links back to a file in the dotfile store.
    Unlink {
        #[clap(parse(try_from_os_str = parse_path))]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.command {
        SubCommand::List => {
            eprintln!("Listing dotfiles from {}", opts.store_dir.display());
            eprintln!("Legend: [x] installed, [-] blocked, [ ] uninstalled");
            eprintln!("---------------------------------------------------");

            WalkDir::new(&opts.store_dir)
                .min_depth(1)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_entry(|e| !is_ignored(e))
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .for_each(|e| {
                    let source = e.path();
                    // We're listing from the store path so this should never fail.
                    let name = source.strip_prefix(&opts.store_dir).unwrap();
                    let target = HOME_DIR.join(prepend_dot(name));

                    match target.read_link() {
                        Ok(s) if s == source => println!("[x] {}", name.display()),
                        Ok(_) => println!("[-] {}", name.display()),
                        Err(_) => println!("[ ] {}", name.display()),
                    }
                });

            Ok(())
        }

        SubCommand::Add { target } => {
            if let Ok(source) = target.read_link() {
                ensure!(
                    source.starts_with(&opts.store_dir),
                    "Target path is already a symlink to another file: `{}`",
                    source.display()
                );
                return Ok(());
            }

            ensure!(!target.is_dir(), "Target path must be a regular file");
            ensure!(
                !target.starts_with(&opts.store_dir),
                "Target path cannot be in the dotfile store"
            );

            let name = target
                .strip_prefix(&*HOME_DIR)
                .context("Target path must be in the user's home directory")?;

            ensure!(
                name.to_string_lossy().starts_with('.'),
                "Target path must be a dotfile"
            );

            let name = PathBuf::from(name.to_string_lossy().trim_start_matches('.'));
            let source = opts.store_dir.join(&name);

            ensure!(
                !source.exists(),
                "File already exists in dotfile store with source path: `{}`",
                source.display()
            );

            if let Some(parent_dir) = source.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            fs::rename(&target, &source)?;
            symlink(&source, &target)?;

            Ok(())
        }

        SubCommand::Remove { target } => {
            let err_msg = "Target path must be a symlink to a file in the dotfile store";

            let source = target.read_link().context(err_msg)?;
            ensure!(source.starts_with(&opts.store_dir), err_msg);

            fs::remove_file(&target)?;
            fs::rename(&source, &target)?;

            Ok(())
        }

        SubCommand::Link { source } => {
            ensure!(
                source.starts_with(&opts.store_dir),
                "Source path must be in the dotfile store"
            );

            let name = source.strip_prefix(&opts.store_dir)?;
            let target = HOME_DIR.join(prepend_dot(name));

            if target.exists() {
                match target.read_link() {
                    Ok(s) if s == source => return Ok(()),
                    _ => bail!(
                        "Target path (`{}`) blocked by an existing file",
                        target.display()
                    ),
                }
            }

            if let Some(parent_dir) = target.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            symlink(&source, &target)?;

            Ok(())
        }

        SubCommand::Unlink { path } => {
            if path.starts_with(&opts.store_dir) {
                let source = path; // For clarity
                let name = source.strip_prefix(&opts.store_dir)?;
                let target = HOME_DIR.join(prepend_dot(name));

                if !target.exists() {
                    return Ok(());
                }

                match target.read_link() {
                    Ok(s) if s == source => {
                        fs::remove_file(target)?;
                    }
                    _ => bail!(
                        "The derived target (`{}`) exists, but does not link back to the given path",
                        target.display()
                    ),
                };
            } else if path.starts_with(&*HOME_DIR) {
                // Not much sanity checking is done here. If you have a symlink into your dotfile
                // directory that's not a dotfile, you probably know what you're doing.
                match path.read_link() {
                    Ok(s) if s.starts_with(&opts.store_dir) => {
                        fs::remove_file(path)?;
                    }
                    _ => bail!("Given path is not a symlink to a file in the dotfile store"),
                };
            } else {
                bail!("Path must be either in the store directory or home directory");
            }

            Ok(())
        }
    }
}

fn is_ignored(entry: &DirEntry) -> bool {
    match entry.file_name().to_str() {
        Some(name) => name.starts_with('.') || name.starts_with("README"),
        None => false,
    }
}

fn prepend_dot(path: &Path) -> PathBuf {
    let mut res = OsString::from(".");
    res.push(path);
    PathBuf::from(res)
}

fn parse_path(path: &OsStr) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Given path does not exist".into());
    }
    if path.is_absolute() {
        Ok(path.clean())
    } else {
        Ok(CUR_DIR.join(path).clean())
    }
}
