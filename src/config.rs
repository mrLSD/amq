use serde_derive::Serialize;
use sodiumoxide::crypto::sign::ed25519::{PublicKey, SecretKey};
use std::env;
use std::fs;
use toml;

use std::io::Write;


mod sign;

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
    check_commands();
    let mut args = env::args();

    // Fetch config generation parameters
    let (config_type, config_file) = {
        let config_type = match args.nth(1).unwrap().as_ref() {
            "node" => AppConfigType::Node,
            "client" => AppConfigType::Client,
            _ => {
                panic!("Failed to fetch arguments");
            }
        };
        (config_type, args.nth(0).unwrap())
    };

    // Generate config
    let cfg = generate_config_date(config_type);

    // Get TOML config data
    let cfg_toml = toml::to_string_pretty(&cfg).unwrap();

    println!("{}", cfg_toml);

    // Save config to file
//    if let Err(err) = fs::write(config_file, cfg_toml) {
//        eprintln!("Failed to create config file: {}", err);
//    }

    let mut file = std::fs::File::create("1.toml").unwrap();
    file.write_all(cfg_toml.as_bytes()).expect("Could not write to file!");
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
fn check_commands() {
    let mut args = env::args();
    if args.len() != 3 {
        help_message(1);
    }
    match args.nth(1).unwrap().as_ref() {
        "node" => return,
        "client" => return,
        "help" => {
            help_message(0);
        }
        _ => {
            help_message(1);
        }
    };
}

/// Print help message for CLI commands
fn help_message(code: i32) {
    println!(
        r#"
Active MQ network config generator

Usage: config [COMMAND] [FILE]

Available commands:
    node        generate config for Server Node
    client      generate config for Client that
                can connect to specific Server Node
    help        print that help
    "#
    );
    std::process::exit(code);
}
