#[macro_use]
extern crate actix;
extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;
#[macro_use]
extern crate serde_derive;

mod codec;

use actix::prelude::*;
use futures::Future;
use std::str::FromStr;
use std::time::Duration;
use std::{io, net, process, thread};
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;

fn main() {
    println!("Running MQ client");

    actix::System::run(|| {
        // Connect to server
        let addr = net::SocketAddr::from_str("127.0.0.1:3000").unwrap();

        Arbiter::spawn(
            TcpStream::connect(&addr)
                .and_then(|stream| {
                    let addr = MqClient::create(|ctx| {
                        let (r, w) = stream.split();
                        ctx.add_stream(FramedRead::new(r, codec::ClientMqCodec));
                        MqClient {
                            framed: actix::io::FramedWrite::new(w, codec::ClientMqCodec, ctx),
                        }
                    });

                    // start console loop
                    thread::spawn(move || loop {
                        let mut cmd = String::new();
                        if let Err(msg) = io::stdin().read_line(&mut cmd) {
                            println!("Error: {:?}", msg);
                            return;
                        }

                        addr.do_send(ClientCommand(cmd));
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
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.framed.write(codec::MqRequest::Ping);
            act.hb(ctx);
        });
    }
}

impl actix::io::WriteHandler<io::Error> for MqClient {}

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
        println!("StreamHandler<codec::MqResponse>");
        match msg {
            codec::MqResponse::Message(ref msg) => {
                println!("message: {}", msg);
            }
            codec::MqResponse::Pong => {
                println!("PONG");
            }
        }
    }
}
