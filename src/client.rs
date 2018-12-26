mod codec;
mod sign;
mod types;

use crate::types::{ClientAppConfig, ClientConfig};
use actix::prelude::*;
use futures::stream::once;
use futures::Future;
use std::str::FromStr;
use std::time::Duration;
use std::{io, net, process, thread};
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;
use toml;

use sodiumoxide::crypto::sign::ed25519::PublicKey;

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
        let addr = net::SocketAddr::from_str(&format!("{:?}:{:?}", client_config.node.ip, client_config.node.port)).unwrap();

        Arbiter::spawn(
            TcpStream::connect(&addr)
                .and_then(move |stream| {
                    let addr = MqClient::create(move |ctx| {
                        let (r, w) = stream.split();
                        ctx.add_stream(FramedRead::new(r, codec::ClientMqCodec));
                        ctx.add_message_stream(once(Ok(RegisterCommand(client_config.public_key))));
                        MqClient {
                            framed: actix::io::FramedWrite::new(w, codec::ClientMqCodec, ctx),
                        }
                    });

                    // start console loop
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

struct MqClient {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, codec::ClientMqCodec>,
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

    fn handle(&mut self, _: RegisterCommand, _: &mut Context<Self>) {
        let (pk, _) = sign::gen_keypair();
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

        // we check for /sss type of messages
        if m.starts_with('/') {
            let v: Vec<&str> = m.splitn(2, ' ').collect();
            match v[0] {
                "/ping" => {
                    self.framed.write(codec::MqRequest::Ping);
                }
                _ => println!(">> unknown command"),
            }
        } else {
            self.framed.write(codec::MqRequest::Message(m.to_owned()));
        }
    }
}

/// Server communication
impl StreamHandler<codec::MqResponse, io::Error> for MqClient {
    fn handle(&mut self, msg: codec::MqResponse, _: &mut Context<Self>) {
        match msg {
            codec::MqResponse::Message(ref msg) => {
                println!("message: {}", msg);
            }
            codec::MqResponse::Pong => {}
        }
    }
}
