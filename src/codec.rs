use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::message::{ClientMessage, ServerMessage};

pub struct ClientCodec;

const MAX: usize = 8 * 1024 * 1024;

impl Encoder<ClientMessage> for ClientCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ClientMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let item = serde_json::to_string(&item)?;
        // Don't send a string if it is longer than the other end will
        // accept.
        if item.len() > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", item.len()),
            ));
        }
        // Reserve space in the buffer.
        dst.reserve(item.len());

        dst.extend_from_slice(item.as_bytes());
        Ok(())
    }
}

impl Decoder for ClientCodec {
    type Item = ServerMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            // Not enough data to read length marker.
            return Ok(None);
        }
        let de = serde_json::Deserializer::from_slice(src);
        let mut iter = de.into_iter::<ServerMessage>();
        let item = match iter.next() {
            Some(Ok(item)) => item,
            Some(Err(ref e)) if e.is_eof() => return Ok(None),
            Some(Err(e)) => return Err(e.into()),
            None => return Ok(None),
        };

        let offset = iter.byte_offset();
        src.advance(offset);
        Ok(Some(item))
    }
}

pub struct ServerCodec;

impl Decoder for ServerCodec {
    type Item = ClientMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            // Not enough data to read length marker.
            return Ok(None);
        }
        let de = serde_json::Deserializer::from_slice(src);
        let mut iter = de.into_iter::<ClientMessage>();
        let item = match iter.next() {
            Some(Ok(item)) => item,
            Some(Err(ref e)) if e.is_eof() => return Ok(None),
            Some(Err(e)) => return Err(e.into()),
            None => return Ok(None),
        };

        let offset = iter.byte_offset();
        src.advance(offset);
        Ok(Some(item))
    }
}

impl Encoder<ServerMessage> for ServerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ServerMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let item = serde_json::to_string(&item)?;
        // Don't send a string if it is longer than the other end will
        // accept.
        if item.len() > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", item.len()),
            ));
        }
        // Reserve space in the buffer.
        dst.reserve(item.len());

        dst.extend_from_slice(item.as_bytes());
        Ok(())
    }
}
