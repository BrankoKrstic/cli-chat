use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChatFrameTag {
    Message = 0,
    NameChange = 1,
    Connect = 2,
    Disconnect = 3,
}
#[derive(Debug, Clone)]
pub struct ChatFrame {
    pub tag: ChatFrameTag,
    pub payload: String,
}

pub struct ChatFrameCodec;

const MAX: usize = 8 * 1024 * 1024;

impl Decoder for ChatFrameCodec {
    type Item = ChatFrame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;
        if length == 0 {
            // discard in case of heartbeat (0 length read from buf)
            src.advance(4);
            return self.decode(src);
        }

        if src.len() < 5 {
            // Not enough data to read length marker + tag
            return Ok(None);
        }

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        let tag = match src[4] {
            0 => ChatFrameTag::Message,
            1 => ChatFrameTag::NameChange,
            2 => ChatFrameTag::Connect,
            3 => ChatFrameTag::Disconnect,
            tag => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown message type {}", tag),
                ))
            }
        };

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = if src.len() > 5 {
            src[5..4 + length].to_vec()
        } else {
            vec![]
        };

        src.advance(4 + length);

        // Convert the data to a string, or fail if it is not valid utf-8.
        match String::from_utf8(data) {
            Ok(payload) => Ok(Some(ChatFrame { tag, payload })),
            Err(utf8_error) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                utf8_error.utf8_error(),
            )),
        }
    }
}

impl Encoder<ChatFrame> for ChatFrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ChatFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Don't send a string if it is longer than the other end will
        // accept.
        if item.payload.len() + 1 > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", item.payload.len()),
            ));
        }

        let len_slice = u32::to_be_bytes(item.payload.len() as u32 + 1);
        // Reserve space in the buffer.
        dst.reserve(4 + 1 + item.payload.len());

        // Write the length and string to the buffer.
        dst.extend_from_slice(&len_slice);
        dst.put_u8(item.tag as u8);
        dst.extend_from_slice(item.payload.as_bytes());
        Ok(())
    }
}
