#[macro_use]
extern crate actix;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;

mod server;

use actix::prelude::*;
use tokio_tcp::{TcpListener, TcpStream};
use std::net;

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


fn main() {
}
