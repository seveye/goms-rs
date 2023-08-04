use crate::watch::connect::{TcpReader, TcpWriter};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc, Mutex},
};

use super::{message::new_message, Message};

pub struct Client {
    pub connection: TcpWriter,
    map: Arc<Mutex<HashMap<i64, mpsc::Sender<Message>>>>,
    seq: Mutex<i64>,
    pub keys: Vec<String>,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, std::io::Error> {
    let socket: tokio::net::TcpStream = tokio::net::TcpStream::connect(addr).await?;
    let (read, write) = socket.into_split();
    let mut reader = TcpReader::new(read);
    let db = Arc::new(Mutex::new(HashMap::<i64, mpsc::Sender<Message>>::new()));
    let adb = db.clone();
    tokio::spawn(async move {
        loop {
            let msg: crate::watch::Message = reader.read_message().await.unwrap();
            log::info!("msg: {:?}", msg);
            let db = adb.lock().await;
            match db.get(&msg.seq) {
                Some(sender) => {
                    sender.send(msg).await.unwrap();
                }
                None => {}
            }
        }
    });

    Ok(Client {
        connection: TcpWriter::new(write),
        // map: HashMap::new(),
        map: db.clone(),
        seq: 0.into(),
        keys: vec![],
    })
}

impl Client {
    pub async fn send_message(&mut self, msg: &mut Message) -> Result<Message, std::io::Error> {
        let (tx, mut rx) = mpsc::channel::<Message>(1);

        let mut seq = self.seq.lock().await;
        *seq += 1;
        msg.seq = *seq;

        {
            let mut db = self.map.lock().await;
            db.insert(msg.seq, tx);
        }

        self.connection.write_message(msg).await?;

        Ok(rx.recv().await.unwrap())
    }

    pub async fn initialize(&mut self) -> Result<(), std::io::Error> {
        let mut msg = new_message(String::from("initialize"), 0, self.keys.clone());
        match self.send_message(&mut msg).await {
            Ok(_) => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn heartbeat(&mut self) -> Result<(), std::io::Error> {
        let mut msg = new_message(String::from("heartbeat"), 0, vec![]);
        match self.send_message(&mut msg).await {
            Ok(_) => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn hget(&mut self, key: &str, field: &str) -> Result<String, std::io::Error> {
        let mut msg = new_message(
            String::from("hget"),
            0,
            vec![String::from(key), String::from(field)],
        );
        match self.send_message(&mut msg).await {
            Ok(msg) => {
                if msg.values.len() > 0 {
                    Ok(msg.values[0].clone())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "hget error"))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn hgetall(&mut self, key: &str) -> Result<Vec<String>, std::io::Error> {
        let mut msg = new_message(String::from("hgetall"), 0, vec![String::from(key)]);
        match self.send_message(&mut msg).await {
            Ok(msg) => Ok(msg.values),
            Err(e) => Err(e),
        }
    }

    pub async fn hset(
        &mut self,
        key: &str,
        field: &str,
        value: &str,
    ) -> Result<(), std::io::Error> {
        let mut msg = new_message(
            String::from("hset"),
            0,
            vec![String::from(key), String::from(field), String::from(value)],
        );
        match self.send_message(&mut msg).await {
            Ok(msg) => {
                if msg.values.len() > 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "hget error"))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn key_prefix(&mut self, prefix: &str) -> Result<Vec<String>, std::io::Error> {
        let mut msg = new_message(String::from("key_prefix"), 0, vec![String::from(prefix)]);
        match self.send_message(&mut msg).await {
            Ok(msg) => Ok(msg.values),
            Err(e) => Err(e),
        }
    }

    pub async fn del(&mut self, key: &str) -> Result<(), std::io::Error> {
        let mut msg = new_message(String::from("del"), 0, vec![String::from(key)]);
        match self.send_message(&mut msg).await {
            Ok(msg) => {
                if msg.values.len() > 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "del error"))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn hincrby(
        &mut self,
        key: &str,
        field: &str,
        value: i64,
    ) -> Result<i64, std::io::Error> {
        let mut msg = new_message(
            String::from("hincrby"),
            0,
            vec![String::from(key), String::from(field), value.to_string()],
        );
        match self.send_message(&mut msg).await {
            Ok(msg) => {
                if msg.values.len() > 0 {
                    match msg.values[0].parse::<i64>() {
                        Ok(v) => Ok(v),
                        Err(e) => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        )),
                    }
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "hget error"))
                }
            }
            Err(e) => Err(e),
        }
    }
}
