use clap::{App, Arg};
use std::sync::Arc;
use sam_utils::{Logger, LogLevel};
use tokio::time::{sleep, Duration};

async fn exec(exec_path: &str, task_id: usize, logger: Arc<Logger>) -> Option<tokio::process::Child> {
    logger.info(format!("Spawning child from {}", exec_path)).await;

    let mut parts = exec_path.split_whitespace();
    let command = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();

    match tokio::process::Command::new(command).args(args).spawn() {
        Ok(child) => {
            logger.info(format!("Task {}: Spawned child from {}", task_id, exec_path)).await;
            Some(child)
        },
        Err(e) => {
            logger.error(format!("Task {}: Failed to spawn child from {} - Error: {:?}", task_id, exec_path, e)).await;
            None
        }
    }
}

async fn spawn_process(exec_path: &str, task_id: usize, poll_rate: u64, logger: Arc<Logger>) {
    'restart: loop {
        let mut child = exec(exec_path, task_id, logger.clone()).await;

        loop {
            tokio::select! {
                status = child.as_mut().unwrap().wait() => {
                    match status {
                        Ok(status) => {
                            if status.success() {
                                logger.info(format!("Task {}: Child exited successfully", task_id)).await;
                                return;
                            } else {
                                logger.warn(format!("Task {}: Child exited with status {:?}", task_id, status)).await;
                                continue 'restart;
                            }
                        }
                        Err(e) => {
                            logger.error(format!("Task {}: Error waiting for child process: {:?}", task_id, e)).await;
                            continue 'restart;
                        }
                    }
                }

                else => {
                }
            }

            sleep(Duration::from_secs(poll_rate)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new("Sam Watchdog")
        .version("1.0.0")
        .arg(
            Arg::new("log_file")
                .short('f')
                .long("log-file")
                .value_name("FILE")
                .takes_value(true)
                .default_value("striker.log"),
        )
        .get_matches();

    let log_file = matches.value_of("log_file").unwrap();
    let logger = Arc::new(Logger::new(log_file, LogLevel::Info).await.unwrap());

    let binaries = vec![
        "./sam_dap",
        "./sam_rep"
    ];

    let mut tasks = vec![];

    for (i, bin) in binaries.iter().enumerate() {
        let task_id = i + 1;
        let bin = bin.to_string();
        let logger = logger.clone();
        tasks.push(tokio::spawn(async move {
            spawn_process(&bin, task_id, 60, logger).await;
        }));
    }

    for task in tasks {
        let _ = task.await;
    }
}