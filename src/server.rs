use actix::prelude::*;
use session;

/// `MqServer` manages MQ network and
/// responsible for network nodes
/// coordinating.
pub struct MqServer;

impl Default for MqServer {
    fn default() -> MqServer {
        MqServer {}
    }
}

/// Make actor from `MqServer`
impl Actor for MqServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Message for chat server communications

/// New MQ session is created
pub struct Connect {
    pub addr: Addr<session::MqSession>,
}

/// Response type for Connect message
///
/// MQ server returns unique session id
impl actix::Message for Connect {
    type Result = u64;
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: u64,
}

/// Send message to specific node
#[derive(Message)]
pub struct Message;