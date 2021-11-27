mod config;
mod server;
mod initializer;
mod err_context;

use thiserror::Error;

pub use server::start;
pub use initializer::init;
pub use config::{Config, ConfigError};

use std::fmt::Write;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error(transparent)]
    Initializer(#[from] initializer::InitError),
    #[error(transparent)]
    Server(#[from] server::ServerError),
}

pub fn traceback(mut err: &dyn std::error::Error) -> String {
    let mut trace = String::new();
    loop {
        writeln!(&mut trace, "{}", err).expect("Error while reporting error");
        match err.source() {
            Some(source) => {
                err = source;
                writeln!(&mut trace, "Caused by:").expect("Error while reporting error");
            }
            None => return trace,
        }
    }
}
