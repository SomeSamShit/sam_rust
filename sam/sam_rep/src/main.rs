use clap::{App, Arg};
use std::sync::Arc;
use tokio::io::Result;
use sam_utils::{Logger, LogLevel, REP_SOCK};
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::remove_file;

async fn handle_client(stream: UnixStream) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                println!("Received: {}", buffer.trim_end());
                if let Err(e) = writer.write_all(format!("Echo: {}\n", buffer).as_bytes()).await {
                    eprintln!("Failed to send response: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from client: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Sam Reputation")
    .version("1.0.0")
    .arg(
        Arg::new("log_file")
            .short('f')
            .long("log-file")
            .value_name("FILE")
            .takes_value(true)
            .default_value("rep.log"),
    )
    .get_matches();

    let log_file = matches.value_of("log_file").unwrap();
    let logger = Arc::new(Logger::new(log_file, LogLevel::Info).await.unwrap());

    logger.info(format!("rep init!")).await;

    remove_file(REP_SOCK).await.ok();

    let listener: UnixListener = UnixListener::bind(REP_SOCK)?;
    logger.info(format!("server listening on {}", REP_SOCK)).await;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_client(stream));
    }
}

