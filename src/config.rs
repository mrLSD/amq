use serde_derive::Serialize;
use sodiumoxide::crypto::{
    sign::ed25519,
    sign::ed25519::{PublicKey, SecretKey, Seed, Signature},
};
use std::env;
use std::fs;
use toml;

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const DEFAULT_CONFIG_TYPE: AppConfigType = AppConfigType::Client;

/// Basic config types
enum AppConfigType {
    Node,
    Client,
}

/// Basic client config
#[derive(Serialize)]
pub struct ClientConfig {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub node: ClientNodeConfig,
}

/// Client config - node for connection
#[derive(Serialize)]
pub struct ClientNodeConfig {
    pub public_key: PublicKey,
    pub ip: String,
    pub port: u32,
}

/// Basic Node configuration
#[derive(Serialize)]
pub struct NodeConfig {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub port: u32,
}

fn main() {
    let mut args = env::args();

    // Fetch config generation parameters
    let (config_type, config_file) = if args.len() > 3 {
        let config_type = match args.nth(1).unwrap().as_ref() {
            "node" => AppConfigType::Node,
            _ => AppConfigType::Client,
        };
        (config_type, args.nth(2).unwrap())
    } else {
        // If config type not set - get only file name arg
        if args.len() > 2 {
            (DEFAULT_CONFIG_TYPE, args.nth(1).unwrap())
        } else {
            // All parameters are default
            (DEFAULT_CONFIG_TYPE, DEFAULT_CONFIG_FILE.to_string())
        }
    };

    // Generate config
    let cfg = match config_type {
        AppConfigType::Client => {}
        AppConfigType::Node => {}
    };

    // Get TOML config data
    let cfg_toml = toml::to_string(&cfg).unwrap();

    // Save config to file
    if let Err(err) = fs::write(config_file, &cfg_toml) {
        eprintln!("Failed to create config file: {}", err);
    }
}
