use chrono::Local;
use serde::Deserialize;
use std::fs;
use tracing_subscriber;



#[derive(Deserialize, Clone)]
pub struct Config {
    pub ip: String,
    pub static_port: Option<u16>,
    pub name_for_port_manager: String,
    pub logs_dir: String,
    pub workers_count: usize
}



pub fn load_config() -> Config {
    let config_path = "config.json";
    let config_data = fs::read_to_string(config_path).expect("Can't read config.json");
    serde_json::from_str(&config_data).expect("Can't parse config.json")
}



pub fn init_tracing(logs_dir: &str, log_name: &str) {
    let date = Local::now().format("%d_%m_%Y").to_string();
    let log_dir = if logs_dir.trim().is_empty() {
        "./logs"
    } else {
        logs_dir
    };

    fs::create_dir_all(log_dir).expect("Can't create logs directory");

    let log_path = format!("{}/{}_{}.log", log_dir, log_name, date);

    tracing_subscriber::fmt()
        .with_target(true)
        .with_writer(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Can't open log file"),
        )
        .with_thread_names(true)
        .with_ansi(false)
        .init();
}
