use dirs;
use walkdir::{DirEntry, WalkDir};

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

pub struct Store {
    pub path: PathBuf,
    dotfiles: Vec<Dotfile>,
}

impl Store {
    pub fn new(path: &Path) -> Store {
        let path = path.to_path_buf();

        let mut dotfiles: Vec<Dotfile> = WalkDir::new(&path)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .flat_map(|e| e)
            .filter(|e| e.file_type().is_file())
            .flat_map(|e| Dotfile::from_source(&path, &e.path()))
            .collect();
        dotfiles.sort();

        Store { path, dotfiles }
    }

    pub fn get(&self, path: &Path) -> Option<&Dotfile> {
        self.dotfiles.iter().find(|d| d.referenced_by(path))
    }

    pub fn add(&mut self, dotfile: Dotfile) {
        self.dotfiles.push(dotfile);
        self.dotfiles.sort();
    }

    pub fn remove(&mut self, path: &Path) {
        self.dotfiles.retain(|df| !df.referenced_by(path))
    }

    pub fn all(&self) -> impl Iterator<Item = &Dotfile> {
        self.dotfiles.iter()
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Dotfile {
    pub name: PathBuf,
    pub source: PathBuf,
    pub target: PathBuf,
}

impl Dotfile {
    fn from_source(store_path: &Path, source: &Path) -> Result<Self> {
        let home_dir = dirs::home_dir().unwrap();
        let name = source.strip_prefix(store_path)?;

        Ok(Self {
            name: name.to_path_buf(),
            source: source.to_path_buf(),
            target: home_dir.join(format!(".{}", name.display())),
        })
    }

    pub fn from_target(store_path: &Path, target: &Path) -> Result<Self> {
        let home_dir = dirs::home_dir().unwrap();

        if target.starts_with(store_path) {
            return AppError::result("Target cannot be in store path.");
        }

        let stripped = target.strip_prefix(home_dir)?;
        if !stripped.to_string_lossy().starts_with('.') {
            return AppError::result("Target must be a dotfile.");
        }
        let name = PathBuf::from(stripped.to_string_lossy().trim_start_matches('.'));

        Ok(Self {
            source: store_path.join(&name),
            name,
            target: target.to_path_buf(),
        })
    }

    fn referenced_by(&self, reference: &Path) -> bool {
        self.name == reference || self.source == reference || self.target == reference
    }

    pub fn install(&self) -> Result<()> {
        create_parent_dirs(&self.target)?;
        symlink(&self.source, &self.target)?;
        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        fs::remove_file(&self.target)?;
        Ok(())
    }

    pub fn store(&self) -> Result<()> {
        create_parent_dirs(&self.source)?;
        fs::rename(&self.target, &self.source)?;
        symlink(&self.source, &self.target)?;
        Ok(())
    }

    pub fn unstore(&self) -> Result<()> {
        create_parent_dirs(&self.target)?;
        fs::rename(&self.source, &self.target)?;
        Ok(())
    }

    pub fn state(&self) -> DFState {
        if self.target.exists() {
            match fs::read_link(&self.target) {
                Ok(linked) => {
                    if linked == self.source {
                        DFState::Installed
                    } else {
                        DFState::Blocked
                    }
                }
                _ => DFState::Blocked,
            }
        } else {
            DFState::Uninstalled
        }
    }
}

pub enum DFState {
    Installed,
    Uninstalled,
    Blocked,
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "README.md")
        .unwrap_or(false)
}

fn create_parent_dirs(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
