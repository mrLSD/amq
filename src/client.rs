mod codec;
mod server;
mod session;
mod sign;
mod types;

use crate::types::{ClientAppConfig, ClientConfig};

use actix::prelude::*;
use futures::stream::once;
use futures::Future;
use serde_json;
use sodiumoxide::crypto::sign::ed25519::PublicKey;
use std::str::FromStr;
use std::time::Duration;
use std::time::SystemTime;
use std::{io, net, process, thread};
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;
use toml;

const PING_TIME_SEC: u64 = 5;

/// Check command arguments
fn check_commands() {
    let args = std::env::args();
    if args.len() != 2 {
        help_message(1);
    }
}

/// Print help message for CLI commands
fn help_message(code: i32) {
    println!(
        r#"
Actix MQ network Client

Usage: client [CONFIG_FILE]
    "#
    );
    std::process::exit(code);
}

/// Read config data form TOML file
fn read_config() -> ClientConfig {
    let mut args = std::env::args();
    let config_file = args.nth(1).unwrap();

    let config_data = std::fs::read_to_string(config_file).expect("File not found");
    toml::from_str(&config_data).expect("Failed to parse config file")
}

fn main() {
    check_commands();
    let client_config = ClientAppConfig::new(&read_config());

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
                        ctx.add_stream(FramedRead::new(r, codec::ClientMqCodec));
                        ctx.add_message_stream(once(Ok(RegisterCommand(client_config.public_key))));
                        MqClient {
                            framed: actix::io::FramedWrite::new(w, codec::ClientMqCodec, ctx),
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

/// Basic MQ client data
struct MqClient {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, codec::ClientMqCodec>,
    settings: ClientAppConfig,
}

#[derive(Message)]
struct ClientCommand(String);

#[derive(Message)]
struct RegisterCommand(PublicKey);

impl Actor for MqClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();

        Running::Stop
    }
}

impl MqClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(PING_TIME_SEC, 0), |act, ctx| {
            act.framed.write(codec::MqRequest::Ping);
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
        self.framed.write(codec::MqRequest::Register(pk));
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
                &"b521a35da1439d86c8c5c9eb54e5bbef6500f53afb477a89c46d7e54dc77efaf".to_string(),
            );

            let client2_pk = sign::from_string_pk(
                &"7b2e20f9c6bac2033185fe5c9952d8053ccef6af30104060f3475c9be9d40e78".to_string(),
            );

            match v[0] {
                "/reqrep" => {
                    if v.len() < 2 {
                        println!(">> Wrong /reqrep command. For help print: /help");
                        return;
                    }
                    match v[1] {
                        "client1" => {
                            let mut msg = codec::MessageData {
                                to: client1_pk,
                                signature: None,
                                name: None,
                                protocol: codec::MessageProtocol::ReqRep,
                                time: SystemTime::now(),
                                nonce: None,
                                body: "message for client1".to_owned(),
                            };
                            let data = serde_json::to_string(&msg)
                                .expect("Message should be serialize to JSON");
                            msg.signature =
                                Some(sign::sign(data.as_bytes(), &self.settings.secret_key));
                            self.framed.write(codec::MqRequest::Message(msg));
                        }
                        "client2" => {
                            let mut msg = codec::MessageData {
                                to: client2_pk,
                                signature: None,
                                name: None,
                                protocol: codec::MessageProtocol::ReqRep,
                                time: SystemTime::now(),
                                nonce: None,
                                body: "message for client2".to_owned(),
                            };
                            let data = serde_json::to_string(&msg)
                                .expect("Message should be serialize to JSON");
                            msg.signature =
                                Some(sign::sign(data.as_bytes(), &self.settings.secret_key));
                            self.framed.write(codec::MqRequest::Message(msg));
                        }
                        _ => println!(">> Wrong /reqrep command. For help print: /help"),
                    }
                }
                "/ping" => {
                    if v.len() < 2 {
                        println!(">> Wrong /ping command. For help print: /help");
                        return;
                    }
                    match v[1] {
                        "client1" => {
                            self.framed.write(codec::MqRequest::PingClient(client1_pk));
                        }
                        "client2" => {
                            self.framed.write(codec::MqRequest::PingClient(client2_pk));
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

    /reqrep             send REQ/REP message to specific client.
                        Available clients name: client1, client2

                "#
                    );
                }
                _ => println!(">> unknown command"),
            }
        } else {
            println!(">> Unknown command. For help print: /help");
        }
    }
}

/// Server communication
impl StreamHandler<codec::MqResponse, io::Error> for MqClient {
    fn handle(&mut self, msg: codec::MqResponse, _: &mut Context<Self>) {
        match msg {
            codec::MqResponse::Message(msg) => {
                let mut msg_for_sign = msg.clone();
                msg_for_sign.signature = None;
                println!("message: {:#?}", msg);
            }
            codec::MqResponse::Pong => {}
            codec::MqResponse::PingClient(pk) => {
                println!("PingClient");
                self.framed.write(codec::MqRequest::PongClient(pk));
            }
            codec::MqResponse::PongClient(pk) => {
                println!("PongClient response: {:}", sign::to_hex_pk(&pk));
            }
        }
    }
}
