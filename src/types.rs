use serde_derive::Serialize;

/// Basic Node configuration
#[derive(Serialize)]
pub struct NodeConfig {
    pub public_key: String,
    pub secret_key: String,
    pub port: u32,
}

/// Basic client config
#[derive(Serialize)]
pub struct ClientConfig {
    pub public_key: String,
    pub secret_key: String,
    pub node: ClientNodeConfig,
}

/// Client config - node for connection
#[derive(Serialize)]
pub struct ClientNodeConfig {
    pub public_key: String,
    pub ip: String,
    pub port: u32,
}
