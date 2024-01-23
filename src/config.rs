use std::path::Path;

use axum_extra::extract::cookie::Key;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

pub fn load() -> Config {
    let content = std::fs::read_to_string("config.toml").expect("failed to read config file");
    toml::from_str(&content).expect("invalid config")
}

pub fn load_key() -> Key {
    let path = Path::new("links.key");
    if path.exists() {
        let data = std::fs::read(path).expect("Failed to read links key");
        Key::from(&data)
    } else {
        let key = Key::generate();
        std::fs::write(path, key.master()).expect("Failed to write links key");
        key
    }
}
