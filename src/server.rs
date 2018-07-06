use actix::prelude::*;

/// `MqServer` manages MQ network and
/// responsible for network nodes
/// coordinating.
pub struct MqServer {
    sessions: HashMap<usize, Addr<session::ChatSession>>,
    nodes: HashMap<String, HashSet<usize>>,
}
