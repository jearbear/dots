use dirs;
use walkdir::{DirEntry, WalkDir};

use std::fs;
use std::path::PathBuf;

pub struct Store {
    root: PathBuf,
    dotfiles: Vec<Dotfile>,
}

impl Store {
    pub fn new<P: Into<PathBuf>>(path: P) -> Store {
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

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Option<&Dotfile> {
        let path = path.into();
        self.dotfiles.iter().find(|d| d.has_name(&path))
    }

    pub fn all(&self) -> impl Iterator<Item = &Dotfile> {
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
pub enum DotfileState {
    Installed,
    Uninstalled,
    Blocked,
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Dotfile {
    store_path: PathBuf,
    path: PathBuf,
}

impl Dotfile {
    pub fn new<P: Into<PathBuf>>(store_path: P, path: P) -> Dotfile {
        Dotfile {
            store_path: store_path.into(),
            path: path.into(),
        }
    }

    pub fn has_name<P: Into<PathBuf>>(&self, path: P) -> bool {
        let path = path.into();
        path == self.name() || path == self.source() || path == self.target()
    }

    pub fn name(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn source(&self) -> PathBuf {
        self.store_path.join(&self.path)
    }

    pub fn target(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap();
        home.join(PathBuf::from(format!(".{}", self.path.to_string_lossy())))
    }

    pub fn state(&self) -> DotfileState {
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
