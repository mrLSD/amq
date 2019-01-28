use serde_derive::Serialize;
use sodiumoxide::crypto::sign::ed25519::{PublicKey, SecretKey};
use std::env;
use std::fs;
use toml;

mod sign;

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

    check_commands(&mut args);

    // Fetch config generation parameters
    let (config_type, config_file) = if args.len() == 3 {
        let config_type = match args.nth(1).unwrap().as_ref() {
            "node" => AppConfigType::Node,
            "client" => AppConfigType::Client,
            _ => {
                eprintln!(
                    "Wrong config type generator parameter.\nAvailable variants: node, client"
                );
                std::process::exit(1);
            }
        };
        (config_type, args.nth(2).unwrap())
    } else {
        // If config type not set - get only file name arg
        if args.len() == 2 {
            (DEFAULT_CONFIG_TYPE, args.nth(1).unwrap())
        } else {
            if args.len() > 3 {
                eprintln!(
                    "Wrong parameters count: {}\nAvailable 2 parameters",
                    args.len()
                );
                std::process::exit(1);
            }

            // All parameters are default
            (DEFAULT_CONFIG_TYPE, DEFAULT_CONFIG_FILE.to_string())
        }
    };

    // Generate config
    let cfg = generate_config_date(config_type);

    // Get TOML config data
    let cfg_toml = toml::to_string(&cfg).unwrap();

    // Save config to file
    if let Err(err) = fs::write(config_file, &cfg_toml) {
        eprintln!("Failed to create config file: {}", err);
    }
}

/// Generate config data by specific ty[e
fn generate_config_date(config_type: AppConfigType) -> String {
    let (pk, sk) = sign::gen_keypair();

    match config_type {
        AppConfigType::Client => {
            let cfg = ClientConfig {
                public_key: pk,
                secret_key: sk,
                node: ClientNodeConfig {
                    public_key: pk,
                    ip: "127,0,0,1".to_string(),
                    port: 3030,
                },
            };
            toml::to_string(&cfg).unwrap()
        }
        AppConfigType::Node => {
            let cfg = NodeConfig {
                public_key: pk,
                secret_key: sk,
                port: 3030,
            };
            toml::to_string(&cfg).unwrap()
        }
    }
}

/// Check command arguments
fn check_commands(args: &mut env::Args) {
    if args.len() != 3 {
        help_message();
        std::process::exit(1);
    }
    match args.nth(1).unwrap().as_ref() {
        "node" => AppConfigType::Node,
        "client" => AppConfigType::Client,
        _ => {
            help_message();
            std::process::exit(0);
        }
    };
}

/// Print help message for CLI commands
fn help_message() {
    println!(
        r#"
Active MQ network config generator

Usage: config [COMMAND] [FILE]

Available commands:
    node        generate config for Server Node
    client      generate config for Client that can connect to Node
    help        print that help
    "#
    );
}
