use std::{io::{Cursor, BufRead}, fs::read_link};

use bytes::{Buf, BytesMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufWriter}, net::TcpStream};

use crate::watch::message::Message;

use super::message::get_line;

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
            let mut msg = Message::new("".to_string(), 0, vec![]);

            let x = get_line(&mut buf);
            match x {
                Ok(x) => {
                    msg.cmd = String::from_utf8(x.to_vec()).unwrap();
                },
                Err(e) => {
                    ()
                }
            }

            let req = get_line(&mut buf);
            match req {
                Ok(req) => {
                    //to int64
                    msg.seq = String::from_utf8(req.to_vec()).unwrap().parse::<i64>().unwrap();
                },
                Err(e) => {
                    ()
                }
            }

            let mut n : i64 = 0;
            let vn = get_line(&mut buf);
            match vn {
                Ok(vn) => {
                    //to int64
                    n = String::from_utf8(vn.to_vec()).unwrap().parse::<i64>().unwrap();
                },
                Err(e) => {
                    ()
                }
            }

            for _ in 0..n  {
                let x = get_line(&mut buf);
                match x {
                    Ok(x) => {
                        msg.values.push(String::from_utf8(x.to_vec()).unwrap());
                    },
                    Err(e) => {
                        ()
                    }
                }
            }

            return Some(msg);
        }
    }
}