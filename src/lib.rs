mod config;

use std::fs::File;

use rouille::{Request, Response};

pub fn handler(req: &Request) -> Response {
    let req = req.remove_prefix("/api/v1/crates").expect("Cargo, please!");
    log::info!("{:?}", req);
    Response::text("")
}