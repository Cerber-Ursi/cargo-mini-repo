use std::{io, path::PathBuf};

use cargo_mini_repo::{CommandError, Config, ConfigError};
use log::SetLoggerError;
use structopt::{StructOpt, clap};
use thiserror::Error;

#[derive(Debug, Error)]
enum MiniRepoError {
    #[error("Failed to open config file {0}")]
    NoConfigFile(String, #[source] io::Error),
    #[error("Failed to parse config")]
    InvalidConfig(#[from] ConfigError),
    #[error("Failed to init logger")]
    LoggerFailed(#[from] SetLoggerError),
    #[error("Error executing subcommand")]
    CommandError(#[from] CommandError),
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long)]
    /// Path to the config.toml file.
    cfg: PathBuf,
}

#[derive(StructOpt)]
#[structopt(about = "Mini-repository for Cargo, intended for local usage")]
enum Command {
    /// Initialize the local repository, according to the provided configuration
    Init(Args),
    /// Launch the repository server, listening on localhost
    Start(Args),
}

fn launch<E: Into<CommandError>>(
    args: Args,
    cmd: impl FnOnce(Config) -> Result<(), E>,
) -> Result<(), MiniRepoError> {
    let cfg_toml = std::fs::read_to_string(&args.cfg)
        .map_err(|e| MiniRepoError::NoConfigFile(args.cfg.to_string_lossy().to_string(), e))?;
    let cfg = Config::from_toml(&cfg_toml)?;
    simplelog::SimpleLogger::init(log::LevelFilter::Info, Default::default())?;
    cmd(cfg).map_err(|e| MiniRepoError::CommandError(e.into()))
}

fn main() {
    if let Err(error) = match Command::from_args() {
        Command::Init(args) => launch(args, cargo_mini_repo::init),
        Command::Start(args) => launch(args, cargo_mini_repo::start),
    } {
        clap::Error::with_description(
            &cargo_mini_repo::traceback(&error),
            clap::ErrorKind::ValueValidation,
        )
        .exit();
    }
}
