use crate::codec::{
    ClientMqCodec, MessageData,
    MessageProtocol::{Pub, ReqRep, Sub, UnSub},
    MqRequest, MqResponse, MqResponse,
};
use crate::server;
use crate::sign;
use crate::types::{ClientAppConfig, ClientConfig};

use actix::prelude::*;
use futures::{stream::once, Future};
use log::info;
use serde_derive::{Deserialize, Serialize};
use serde_json as json;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sign::ed25519::PublicKey;
use std::str::FromStr;
use std::time::Duration;
use std::time::SystemTime;
use std::{io, net, process, thread};
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;
use uuid::Uuid;

const PING_TIME_SEC: u64 = 5;

/// Basic type for MQ Client
pub struct MqClient {
    pub config: NodeAppConfig,
}

/// Basic MQ client connection data
struct MqClientConnection {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, ClientMqCodec>,
    settings: ClientAppConfig,
}

/// Struct for client message
#[derive(Debug, Deserialize, Serialize)]
struct ClientMessageData {
    title: String,
    amount: i32,
}

#[derive(Message)]
struct ClientCommand(String);

#[derive(Message)]
struct RegisterCommand(PublicKey);

/// MQ ClientCommand actor
impl Actor for MqClientConnection {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        info!("Disconnected");

        // Stop application on disconnect
        System::current().stop();

        Running::Stop
    }
}

/// Basic  MQ Client implementation
impl MqClient {
    /// Init New node struct with config data
    pub fn new(cfg: &NodeConfig) -> Self {
        Self {
            config: NodeAppConfig::new(cfg),
        }
    }

    /// Dial Client connection to Node
    pub fn dial(&self) {
        actix::System::run(move || {
            // Connect to server
            let addr = net::SocketAddr::from_str(&format!(
                "{}:{:?}",
                client_config.node.ip, client_config.node.port
            ))
            .unwrap();

            Arbiter::spawn(
                TcpStream::connect(&addr)
                    .and_then(move |stream| {
                        let addr = MqClient::create(move |ctx| {
                            let (r, w) = stream.split();
                            ctx.add_stream(FramedRead::new(r, ClientMqCodec));
                            ctx.add_message_stream(once(Ok(RegisterCommand(
                                client_config.public_key,
                            ))));
                            MqClientConnection {
                                framed: actix::io::FramedWrite::new(w, ClientMqCodec, ctx),
                                settings: client_config,
                            }
                        });

                        // Start console loop
                        let addr_to_send = addr.clone();
                        thread::spawn(move || loop {
                            let mut cmd = String::new();
                            if let Err(msg) = io::stdin().read_line(&mut cmd) {
                                println!("Error: {:?}", msg);
                                return;
                            }

                            addr_to_send.do_send(ClientCommand(cmd));
                        });

                        futures::future::ok(())
                    })
                    .map_err(|e| {
                        println!("Can not connect to server: {:?}", e);
                        process::exit(1)
                    }),
            );
        });
    }
}

/// Basic Mq Client Connection implementations
impl MqClientConnection {
    /// Heard beat flow for Ping connections
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(PING_TIME_SEC, 0), |act, ctx| {
            act.framed.write(MqRequest::Ping);
            act.hb(ctx);
        });
    }
}

impl actix::io::WriteHandler<io::Error> for MqClient {}

/// Handle Register commands
impl Handler<RegisterCommand> for MqClient {
    type Result = ();

    fn handle(&mut self, msg: RegisterCommand, _: &mut Context<Self>) {
        let pk = msg.0;
        println!("Handler<RegisterCommand>: {}", sign::to_hex_pk(&pk));
        self.framed.write(MqRequest::Register(pk));
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for MqClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _: &mut Context<Self>) {
        println!("Handler<ClientCommand>");

        let m = msg.0.trim();

        // we check for /command type of messages
        if m.starts_with('/') {
            let v: Vec<&str> = m.splitn(2, ' ').collect();
            let client1_pk = sign::from_string_pk(
                &"f20bfbae14396d9d3da5b35f8d9c2800244f59ddb7492af045930b760c893185".to_string(),
            );

            let client2_pk = sign::from_string_pk(
                &"5238e1c69a42280dc5d2d93ca18889a7ecbc6388872d4e64ac328eed7940b5b7".to_string(),
            );

            match v[0] {
                "/reqrep" => {
                    if v.len() < 2 {
                        println!(">> Wrong /reqrep command. For help print: /help");
                        return;
                    }
                    match v[1] {
                        "client1" => {
                            let msg_data = json::to_string(&ClientMessageData {
                                title: "message for client1".to_owned(),
                                amount: 100,
                            })
                            .expect("Message should be serialize to JSON");

                            let mut msg = MessageData {
                                id: Uuid::new_v4().to_string(),
                                to: Some(client1_pk),
                                signature: None,
                                event: None,
                                protocol: ReqRep,
                                time: SystemTime::now(),
                                nonce: None,
                                body: msg_data,
                            };

                            if self.settings.message.encode {
                                let nonce = box_::gen_nonce();
                                let encoded_msg = box_::seal(
                                    &msg.body.as_bytes(),
                                    &nonce,
                                    &self.settings.message.public_key,
                                    &self.settings.message.secret_key,
                                );

                                msg.body = sign::to_hex(&encoded_msg);
                                msg.nonce = Some(nonce);
                            }

                            let data =
                                json::to_string(&msg).expect("Message should be serialize to JSON");

                            // Set message sign
                            msg.signature = if self.settings.message.sign {
                                Some(sign::sign(data.as_bytes(), &self.settings.secret_key))
                            } else {
                                None
                            };

                            self.framed.write(MqRequest::Message(msg));
                        }
                        "client2" => {
                            let msg_data = json::to_string(&ClientMessageData {
                                title: "message for client2".to_owned(),
                                amount: 200,
                            })
                            .expect("Message should be serialize to JSON");

                            let mut msg = MessageData {
                                id: Uuid::new_v4().to_string(),
                                to: Some(client2_pk),
                                signature: None,
                                event: None,
                                protocol: ReqRep,
                                time: SystemTime::now(),
                                nonce: None,
                                body: msg_data,
                            };

                            if self.settings.message.encode {
                                let nonce = box_::gen_nonce();
                                let encoded_msg = box_::seal(
                                    &msg.body.as_bytes(),
                                    &nonce,
                                    &self.settings.message.public_key,
                                    &self.settings.message.secret_key,
                                );

                                msg.body = sign::to_hex(&encoded_msg);
                                msg.nonce = Some(nonce);
                            }

                            let data =
                                json::to_string(&msg).expect("Message should be serialize to JSON");

                            // Set message sign
                            msg.signature = if self.settings.message.sign {
                                Some(sign::sign(data.as_bytes(), &self.settings.secret_key))
                            } else {
                                None
                            };

                            self.framed.write(MqRequest::Message(msg));
                        }
                        _ => println!(">> Wrong /reqrep command. For help print: /help"),
                    }
                }
                "/pub" => {
                    if v.len() < 2 {
                        println!(">> Wrong /pub command. For help print: /help");
                        return;
                    }
                    let event_name = Some(v[1].to_owned());

                    let msg_data = json::to_string(&ClientMessageData {
                        title: format!(
                            "Public message from: {}",
                            sign::to_hex(&self.settings.public_key[..])
                        )
                        .to_owned(),
                        amount: 100,
                    })
                    .expect("Message should be serialize to JSON");

                    // Public message not encode message body
                    let mut msg = MessageData {
                        id: Uuid::new_v4().to_string(),
                        to: None,
                        signature: None,
                        event: event_name,
                        protocol: Pub,
                        time: SystemTime::now(),
                        nonce: None,
                        body: msg_data,
                    };

                    let data = json::to_string(&msg).expect("Message should be serialize to JSON");

                    // Set message sign
                    msg.signature = if self.settings.message.sign {
                        Some(sign::sign(data.as_bytes(), &self.settings.secret_key))
                    } else {
                        None
                    };

                    self.framed.write(MqRequest::Message(msg));
                }
                "/sub" => {
                    if v.len() < 2 {
                        println!(">> Wrong /sub command. For help print: /help");
                        return;
                    }
                    let event_name = Some(v[1].to_owned());

                    let mut msg = MessageData {
                        id: Uuid::new_v4().to_string(),
                        to: None,
                        signature: None,
                        event: event_name,
                        protocol: Sub,
                        time: SystemTime::now(),
                        nonce: None,
                        body: String::from(""),
                    };

                    let data = json::to_string(&msg).expect("Message should be serialize to JSON");

                    // Set message sign
                    msg.signature = if self.settings.message.sign {
                        Some(sign::sign(data.as_bytes(), &self.settings.secret_key))
                    } else {
                        None
                    };

                    self.framed.write(MqRequest::Message(msg));
                }
                "/unsub" => {
                    if v.len() < 2 {
                        println!(">> Wrong /unsub command. For help print: /help");
                        return;
                    }
                    let event_name = Some(v[1].to_owned());

                    let mut msg = MessageData {
                        id: Uuid::new_v4().to_string(),
                        to: None,
                        signature: None,
                        event: event_name,
                        protocol: UnSub,
                        time: SystemTime::now(),
                        nonce: None,
                        body: String::from(""),
                    };

                    let data = json::to_string(&msg).expect("Message should be serialize to JSON");

                    // Set message sign
                    msg.signature = if self.settings.message.sign {
                        Some(sign::sign(data.as_bytes(), &self.settings.secret_key))
                    } else {
                        None
                    };

                    self.framed.write(MqRequest::Message(msg));
                }
                "/ping" => {
                    if v.len() < 2 {
                        println!(">> Wrong /ping command. For help print: /help");
                        return;
                    }
                    match v[1] {
                        "client1" => {
                            self.framed.write(MqRequest::PingClient(client1_pk));
                        }
                        "client2" => {
                            self.framed.write(MqRequest::PingClient(client2_pk));
                        }
                        _ => {
                            println!("Unknown client name. Print for help: /help");
                            return;
                        }
                    }
                }
                "/help" => {
                    println!(
                        r#"Commands HELP:
    /ping [CLIENT]      ping connected clients
                        client will ping by pub_key.
                        Available clients name: client1, client2

    /help               print this help
    [CLIENT] [MESSAGE]  send message to specific client.
                        Available clients name: client1, client2

    /reqrep [CLIENT]    send REQ/REP message to specific client.
                        Available clients name: client1, client2

    /pub [NAME]         send PUB message for specific channel name.
                        Example: /pub my.public.channel

    /sub [NAME]         send SUB message to subscribe for specific channel name.
                        Example: /sub my.public.channel

    /unsub [NAME]       send UNSUB message to unsubscrive from specific channel name.
                        Example: /unsub my.public.channel

                "#
                    );
                }
                _ => println!(">> unknown command. For help print: /help"),
            }
        } else {
            println!(">> Unknown command. For help print: /help");
        }
    }
}

/// Server communication
impl StreamHandler<MqResponse, io::Error> for MqClient {
    fn handle(&mut self, msg: MqResponse, _: &mut Context<Self>) {
        match msg {
            MqResponse::Message(mut msg) => {
                let is_verified = msg.verify();
                println!("message: {:#?}", msg);
                println!("is verified: {:#?}", is_verified);

                // Encode message
                if msg.protocol != Pub && self.settings.message.encode {
                    let encoded_msg = box_::open(
                        &sign::from_hex(&msg.body),
                        &msg.nonce.unwrap(),
                        &self.settings.message.public_key,
                        &self.settings.message.secret_key,
                    )
                    .expect("Message should be decoded.");

                    let msg_data = std::str::from_utf8(&encoded_msg[..])
                        .expect("Message should be valid UTF8 string");
                    let client_msg: ClientMessageData = json::from_str(&msg_data).unwrap();
                    dbg!(client_msg);
                } else {
                    let client_msg: ClientMessageData = json::from_str(&msg.body).unwrap();
                    dbg!(client_msg);
                }

                // Send message response data for ReqRep
                if msg.protocol == ReqRep {
                    self.framed
                        .write(MqRequest::MessageResponse(server::MqMessageResponse {
                            from: msg.from,
                            to: msg.to,
                            status: server::MessageSendStatus::Received,
                        }));
                }
            }
            MqResponse::Pong => {}
            MqResponse::PingClient(pk) => {
                println!("PingClient");
                self.framed.write(MqRequest::PongClient(pk));
            }
            MqResponse::PongClient(pk) => {
                println!("PongClient response: {:}", sign::to_hex_pk(&pk));
            }
            MqResponse::MessageResponseStatus(status) => {
                println!("MessageResponseStatus: {:#?}", status);
            }
        }
    }
}
