#![allow(dead_code)]
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

/// Client request
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum MqRequest {
    /// Ping request
    Ping,
    /// Send message
    Message(String),
}

/// Server response
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum MqResponse {
    /// Pong response
    Pong,
    /// Receive Message
    Message(String),
}

/// Codec for Client -> Server transport
pub struct MqCodec;

impl Decoder for MqCodec {
    type Item = MqRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MqRequest>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for MqCodec {
    type Item = MqResponse;
    type Error = io::Error;

    fn encode(&mut self, msg: MqResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}

/// Codec for Server -> Client transport
pub struct ClientMqCodec;

impl Decoder for ClientMqCodec {
    type Item = MqResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MqResponse>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ClientMqCodec {
    type Item = MqRequest;
    type Error = io::Error;

    fn encode(&mut self, msg: MqRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
