use actix::io::{FramedWrite, WriteHandler};
use actix::prelude::*;
use std::io;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use codec::{MqCodec, MqRequest, MqResponse};
use server::{self, MqServer};

/// MQ server sends this messages to session
#[derive(Message)]
pub struct MqMessage(pub String);

/// `MqSession` actor is responsible for tcp peer communications.
pub struct MqSession {
    /// MQ session unique ID
    id: u64,
    /// this is address of MQ server
    addr: Addr<MqServer>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// Framed wrapper
    framed: FramedWrite<WriteHalf<TcpStream>, MqCodec>,
}

impl Actor for MqSession {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // We'll start heartbeat process on session start.
        self.hb(ctx);

        // Register self in MQ server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        self.addr
            .send(server::Connect {
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with MQ server
                    _ => ctx.stop(),
                }
                actix::fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify MQ server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

impl WriteHandler<io::Error> for MqSession {}

impl MqSession {
    pub fn new(
        addr: Addr<MqServer>,
        framed: FramedWrite<WriteHalf<TcpStream>, MqCodec>,
    ) -> MqSession {
        MqSession {
            addr,
            framed,
            id: 0,
            hb: Instant::now(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method check heartbeats from client
    fn hb(&self, ctx: &mut actix::Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                // heartbeat timed out
                println!("Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();
            }

            act.framed.write(ChatResponse::Ping);
            act.hb(ctx);
        });
    }
}
