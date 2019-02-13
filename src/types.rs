use crate::sign;
use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign::ed25519::{PublicKey, SecretKey};

/// Basic Node configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeConfig {
    pub public_key: String,
    pub secret_key: String,
    pub port: u32,
}

/// Basic client config
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    pub public_key: String,
    pub secret_key: String,
    pub node: ClientNodeConfig,
    pub message: ClientMessageConfig,
}

/// Client config - node for connection
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientNodeConfig {
    pub public_key: String,
    pub ip: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientMessageConfig {
    /// Should message be crypto sign
    pub sign: bool,
    /// Should message be encoded with crypto keys
    pub encode: bool,
}

/// Node app config struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeAppConfig {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub port: u32,
}

/// Client app config struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientAppConfig {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub node: ClientAppNodeConfig,
    pub message: ClientMessageConfig,
}

/// Client app config - node for connection
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientAppNodeConfig {
    pub public_key: PublicKey,
    pub ip: String,
    pub port: u32,
}

/// Init Node app configuration
#[allow(dead_code)]
impl NodeAppConfig {
    pub fn new(cfg: &NodeConfig) -> Self {
        NodeAppConfig {
            public_key: sign::from_string_pk(&cfg.public_key),
            secret_key: sign::from_string_sk(&cfg.secret_key),
            port: cfg.port,
        }
    }
}

/// Init Client app configuration
#[allow(dead_code)]
impl ClientAppConfig {
    pub fn new(cfg: &ClientConfig) -> Self {
        ClientAppConfig {
            public_key: sign::from_string_pk(&cfg.public_key),
            secret_key: sign::from_string_sk(&cfg.secret_key),
            node: ClientAppNodeConfig {
                public_key: sign::from_string_pk(&cfg.node.public_key),
                ip: cfg.node.ip.clone(),
                port: cfg.node.port,
            },
            message: ClientMessageConfig {
                sign: true,
                encode: false,
            },
        }
    }
}
