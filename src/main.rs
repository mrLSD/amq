#![allow(unused_imports)]
#[macro_use]
extern crate actix;
extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate byteorder;
extern crate bytes;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod codec;
mod server;
mod session;

use actix::prelude::*;
use futures::Stream;
use server::MqServer;
use std::net;
use std::str::FromStr;
use tokio_tcp::{TcpListener, TcpStream};

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

    fn handle(&mut self, _msg: TcpConnect, _: &mut Context<Self>) {
        println!("Handler<TcpConnect>");
    }
}

fn main() {
    actix::System::run(|| {
        // Start server actor
        let server = MqServer::default().start();

        // Create server listener
        let addr = net::SocketAddr::from_str("127.0.0.1:3000").unwrap();
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
            Server { server: server }
        });

        println!("Running chat MQ server...");
    });
}
