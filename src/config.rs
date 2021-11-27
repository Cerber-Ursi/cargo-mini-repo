use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use git2::Repository;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to parse config - invalid TOML")]
    InvalidTOML(#[from] toml::de::Error),
    #[error("Failed to open index repository")]
    GitError(#[from] git2::Error),
    #[error("IO error processing config")]
    Io(#[from] std::io::Error),
}

pub struct Config {
    port: u16,
    repo_path: PathBuf,
    repo: Arc<Mutex<Repository>>,
    crates: PathBuf,
}

#[derive(Deserialize)]
struct RawConfig {
    port: u16,
    repo: PathBuf,
    crates: PathBuf,
}

impl Config {
    pub fn from_toml(toml: &str) -> Result<Self, ConfigError> {
        let cfg: RawConfig = toml::from_str(toml)?;

        let _ = File::open(&cfg.crates)?;
        let repo = Arc::new(Mutex::new(Repository::open(&cfg.repo)?));
        Ok(Config {
            port: cfg.port,
            repo_path: cfg.repo,
            repo,
            crates: cfg.crates,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn lock_repo(&self) -> MutexGuard<Repository> {
        self.repo.lock().expect("Repository is poisoned")
    }
    pub fn crates_root(&self) -> &Path {
        &self.crates
    }
    pub fn repo_root(&self) -> &Path {
        &self.repo_path
    }
}
