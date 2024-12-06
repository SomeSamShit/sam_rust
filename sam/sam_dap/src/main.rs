use clap::{App, Arg};
use tokio::time::sleep;
use std::sync::Arc;
use std::time::Duration;
use sam_utils::{Logger, LogLevel};
// use tokio::net::UnixStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::io::Result;

#[derive(Debug, Clone)]
struct RepCallData {
    domain: String,
    rep_score: u8,
}

async fn nfq_handler(logger: Arc<Logger>, tx: Sender<RepCallData>, mut rx: Receiver<RepCallData>) -> Result<()> {
    let rep_call_data = RepCallData {
        domain: String::from("google.com"),
        rep_score: 50,
    };

    loop {
        sleep(Duration::from_secs(10)).await;
        if let Err(e) = tx.send(rep_call_data.clone()).await {
            logger.error(format!("Failed to pass domain {} to rep thread - error: {}", rep_call_data.domain, e)).await;
            continue;
        }

        if let Some(data) = rx.recv().await {
            logger.info(format!("Received rep score from rep thread: {:?}", data)).await;
        }
    }
}

async fn rep_handler(logger: Arc<Logger>, tx: Sender<RepCallData>, mut rx: Receiver<RepCallData>) -> Result<()> {
    loop {
        if let Some(mut data) = rx.recv().await {
            logger.info(format!("Received data from nfq thread: {:?}", data)).await;

            data.rep_score =  100;
            logger.debug(format!("setting rep of {} to {}", data.domain, data.rep_score)).await;

            if let Err(e) = tx.send(data).await {
                logger.error(format!("Failed to send modified data back to nfq_handler. error: {}", e)).await;
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
            .default_value("dap.log"),
    )
    .get_matches();

    let log_file = matches.value_of("log_file").unwrap();
    let logger = Arc::new(Logger::new(log_file, LogLevel::Info).await.unwrap());

    logger.info(format!("dap init")).await;

    let (tx_nfq, rx_nfq) = channel(1);
    let (tx_rep, rx_rep) = channel(1);

    let logger_nfq = logger.clone();
    let nfq_task = tokio::spawn(async move {
        if let Err(e) = nfq_handler(logger_nfq, tx_nfq, rx_rep).await {
            eprintln!("nfq_handler error: {:?}", e);
        }
    });

    let logger_rep = logger.clone();
    let rep_task = tokio::spawn(async move {
        if let Err(e) = rep_handler(logger_rep, tx_rep, rx_nfq).await {
            eprintln!("rep_handler error: {:?}", e);
        }
    });

    // Wait for tasks to finish
    let _ = nfq_task.await.unwrap();
    let _ = rep_task.await.unwrap();

    Ok(())
}


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let socket_path = "/tmp/rust_tokio_ipc.sock";
//     let stream = UnixStream::connect(socket_path).await?;
//     let (reader, mut writer) = stream.into_split();
//     let mut reader = BufReader::new(reader);

//     // Send a message to the server
//     writer.write_all(b"Hello from client\n").await?;
//     writer.write_all(b"Another message\n").await?;

//     // Read responses
//     let mut buffer = String::new();
//     while reader.read_line(&mut buffer).await? != 0 {
//         print!("Server response: {}", buffer);
//         buffer.clear();
//     }

//     Ok(())
// }




