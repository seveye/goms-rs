use crate::watch::connect::{TcpReader, TcpWriter};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc, Mutex},
};

use super::{message::new_message, Message};

pub struct Client {
    // pub connection: TcpWriter,
    map: Arc<Mutex<HashMap<i64, tokio::sync::mpsc::Sender<Message>>>>,
    seq: Arc<Mutex<i64>>,
    pub keys: Vec<String>,
    tx: tokio::sync::mpsc::Sender<Message>,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
    let socket: tokio::net::TcpStream = tokio::net::TcpStream::connect(addr).await?;
    let (read, write) = socket.into_split();
    let db: Arc<Mutex<HashMap<i64, tokio::sync::mpsc::Sender<Message>>>> = Arc::new(Mutex::new(
        HashMap::<i64, tokio::sync::mpsc::Sender<Message>>::new(),
    ));

    //初始化
    let mut reader = TcpReader::new(read);
    let adb = db.clone();
    tokio::spawn(async move {
        loop {
            let msg: crate::watch::Message = reader.read_message().await.unwrap();
            //如果是客户端请求，消息通过channel返回给客户端
            let db = adb.lock().await;
            match db.get(&msg.seq) {
                Some(sender) => {
                    log::trace!("cleint->: {:?}", msg);
                    sender.send(msg).await.unwrap();
                    continue;
                }
                None => {}
            }

            //服务端推送
            log::trace!("watch->: {:?}", msg);
            if msg.cmd == "watch" {
            }
        }
    });

    //写消息
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(32);
    let mut connection = TcpWriter::new(write);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            log::trace!("write 1");
            connection.write_message(&msg).await.unwrap();
            log::trace!("write 2");
        }
    });

    //心跳
    let seq = Arc::new(Mutex::<i64>::new(0));

    let seq_clone = seq.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            log::trace!("heartbeat 1");
            {
                let mut l = seq_clone.lock().await;
                *l += 1;
                let req = new_message(String::from("heartbeat"), *l, vec![]);
                tx_clone.send(req).await.unwrap();
            }
            log::trace!("heartbeat 2");
        }
    });

    Ok(Client {
        // connection: TcpWriter::new(write),
        // map: HashMap::new(),
        map: db.clone(),
        seq: seq.clone(),
        keys: vec![],
        tx: tx,
    })
}

impl Client {
    pub async fn send_message(&mut self, msg: &mut Message) -> Result<Message> {
        let (tx, mut rx) = mpsc::channel::<Message>(1);

        {
            let mut seq = self.seq.lock().await;
            *seq += 1;
            msg.seq = *seq;
        }
        {
            let mut db = self.map.lock().await;
            db.insert(msg.seq, tx);
        }

        let req = new_message(msg.cmd.clone(), msg.seq, msg.values.clone());

        // self.connection.write_message(msg).await?;
        self.tx.send(req).await.unwrap();

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
        Ok(self.send_message(&mut req).await?.values)
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
        Ok(rsp.values[0].parse::<i64>()?)
    }
}
