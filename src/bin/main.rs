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

    match cli.key_prefix("nodeRegister").await {
        Ok(values) => {
            println!("values: {:?}", values);
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    //sleep
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
}
