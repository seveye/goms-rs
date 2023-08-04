use tokio::net::ToSocketAddrs;
use crate::watch::connect::{TcpReader, TcpWriter};


pub struct Client {
    pub connection: TcpWriter,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, std::io::Error> {
    let socket: tokio::net::TcpStream = tokio::net::TcpStream::connect(addr).await?;
    let (read, write) = socket.into_split();

    let mut reader = TcpReader::new(read);
    tokio::spawn(async move {
        loop {
            let msg: crate::watch::Message = reader.read_message().await.unwrap();
            println!("{:?}", msg);
        }
    });

    Ok(Client { connection: TcpWriter::new(write) })
}

impl Client {}
