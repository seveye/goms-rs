use goms_rs::watch;

#[tokio::main]
async fn main() {
    let mut cli = match watch::client::connect("127.0.0.1:16543").await {
        Ok(client) => client,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let msg = watch::message::new_message(String::from("hget"), 1, vec![String::from("nodeRegister:"), String::from("game0")]);
    cli.connection.write_message(&msg).await.unwrap();

    //sleep
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
}
