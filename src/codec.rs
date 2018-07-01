use serde_json as json;
use tokio_io::codec::{Decoder, Encoder};

/// Codec for Client -> Server transport
pub struct ChatCodec;

/// Client request
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum ChatRequest {
    /// List rooms
    List,
    /// Join rooms
    Join(String),
    /// Send message
    Message(String),
    /// Ping
    Ping,
}
