use actix::prelude::*;
use rand::{self, Rng};
use session;
use std::collections::HashMap;

/// `MqServer` manages MQ network and
/// responsible for network nodes
/// coordinating.
pub struct MqServer {
    sessions: HashMap<u64, Addr<session::MqSession>>,
}

impl Default for MqServer {
    fn default() -> MqServer {
        MqServer {
            sessions: HashMap::new(),
        }
    }
}

/// Make actor from `MqServer`
impl Actor for MqServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Message for MQ server communications

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
pub struct MqMessage {
    /// Id of the client session
    pub id: u64,
    /// Peer message
    pub msg: String,
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for MqServer {
    type Result = u64;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // register session with random id
        let id = rand::thread_rng().gen::<u64>();
        self.sessions.insert(id, msg.addr);

        println!("Handler<Connect> | id: {:?}", id);

        // Return ID
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Handler<Disconnect> | id: {:#?}", msg.id);

        // remove address
        self.sessions.remove(&msg.id);
    }
}

/// Handler for Message message.
impl Handler<MqMessage> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqMessage, _: &mut Context<Self>) {
        println!("Handler<Message>");
        if let Some(addr) = self.sessions.get(&msg.id) {
            let message: String = format!("Response message for: {:?}", msg.msg);
            addr.do_send(session::MqSessionMessage(message.to_owned()))
        }
    }
}
