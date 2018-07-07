#[macro_use]
extern crate actix;
extern crate futures;
extern crate tokio_tcp;

use actix::prelude::*;
use futures::Future;
use std::str::FromStr;
use std::{io, net, process, thread};
use tokio_tcp::TcpStream;

fn main() {
    println!("Running MQ client");

    actix::System::run(|| {
        // Connect to server
        let addr = net::SocketAddr::from_str("127.0.0.1:3000").unwrap();

        Arbiter::spawn(
            TcpStream::connect(&addr)
                .and_then(|stream| futures::future::ok(()))
                .map_err(|e| {
                    println!("Can not connect to server: {}", e);
                    process::exit(1)
                }),
        );
    });
}
