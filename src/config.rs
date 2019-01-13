use serde_derive::Serialize;
use std::env;
use std::fs;
use toml;

const DEFAULT_CONFIG_FILE: &str = "config.toml";

#[derive(Serialize)]
pub struct AppConfig2 {
    pub name: String,
}

#[derive(Serialize)]
pub struct AppConfig {
    pub name: String,
    pub app: AppConfig2,
}

fn main() {
    let mut args = env::args();
    let config_file = if args.len() > 2 {
        args.nth(1).unwrap()
    } else {
        DEFAULT_CONFIG_FILE.to_string()
    };

    let cfg = AppConfig {
        name: "123".to_string(),
        app: AppConfig2 {
            name: "321".to_string(),
        },
    };
    let cfg_toml = toml::to_string(&cfg).unwrap();

    if let Err(err) = fs::write(config_file, &cfg_toml) {
        eprintln!("Failed to create config file: {}", err);
    }
}
