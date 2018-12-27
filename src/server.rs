use crate::types::NodeAppConfig;
use crate::sign;
use actix::prelude::*;
use actix::Message;
use sodiumoxide::crypto::sign::ed25519::PublicKey;
use std::collections::HashMap;

use crate::session;

/// `MqServer` manages MQ network and
/// responsible for network nodes
/// coordinating.
pub struct MqServer {
    sessions: HashMap<PublicKey, Addr<session::MqSession>>,
    settigns: NodeAppConfig,
}

impl MqServer {
    pub fn new(cfg: NodeAppConfig) -> MqServer {
        MqServer {
            sessions: HashMap::new(),
            settigns: cfg,
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
    type Result = Vec<String>;
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

/// Register client
#[derive(Message)]
pub struct MqRegister {
    /// Client identifier
    pub pub_key: PublicKey,
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for MqServer {
    type Result = MessageResult<Connect>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<Connect>");
        let (pk, _) = sign::gen_keypair();
        self.sessions.insert(pk, msg.addr);
        let res = vec![];
        MessageResult(res)
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for MqServer {
    type Result = ();

    fn handle(&mut self, _msg: Disconnect, _: &mut Context<Self>) {
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

/// Handler for Register message.
impl Handler<MqRegister> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqRegister, _: &mut Context<Self>) {
        println!("Handler<Register>");/*
        self.sessions.insert();
        if let Some(addr) = self.sessions.get(&msg.pub_key) {
            let message: String = format!("Response message for: {:?}", msg.msg);
            addr.do_send(session::MqSessionMessage(message.to_owned()))
        }*/
    }
}
