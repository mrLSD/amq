#[macro_use]
extern crate actix;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;

mod codec;

use actix::prelude::*;
use futures::Future;
use std::str::FromStr;
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
                }).map_err(|e| {
                    println!("Can not connect to server: {:?}", e);
                    process::exit(1)
                }),
        );
    });
}

struct MqClient {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, codec::ClientChatCodec>,
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
            act.framed.write(codec::ChatRequest::Ping);
            act.hb(ctx);
        });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _: &mut Context<Self>) {
        let m = msg.0.trim();

        // we check for /sss type of messages
        if m.starts_with('/') {
            let v: Vec<&str> = m.splitn(2, ' ').collect();
            match v[0] {
                "/list" => {
                    self.framed.write(codec::ChatRequest::List);
                }
                "/join" => {
                    if v.len() == 2 {
                        self.framed.write(codec::ChatRequest::Join(v[1].to_owned()));
                    } else {
                        println!("!!! room name is required");
                    }
                }
                _ => println!("!!! unknown command"),
            }
        } else {
            self.framed.write(codec::ChatRequest::Message(m.to_owned()));
        }
    }
}
