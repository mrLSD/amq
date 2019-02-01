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
}

/// Client config - node for connection
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientNodeConfig {
    pub public_key: String,
    pub ip: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeAppConfig {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub port: u32,
}

/// Init Node app configuration
impl NodeAppConfig {
    pub fn new(cfg: &NodeConfig) -> Self {
        NodeAppConfig {
            public_key: sign::from_string_pk(&cfg.public_key),
            secret_key: sign::from_string_sk(&cfg.secret_key),
            port: cfg.port,
        }
    }
}
