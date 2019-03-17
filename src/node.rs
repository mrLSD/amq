use crate::types::{NodeAppConfig, NodeConfig};
use actix::io::FramedWrite;
use actix::prelude::*;
use futures::Stream;
use log::info;
use std::net;
use std::str::FromStr;
use tokio_codec::FramedRead;
use tokio_io::AsyncRead;
use tokio_tcp::{TcpListener, TcpStream};
use toml;

mod codec;
mod server;
mod session;
mod sign;
mod types;

use crate::codec::MqCodec;
use crate::server::MqServer;
use crate::session::MqSession;

/// Define tcp server that will accept incoming tcp
/// connection and create MQ actors.
struct Server {
    server: Addr<MqServer>,
}

/// Make actor from `Server`
impl Actor for Server {
    /// Every actor has to provide execution `Context` in which it can run.
    type Context = Context<Self>;
}

#[derive(Message)]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

// Handle stream of TcpStream's
impl Handler<TcpConnect> for Server {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        println!("Handler<TcpConnect>");
        // For each incoming connection we create `MqSession` actor
        // with out MQ server address.
        let server = self.server.clone();
        MqSession::create(move |ctx| {
            let (r, w) = msg.0.split();
            MqSession::add_stream(FramedRead::new(r, MqCodec), ctx);
            MqSession::new(server, FramedWrite::new(w, MqCodec, ctx))
        });
    }
}

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
Actix MQ network Node

Usage: node [CONFIG_FILE]
    "#
    );
    std::process::exit(code);
}

/// Read config data form TOML file
fn read_config() -> NodeConfig {
    let mut args = std::env::args();
    let config_file = args.nth(1).unwrap();

    let config_data = std::fs::read_to_string(config_file).expect("File not found");
    toml::from_str(&config_data).expect("Failed to parse config file")
}

/// MqNode - basic type for MQ Node
pub struct MqNode {
    pub config: NodeAppConfig,
}

/// MqNode - basic Node implementaion
impl MqNode {
    /// Init New node struct with config data
    pub fn new(cfg: &NodeConfig) -> Self {
        Self {
            config: NodeAppConfig::new(cfg),
        }
    }

    /// Serve Node based on Config data
    pub fn serve(&self) {
        let config = self.config.clone();
        actix::System::run(move || {
            // Start server actor
            let server = MqServer::new(config.clone()).start();

            // Create server listener
            let addr = net::SocketAddr::from_str(&format!("0.0.0.0:{:?}", config.port))
                .expect("Can't parse TCP Address");
            let listener = TcpListener::bind(&addr).expect("Can't bind TCP address");

            // Our MQ server `Server` is an actor, first we need to start it
            // and then add stream on incoming tcp connections to it.
            // TcpListener::incoming() returns stream of the (TcpStream, net::SocketAddr)
            // items So to be able to handle this events `Server` actor has to implement
            // stream handler `StreamHandler<(TcpStream, net::SocketAddr), io::Error>`
            Server::create(|ctx| {
                ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|stream| {
                    let addr = stream.peer_addr().unwrap();
                    TcpConnect(stream, addr)
                }));
                Server { server }
            });

            info!("Running MQ server...");
        });
    }
}

fn main() {
    check_commands();
    let node_config = NodeAppConfig::new(&read_config());

    actix::System::run(move || {
        // Start server actor
        let server = MqServer::new(node_config.clone()).start();

        // Create server listener
        let addr = net::SocketAddr::from_str(&format!("0.0.0.0:{:?}", node_config.port)).unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        // Our MQ server `Server` is an actor, first we need to start it
        // and then add stream on incoming tcp connections to it.
        // TcpListener::incoming() returns stream of the (TcpStream, net::SocketAddr)
        // items So to be able to handle this events `Server` actor has to implement
        // stream handler `StreamHandler<(TcpStream, net::SocketAddr), io::Error>`
        Server::create(|ctx| {
            ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|stream| {
                let addr = stream.peer_addr().unwrap();
                TcpConnect(stream, addr)
            }));
            Server { server }
        });

        info!("Running MQ Node...");
    });
}
