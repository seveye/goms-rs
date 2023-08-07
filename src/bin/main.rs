use goms_rs::watch;
use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

#[tokio::main]
async fn main() {
    Builder::new()
    .format(|buf, record| {
        writeln!(buf,
            "{} [{}] -> {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        )
    })
    .filter(None, LevelFilter::Trace)
    .init();

    let mut cli = match watch::client::connect("127.0.0.1:16543", vec![String::from("nodeRegister:")], watch).await {
        Ok(client) => client,
        Err(e) => {
            log::warn!("Error: {}", e);
            return;
        }
    };

    match cli.initialize().await {
        Ok(_) => {
            log::debug!("initialize ok");
        },
        Err(e) => {
            log::warn!("Error: {}", e);
        }
    }

    //sleep
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

fn watch(key: &str, field: &str, value: &str) {
    log::trace!("watch: {} {} {}", key, field, value)
}