use std::io::Write;

use thiserror::Error;

use crate::err_context::ErrContext;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("I/O error while initializing context")]
    Io(#[from] std::io::Error),
    #[error("Bare index must be stored on path with valid UTF-8 name")]
    NonUTF8BarePath,
    #[error("File already exists on index path")]
    IndexIsFile,
    #[error("Git error while creating repository for index")]
    Git(#[from] crate::err_context::ErrWithContext<git2::Error>),
}

pub fn init(cfg: crate::Config) -> Result<(), InitError> {
    std::fs::create_dir_all(cfg.crates_root())?;

    git2::Repository::init_opts(
        cfg.bare_repo_root(),
        git2::RepositoryInitOptions::new()
            .bare(true)
            .initial_head("master"),
    )
    .context("Init bare")?;

    let bare_root = std::fs::canonicalize(cfg.bare_repo_root())?;
    let bare_url = bare_root.to_str().ok_or(InitError::NonUTF8BarePath)?;
    let repo = if let Ok(dir) = std::fs::File::open(cfg.repo_root()) {
        if dir.metadata().expect("metadata").is_dir() {
            let repo = git2::Repository::open(cfg.repo_root()).context("Open repo")?;
            repo.remote_set_url("origin", bare_url)
                .context("Set remote URL")?;
            repo
        } else {
            return Err(InitError::IndexIsFile);
        }
    } else {
        git2::Repository::clone(bare_url, cfg.repo_root()).context("Clone")?
    };

    let config_json = std::fs::File::create(cfg.repo_root().join("config.json"))?;
    write!(
        &config_json,
        r#"{{
    "dl":"http://localhost:{0}/api/v1/crates/download/{{crate}}/{{version}}",
    "api": "http://localhost:{0}"
}}"#,
        cfg.port()
    )?;
    crate::git::commit_and_push(&repo, &[])?;

    Ok(())
}
