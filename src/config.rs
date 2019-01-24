use std::fs;
use std::env;

const DEFAULT_CONFIG_FILE: &str = "config.toml";

fn main() {
    let args = env::args();
    let config_file = if args.len() < 2 {
        args.nth(1).unwrap()
    } else {
        DEFAULT_CONFIG_FILE.to_string()
    };
    if let Err(err) = fs::write(config_file, "123") {
        eprintln!("Failed to create config file: {}", err);
    }
}
