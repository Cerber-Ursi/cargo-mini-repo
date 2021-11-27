use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("I/O error while initializing context")]
    Io(#[from] std::io::Error),
    #[error("Bare index must be stored on path with valid UTF-8 name")]
    NonUTF8BarePath,
    #[error("File already exists on index path")]
    IndexIsFile,
    #[error("Git error while creating repository for index")]
    Git(#[from] git2::Error),
}

pub fn init(cfg: crate::Config) -> Result<(), InitError> {
    std::fs::create_dir_all(cfg.crates_root())?;

    let bare_root = std::fs::canonicalize(cfg.bare_repo_root())?;

    git2::Repository::init_opts(&bare_root, &git2::RepositoryInitOptions::new().bare(true))?;

    let bare_url = bare_root.to_str().ok_or(InitError::NonUTF8BarePath)?;
    if let Ok(dir) = std::fs::File::open(cfg.repo_root()) {
        if dir.metadata().expect("metadata").is_dir() {
            let repo = git2::Repository::open(cfg.repo_root())?;
            repo.remote_set_url("origin", bare_url)?;
        } else {
            return Err(InitError::IndexIsFile);
        }
    } else {
        git2::Repository::clone(bare_url, cfg.repo_root())?;
    }

    Ok(())
}
