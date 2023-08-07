use crate::watch::connect::{TcpReader, TcpWriter};
use anyhow::Result;
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

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
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
    pub async fn send_message(&mut self, msg: &mut Message) -> Result<Message> {
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

    pub async fn initialize(&mut self) -> Result<()> {
        let mut req = new_message(String::from("initialize"), 0, self.keys.clone());
        self.send_message(&mut req).await?;
        Ok(())
    }

    pub async fn heartbeat(&mut self) -> Result<()> {
        let mut req = new_message(String::from("heartbeat"), 0, vec![]);
        self.send_message(&mut req).await?;
        Ok(())
    }

    pub async fn hget(&mut self, key: &str, field: &str) -> Result<String> {
        let mut req = new_message(
            String::from("hget"),
            0,
            vec![String::from(key), String::from(field)],
        );
        let rsp = self.send_message(&mut req).await?;
        if rsp.values.len() == 0 {
            return Err(anyhow::anyhow!("hget error"));
        }
        Ok(rsp.values[0].clone())
    }

    pub async fn hgetall(&mut self, key: &str) -> Result<Vec<String>> {
        let mut req = new_message(String::from("hgetall"), 0, vec![String::from(key)]);
        let rsp = self.send_message(&mut req).await?;
        Ok(rsp.values)
    }

    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut req = new_message(
            String::from("hset"),
            0,
            vec![String::from(key), String::from(field), String::from(value)],
        );
        let rsp = self.send_message(&mut req).await?;
        if rsp.values.len() == 0 {
            return Err(anyhow::anyhow!("hset error"));
        }
        Ok(())
    }

    pub async fn key_prefix(&mut self, prefix: &str) -> Result<Vec<String>> {
        let mut req = new_message(String::from("key_prefix"), 0, vec![String::from(prefix)]);
        let rsp = self.send_message(&mut req).await?;
        Ok(rsp.values)
    }

    pub async fn del(&mut self, key: &str) -> Result<()> {
        let mut req = new_message(String::from("del"), 0, vec![String::from(key)]);
        let rsp = self.send_message(&mut req).await?;
        if rsp.values.len() == 0 {
            return Err(anyhow::anyhow!("del error"));
        }
        Ok(())
    }

    pub async fn hincrby(&mut self, key: &str, field: &str, value: i64) -> Result<i64> {
        let mut req = new_message(
            String::from("hincrby"),
            0,
            vec![String::from(key), String::from(field), value.to_string()],
        );
        let rsp = self.send_message(&mut req).await?;
        if rsp.values.len() == 0 {
            return Err(anyhow::anyhow!("hincrby error"));
        }
        let r: i64 = rsp.values[0].parse::<i64>()?;
        Ok(r)
    }
}
