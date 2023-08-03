use bytes::{Buf, BytesMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufWriter}, net::TcpStream};

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
}