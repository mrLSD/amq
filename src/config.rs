use crate::types::{ClientConfig, ClientNodeConfig, NodeConfig};
use std::env;
use std::fs;
use toml;

mod sign;
mod types;

/// Basic config types
enum AppConfigType {
    Node,
    Client,
}

/// Generate config data by specific ty[e
fn generate_config_date(config_type: AppConfigType) -> String {
    let (pk, sk) = sign::gen_keypair();

    match config_type {
        AppConfigType::Client => {
            let cfg = ClientConfig {
                public_key: sign::to_hex_pk(&pk),
                secret_key: sign::to_hex_sk(&sk),
                node: ClientNodeConfig {
                    public_key: sign::to_hex_pk(&pk),
                    ip: "0.0.0.0".to_string(),
                    port: 3030,
                },
            };
            toml::to_string_pretty(&cfg).unwrap()
        }
        AppConfigType::Node => {
            let cfg = NodeConfig {
                public_key: sign::to_hex_pk(&pk),
                secret_key: sign::to_hex_sk(&sk),
                port: 3030,
            };
            toml::to_string_pretty(&cfg).unwrap()
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
Actix MQ network config generator

Usage: config [COMMAND] [CONFIG_FILE]

Available commands:
    node        generate config for Server Node
    client      generate config for Client that
                can connect to specific Server Node
    help        print that help
    "#
    );
    std::process::exit(code);
}

fn main() {
    sign::init();

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
    let cfg_toml = generate_config_date(config_type);

    // Save config to file
    if let Err(err) = fs::write(config_file, cfg_toml) {
        eprintln!("Failed to create config file: {}", err);
    }
}
