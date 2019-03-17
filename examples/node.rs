use amq::node::MqNode;
use amq::types::NodeConfig;

/// Check command arguments
fn check_commands() {
    let args = std::env::args();
    if args.len() != 2 {
        help_message(1);
    }
}

/// Print help message for CLI commands
fn help_message(code: i32) {
    println!(
        r#"
Actix MQ network Node

Usage: node [CONFIG_FILE]
    "#
    );
    std::process::exit(code);
}

/// Read config data form TOML file
fn read_config() -> NodeConfig {
    let mut args = std::env::args();
    let config_file = args.nth(1).unwrap();

    let config_data = std::fs::read_to_string(config_file).expect("File not found");
    toml::from_str(&config_data).expect("Failed to parse config file")
}

fn main() {
    check_commands();
    let node_config = read_config();
    MqNode::new(&node_config).serve();
}
