use actix::io::{FramedWrite, WriteHandler};
use actix::prelude::*;
use actix::Message;
use sodiumoxide::crypto::sign::ed25519::PublicKey;
use std::io;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use crate::codec::{MqCodec, MqRequest, MqResponse};
use crate::server::{self, MqServer};
use crate::sign;

const PING_TIME_SEC: u64 = 5;
const PING_WAIT_SEC: u64 = 15;

/// MQ server sends this messages to session
#[derive(Message)]
pub struct MqSessionMessage(pub String);

/// MQ server sends this Disconnect for
/// current session
#[derive(Message)]
pub struct MqSessionDisconnect;

/// Ping message for Client
#[derive(Message)]
pub struct MqSessionPingClient(pub PublicKey);

/// Pong message for Client
#[derive(Message)]
pub struct MqSessionPongClient(pub PublicKey);

/// `MqSession` actor is responsible for tcp peer communications.
pub struct MqSession {
    /// MQ session NodePublicKey
    pub_key: Option<PublicKey>,
    /// this is address of MQ server
    addr: Addr<MqServer>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// Framed wrapper
    framed: FramedWrite<WriteHalf<TcpStream>, MqCodec>,
}

impl Actor for MqSession {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // We'll start heartbeat process on session start.
        self.hb(ctx);
        println!("Session started");

        // Register self in MQ server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        self.addr
            .send(server::Connect {
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(pk) => {
                        // Set session pub_key
                        act.pub_key = Some(pk);
                    }
                    // something is wrong with MQ server
                    _ => ctx.stop(),
                }
                actix::fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify MQ server
        self.addr.do_send(server::Disconnect(self.pub_key.unwrap()));
        Running::Stop
    }
}

impl WriteHandler<io::Error> for MqSession {}

/// To use `Framed` with an actor, we have to implement `StreamHandler` trait
impl StreamHandler<MqRequest, io::Error> for MqSession {
    /// This is main event loop for client requests
    fn handle(&mut self, msg: MqRequest, ctx: &mut Self::Context) {
        match msg {
            MqRequest::Message(message) => {
                if self.pub_key.is_none() {
                    eprintln!("MqRequest::Message - pub_key not sets");
                    return;
                }
                // Send message to MQ server
                println!("Peer message: {:#?}", message);
                let msg = server::MqMessage {
                    from: self.pub_key.unwrap(),
                    to: message.to,
                    signature: message.signature,
                    name: message.name,
                    protocol: message.protocol,
                    time: message.time,
                    nonce: message.nonce,
                    body: message.body,
                };
                self.addr.do_send(msg);
            }
            // we update heartbeat time on ping from peer
            MqRequest::Ping => self.hb = { Instant::now() },
            MqRequest::PingClient(pk) => {
                println!("MqRequest::PingClient");
                self.addr.do_send(server::MqPingClient {
                    from: self.pub_key.unwrap(),
                    to: pk,
                });
            }
            MqRequest::PongClient(pk) => {
                println!("MqRequest::PongClient");
                self.addr.do_send(server::MqPongClient {
                    from: self.pub_key.unwrap(),
                    to: pk,
                });
            }
            MqRequest::Register(pk) => {
                if self.pub_key.is_none() {
                    eprintln!("Register pub_key: session pub_key not set");
                    return;
                }

                let old_pub_key = self.pub_key.unwrap();

                println!("Register pub_key: {}", sign::to_hex_pk(&pk));

                self.addr
                    .send(server::MqRegister {
                        old_pub_key: old_pub_key,
                        pub_key: pk,
                    })
                    .into_actor(self)
                    .then(|res, act, ctx| {
                        match res {
                            // Registration successful
                            Ok(Some(pub_key)) => {
                                // Change old pub_key
                                act.pub_key = Some(pub_key.clone());
                            }
                            // Registration failed
                            // stopping current session
                            _ => ctx.stop(),
                        }
                        actix::fut::ok(())
                    })
                    .wait(ctx);
            }
        }
    }
}

/// Handler for MqMessage, MqServer sends this message
impl Handler<MqSessionMessage> for MqSession {
    type Result = ();

    fn handle(&mut self, msg: MqSessionMessage, _: &mut Self::Context) {
        // Send message to peer
        self.framed.write(MqResponse::Message(msg.0));
    }
}

/// Handler for hard disconnect current session
impl Handler<MqSessionDisconnect> for MqSession {
    type Result = ();

    fn handle(&mut self, _: MqSessionDisconnect, ctx: &mut Self::Context) {
        println!("Handler<MqSessionDisconnect>");
        // Notify MQ server
        self.addr.do_send(server::Disconnect(self.pub_key.unwrap()));

        // Stop actor
        ctx.stop();
    }
}

/// Handler for Clent Ping
impl Handler<MqSessionPingClient> for MqSession {
    type Result = ();

    fn handle(&mut self, msg: MqSessionPingClient, _: &mut Self::Context) {
        // Send Ping message to peer
        println!("Handler<MqSessionPingClient>: {}", sign::to_hex_pk(&msg.0));
        self.framed.write(MqResponse::PingClient(msg.0));
    }
}

/// Handler for Clent Pong
impl Handler<MqSessionPongClient> for MqSession {
    type Result = ();

    fn handle(&mut self, msg: MqSessionPongClient, _: &mut Self::Context) {
        // Send Pong message to peer
        self.framed.write(MqResponse::PongClient(msg.0));
    }
}

impl MqSession {
    /// Basic Session initialisation
    pub fn new(
        addr: Addr<MqServer>,
        framed: FramedWrite<WriteHalf<TcpStream>, MqCodec>,
    ) -> MqSession {
        MqSession {
            pub_key: None,
            addr,
            framed,
            hb: Instant::now(),
        }
    }

    /// Helper method that sends ping to client every second.
    ///
    /// Also this method check heartbeats from client
    fn hb(&self, ctx: &mut actix::Context<Self>) {
        ctx.run_later(Duration::new(PING_TIME_SEC, 0), |act, ctx| {
            // Check client heartbeats
            if Instant::now().duration_since(act.hb) > Duration::new(PING_WAIT_SEC, 0) {
                // Heartbeat timed out
                println!("Client heartbeat failed, disconnecting!");

                // Notify MQ server
                act.addr.do_send(server::Disconnect(act.pub_key.unwrap()));

                // Stop actor
                ctx.stop();
            }

            act.framed.write(MqResponse::Pong);
            act.hb(ctx);
        });
    }
}
