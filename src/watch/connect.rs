use std::io::Error;
use bytes::{BytesMut, BufMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufWriter}, net::tcp::{OwnedWriteHalf, OwnedReadHalf}};
use crate::watch::message::{Message, new_message};


#[derive(Debug)]
pub struct TcpWriter {
    pub stream: BufWriter<OwnedWriteHalf>,
}

impl TcpWriter {
    pub fn new(stream: OwnedWriteHalf) -> Self {
        Self {
            stream: BufWriter::new(stream),
        }
    }

    pub async fn write_message(&mut self, msg: &Message) -> Result<(), Error> {
        self.stream.write_all(msg.cmd.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        self.stream.write_all(msg.seq.to_string().as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        self.stream.write_all(msg.values.len().to_string().as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        for v in &msg.values {
            self.stream.write_all(v.as_bytes()).await?;
            self.stream.write_all(b"\n").await?;
        }
        self.stream.flush().await?;
        return Ok(());
    }
}



#[derive(Debug)]
pub struct TcpReader {
    pub stream: OwnedReadHalf,
}

impl TcpReader {
    pub fn new(stream: OwnedReadHalf) -> Self {
        Self {
            stream: stream,
        }
    }

    pub async fn read_line(&mut self) -> Result<String, Error> {
        let mut buf = BytesMut::with_capacity(1024);
        let mut buffer: [u8; 1] = [0; 1];
        loop {
            let ux = self.stream.read(&mut buffer[..]).await.unwrap();
            if ux == 0 {
                return Err(Error::new(std::io::ErrorKind::Other, "No line found"))
            }
            if buffer[0] == b'\n' {
                break;
            }
            buf.put(&buffer[..]);
        }


        return Ok(String::from_utf8(buf.to_vec()).unwrap());
    }

    pub async fn read_message(&mut self) -> Result<Message, Error> {
        let x: String = self.read_line().await?;
        let seqs = self.read_line().await?;
        let vn: String = self.read_line().await?;
        let seq = match seqs.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return Err(Error::new(std::io::ErrorKind::Other, "Invalid sequence number")),
        };
        let mut msg = new_message(x, seq, vec![]);
        let n = match vn.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return Err(Error::new(std::io::ErrorKind::Other, "Invalid number of values")),
        };

        for _ in 0..n {
            let s = self.read_line().await?;
            msg.values.push(s);
        }

        return Ok(msg);
    }
}