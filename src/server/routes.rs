use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::Path,
};

use rouille::{Request, Response};
use sha2::Digest;
use thiserror::Error;

use crate::{config::RuntimeConfig, err_context::{ErrContext, ErrWithContext}};

use super::helpers::PublishedCrate;

#[derive(Debug, Error)]
enum PublishError {
    #[error("Unable to retrieve the data")]
    NoData,
    #[error("I/O error while publishing")]
    Io(#[from] ErrWithContext<std::io::Error>),
    #[error("Failed to parse JSON data")]
    BadJson(#[from] ErrWithContext<serde_json::Error>),
    #[error("Failed to commit changes to index")]
    Git(#[from] ErrWithContext<git2::Error>),
}

fn do_publish(cfg: &RuntimeConfig, req: &Request) -> Result<Response, PublishError> {
    let mut data = req.data().ok_or(PublishError::NoData)?;
    let mut size = [0u8; 4];
    data.read_exact(&mut size).context("Read JSON size")?;

    let mut json = vec![0u8; u32::from_le_bytes(size) as usize];
    data.read_exact(&mut json).context("Read JSON")?;
    let crate_info: PublishedCrate = serde_json::from_slice(&json).context("Parse JSON")?;

    data.read_exact(&mut size).context("Read tarball size")?;
    let mut tar = vec![0u8; u32::from_le_bytes(size) as usize];
    data.read_exact(&mut tar).context("Read tarball")?;
    let checksum = hex::encode(sha2::Sha256::digest(&tar));

    let tarball_path = cfg
        .crates_root()
        .join(&crate_info.name)
        .join(crate_info.vers.to_string());
    create_dir_all(&tarball_path).context("Create dir for tarball")?;
    File::create(tarball_path.join("archive.crate"))
        .context("Create file for tarball")?
        .write_all(&tar)
        .context("Write tarball")?;

    let crate_info = super::helpers::crate_to_package(crate_info, checksum);
    let crate_name = crate_info.name.to_lowercase();
    let crate_path = match crate_name.len() {
        1 => Path::new("1").to_path_buf(),
        2 => Path::new("2").to_path_buf(),
        3 => Path::new("3").join(&crate_name[..1]),
        _ => Path::new(&crate_name[0..2]).join(&crate_name[2..4]),
    };

    let crate_info = serde_json::to_string(&crate_info).context("Generate JSON")?;
    let crate_repo_path = cfg.repo_root().join(&crate_path);
    create_dir_all(&crate_repo_path).context("Create dir for crate data")?;
    let crate_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&crate_repo_path.join(&crate_name))
        .context("Open file for crate data")?;
    writeln!(&crate_file, "{}", crate_info).context("Append crate data")?;

    crate::git::push_existing(&cfg.lock_repo())?;

    Ok(Response::text(
        r#"{"warnings":{"invalid_categories":[],"invalid_badges":[],"other":[]}}"#,
    ))
}

pub fn publish(cfg: &RuntimeConfig, req: &Request) -> Response {
    match do_publish(cfg, req) {
        Ok(res) => res,
        Err(err) => error(crate::traceback(&err)),
    }
}

pub fn download(cfg: &RuntimeConfig, req: &Request) -> Response {
    if let Ok(file) = File::open(
        cfg.crates_root()
            .join(req.url().trim_start_matches('/'))
            .join("archive.crate"),
    ) {
        Response::from_file("application/tar", file)
    } else {
        Response::text("Crate not found").with_status_code(404)
    }
}

pub fn unknown() -> Response {
    error("Unexpected endpoint")
}

fn error(text: impl std::fmt::Display) -> Response {
    log::error!("{}", text);
    Response::text(format!(r#"{{"errors":[{{"detail":"{}"}}]}}"#, text))
}
