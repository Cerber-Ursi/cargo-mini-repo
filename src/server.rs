
mod helpers;
mod routes;

use rouille::{Request, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to evaluate config")]
    Config(#[from] crate::config::ConfigError)
}

fn handler(cfg: super::config::RuntimeConfig) -> impl Fn(&Request) -> Response + 'static {
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

pub fn start(cfg: crate::Config) -> Result<(), ServerError> {
    rouille::start_server(("127.0.0.1", cfg.port()), handler(cfg.into_runtime()?))
}