#![allow(unused_imports)]
#[macro_use]
extern crate actix;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;

mod server;

use actix::prelude::*;
use server::MqServer;
use std::net;
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

    fn handle(&mut self, _msg: TcpConnect, _: &mut Context<Self>) {}
}

fn main() {
    actix::System::run(|| {
        // Start server actor
        let server = MqServer::default().start();

        println!("Running chat MQ server...");
    });
}
