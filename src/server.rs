use crate::types::NodeAppConfig;
use actix::prelude::*;
use actix::Message;
use serde_derive::{Deserialize, Serialize};
use serde_json as json;
use sodiumoxide::crypto::box_ as cipher;
use sodiumoxide::crypto::sign::ed25519::{PublicKey, Signature};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::codec;
use crate::codec::MessageProtocol::{Pub, Sub, UnSub};
use crate::session;
use crate::sign;

/// `MqServer` manages MQ network and
/// responsible for network nodes
/// coordinating.
#[allow(dead_code)]
pub struct MqServer {
    sessions: HashMap<PublicKey, Addr<session::MqSession>>,
    events: MessageEvents,
    settigns: NodeAppConfig,
}

#[allow(dead_code)]
impl MqServer {
    pub fn new(cfg: NodeAppConfig) -> MqServer {
        MqServer {
            sessions: HashMap::new(),
            events: MessageEvents {
                subscribers: HashMap::new(),
            },
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
    type Result = PublicKey;
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect(pub PublicKey);

/// Basic MQ Message Data
#[derive(Message, Debug, Deserialize, Serialize, Clone)]
pub struct MqMessage {
    pub id: String,
    pub from: PublicKey,
    pub to: Option<PublicKey>,
    pub signature: Option<Signature>,
    pub event: Option<String>,
    pub protocol: codec::MessageProtocol,
    pub time: SystemTime,
    pub nonce: Option<cipher::Nonce>,
    pub body: String,
}

#[allow(dead_code)]
impl MqMessage {
    /// Convert message to Client message
    pub fn to_message(&self) -> codec::MessageData {
        codec::MessageData {
            id: self.id.clone(),
            to: self.to,
            signature: self.signature,
            event: self.event.clone(),
            protocol: self.protocol.clone(),
            time: self.time,
            nonce: self.nonce,
            body: self.body.clone(),
        }
    }

    /// Verify message signature
    pub fn verify(&mut self) -> bool {
        if self.signature.is_none() {
            return false;
        }
        let mut msg = self.to_message();
        msg.signature = None;
        let data = json::to_string(&msg).expect("Message should be serialize to JSON");
        sign::verify(&self.signature.unwrap(), data.as_bytes(), &self.from)
    }
}

/// Public Message Events subscribers
pub type MessageEventsSubscribers = HashMap<String, Vec<PublicKey>>;

/// Message Events data
#[derive(Debug, Clone)]
pub struct MessageEvents {
    /// PUBSUB subscribers
    pub subscribers: MessageEventsSubscribers,
}

/// Sent Message response data
#[derive(Message, Debug, Serialize, Deserialize)]
pub struct MqMessageResponse {
    pub from: PublicKey,
    pub to: Option<PublicKey>,
    pub status: MessageSendStatus,
}

/// Ping message for specific client
#[derive(Message)]
pub struct MqPingClient {
    pub from: PublicKey,
    pub to: PublicKey,
}

/// Pong message for specific client
#[derive(Message)]
pub struct MqPongClient {
    pub from: PublicKey,
    pub to: PublicKey,
}

/// Register client
pub struct MqRegister {
    /// Old client identifier
    /// as temporary value that should
    /// be set to real client pub_key
    pub old_pub_key: PublicKey,
    /// Client identifier
    pub pub_key: PublicKey,
}

/// Message send statuses
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageSendStatus {
    Sent,
    Received,
    PeerNotFound,
    Failed,
}

/// Response type for Register message
/// It can be success or fail
impl actix::Message for MqRegister {
    type Result = Option<PublicKey>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for MqServer {
    type Result = MessageResult<Connect>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<Connect>");
        // Generate temporary pub_key for session identification
        let (pk, _) = sign::gen_keypair();
        self.sessions.insert(pk, msg.addr);
        MessageResult(pk)
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let pub_key = msg.0;
        println!("Handler<Disconnect>");
        // Unregister session
        self.sessions.remove(&pub_key);
    }
}

/// Handler for Message message.
impl Handler<MqMessage> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqMessage, _: &mut Context<Self>) {
        println!("Handler<Message>");
        let msg_data = msg.clone();
        // Send message and set message status response
        let status = if (msg.protocol == Pub || msg.protocol == Sub || msg.protocol == UnSub)
            && msg.event.is_some()
        {
            match msg.protocol {
                Pub => {
                    let event_name = msg.event.clone().unwrap();
                    // Send message to subscribers
                    if let Some(event) = self.events.subscribers.get(&event_name) {
                        for subscriber in event {
                            if let Some(addr) = self.sessions.get(subscriber) {
                                addr.do_send(session::MqSessionMessage(msg.clone()));
                            }
                        }
                    }
                }
                Sub => {
                    let event_name = msg.event.unwrap();
                    // Add subscriber to specific Event
                    if let Some(event) = self.events.subscribers.get_mut(&event_name) {
                        event.push(msg.from);
                    } else {
                        self.events.subscribers.insert(event_name, vec![msg.from]);
                    }
                }
                UnSub => {
                    let event_name = msg.event.clone().unwrap();
                    // Remove subscriber from specific Event
                    if let Some(event) = self.events.subscribers.get_mut(&event_name) {
                        let from = msg.from;
                        let element = event.iter().position(|x| *x == from);
                        if element.is_some() {
                            let _ = event.remove(element.unwrap());
                        }
                    }
                }
                _ => {}
            }
            MessageSendStatus::Sent
        } else if msg.to.is_none() {
            // Check is set peer `to`
            MessageSendStatus::PeerNotFound
        } else if let Some(addr) = self.sessions.get(&msg.to.unwrap()) {
            // Send message to peer `to`
            addr.do_send(session::MqSessionMessage(msg));
            MessageSendStatus::Sent
        } else if self.sessions.get(&msg.from).is_some() {
            // We found `from` peer but not found peer `to`
            MessageSendStatus::PeerNotFound
        } else {
            MessageSendStatus::Failed
        };

        // Send message response to peer
        if let Some(addr) = self.sessions.get(&msg_data.from) {
            addr.do_send(MqMessageResponse {
                from: msg_data.from,
                to: msg_data.to,
                status,
            });
        }
    }
}

/// Handler for Register message.
impl Handler<MqRegister> for MqServer {
    type Result = MessageResult<MqRegister>;

    fn handle(&mut self, msg: MqRegister, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<Register>");

        // Check is Client already registered
        if self.sessions.get(&msg.pub_key).is_some() {
            eprintln!("Client already registered - close session");
            return MessageResult(None);
        }

        if let Some(addr) = self.sessions.get(&msg.old_pub_key) {
            self.sessions.insert(msg.pub_key, addr.to_owned());
            self.sessions.remove(&msg.old_pub_key);
        } else {
            eprintln!("Session address not found");
            return MessageResult(None);
        }
        MessageResult(Some(msg.pub_key))
    }
}

/// Handler for Ping Client message.
impl Handler<MqPingClient> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqPingClient, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<MqPingClient>");

        if let Some(addr) = self.sessions.get(&msg.to) {
            addr.do_send(session::MqSessionPingClient(msg.from));
        }
    }
}

/// Handler for Pong Client message.
impl Handler<MqPongClient> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqPongClient, _: &mut Context<Self>) -> Self::Result {
        println!("Handler<MqPongClient>");

        if let Some(addr) = self.sessions.get(&msg.to) {
            addr.do_send(session::MqSessionPongClient(msg.from));
        }
    }
}

/// Handler for Response message.
impl Handler<MqMessageResponse> for MqServer {
    type Result = ();

    fn handle(&mut self, msg: MqMessageResponse, _: &mut Context<Self>) {
        println!("Handler<MqMessageResponse>");

        // Send response message to `from` peer
        if let Some(addr) = self.sessions.get(&msg.from) {
            addr.do_send(msg);
        }
    }
}
