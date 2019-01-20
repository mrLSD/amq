use actix::prelude::*;
use actix::Message;
use rand::{self, Rng};
use std::collections::HashMap;
use sodiumoxide::crypto::sign::ed25519::PublicKey;

use crate::session;

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
    type Result = ();
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect;

/// Send message to specific node
#[derive(Message)]
pub struct MqMessage {
    /// Node identifier
    pub pub_key: PublicKey,
    /// Peer message
    pub msg: String,
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<Connect>");
        // register session with random id
        /*let id = rand::thread_rng().gen::<u64>();
        self.sessions.insert(id, msg.addr);

        println!("Handler<Connect> | id: {:?}", id);

        // Return ID
        id*/
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Handler<Disconnect>");
    }
}

/// Handler for Message message.
impl Handler<MqMessage> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqMessage, _: &mut Context<Self>) {
        println!("Handler<Message>");
        if let Some(addr) = self.sessions.get(&msg.pub_key) {
            let message: String = format!("Response message for: {:?}", msg.msg);
            addr.do_send(session::MqSessionMessage(message.to_owned()))
        }
    }
}
