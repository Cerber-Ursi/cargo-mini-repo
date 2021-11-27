fn main() {
    simplelog::SimpleLogger::init(log::LevelFilter::Info, Default::default()).expect("Failed to init logger");
    rouille::start_server(("127.0.0.1", 8888), cargo_mini_repo::handler);
}
