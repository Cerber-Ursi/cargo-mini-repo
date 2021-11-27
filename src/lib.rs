mod config;
mod err_context;
mod helpers;
mod routes;

use std::io;

use log::SetLoggerError;
use rouille::{Request, Response};
use thiserror::Error;

fn handler(cfg: config::Config) -> impl Fn(&Request) -> Response + 'static {
    move |req| {
        if let Some(req) = req.remove_prefix("/api/v1/crates") {
            if let Some(req) = req.remove_prefix("/download") {
                if req.method() == "GET" {
                    routes::download(&cfg, &req)
                } else {
                    routes::unknown()
                }
            } else if let Some(req) = req.remove_prefix("/new") {
                if req.method() == "PUT" {
                    routes::publish(&cfg, &req)
                } else {
                    routes::unknown()
                }
            } else {
                routes::unknown()
            }
        } else {
            routes::unknown()
        }
    }
}

#[derive(Debug, Error)]
pub enum MiniRepoError {
    #[error("Expected path to config file as argument")]
    NoConfigArg,
    #[error("Failed to open config file {0}")]
    NoConfigFile(String, #[source] io::Error),
    #[error("Failed to parse config")]
    InvalidConfig(#[from] config::ConfigError),
    #[error("Failed to init logger")]
    LoggerFailed(#[from] SetLoggerError),
}

fn do_start() -> Result<(), MiniRepoError> {
    let cfg_path = std::env::args_os()
        .nth(1)
        .ok_or(MiniRepoError::NoConfigArg)?;
    let cfg_toml = std::fs::read_to_string(&cfg_path)
        .map_err(|e| MiniRepoError::NoConfigFile(cfg_path.to_string_lossy().to_string(), e))?;
    let cfg = config::Config::from_toml(&cfg_toml)?;
    simplelog::SimpleLogger::init(log::LevelFilter::Info, Default::default())?;
    rouille::start_server(("127.0.0.1", cfg.port()), handler(cfg))
}

pub fn start() {
    if let Err(e) = do_start() {
        traceback(std::io::stderr(), &e);
        std::process::exit(2);
    }
}

fn traceback(mut w: impl std::io::Write, mut err: &dyn std::error::Error) {
    loop {
        writeln!(w, "{}", err).expect("Error while reporting error");
        match err.source() {
            Some(source) => {
                err = source;
                writeln!(w, "Caused by:").expect("Error while reporting error");
            }
            None => return,
        }
    }
}
