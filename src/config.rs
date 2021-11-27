use std::{ops::Deref, fs::File, path::{Path, PathBuf}, sync::{Arc, Mutex, MutexGuard}};

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
    #[error("File already exists on index path")]
    IndexIsFile,
}

pub struct RuntimeConfig {
    cfg: Config,
    repo: Arc<Mutex<Repository>>,
}

#[derive(Deserialize)]
pub struct Config {
    port: u16,
    bare_repo: PathBuf,
    repo: PathBuf,
    crates: PathBuf,
}

impl Config {
    pub fn from_toml(toml: &str) -> Result<Self, ConfigError> {
        Ok(toml::from_str(toml)?)
    }

    pub fn into_runtime(self) -> Result<RuntimeConfig, ConfigError> {
        // Sanity check - to be sure that the directory for crates is here
        let crates = File::open(&self.crates)?;
        if !crates.metadata().expect("metadata").is_dir() {
            return Err(ConfigError::IndexIsFile);
        }
        
        let repo = Repository::open(&self.repo)?;
        // Sanity check - to be sure that the necessary "remote" is reachable
        repo.find_remote("origin")?.fetch_refspecs()?;
        
        Ok(RuntimeConfig {
            cfg: self,
            repo: Arc::new(Mutex::new(repo)),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn crates_root(&self) -> &Path {
        &self.crates
    }
    pub fn repo_root(&self) -> &Path {
        &self.repo
    }
    pub fn bare_repo_root(&self) -> &Path {
        &self.bare_repo
    }
}

impl RuntimeConfig {
    pub fn lock_repo(&self) -> MutexGuard<Repository> {
        self.repo.lock().expect("Repository is poisoned")
    }
}

impl Deref for RuntimeConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.cfg
    }
}