use actix::prelude::*;

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
