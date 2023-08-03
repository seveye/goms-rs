use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufWriter}, net::TcpStream};

use crate::watch::message::Message;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer : BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer : BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_message(&mut self) -> Option<Message> {
        loop {
            let mut buf = Cursor::new(&self.buffer[..]);
            // if let Some(message) = self.parse_message() {
            //     return Some(message);
            // }

            // if 0 == self.stream.read_buf(&mut self.buffer).await.unwrap() {
            //     return None;
            // }
        }
    }
}