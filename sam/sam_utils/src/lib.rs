// sam_utils::lib.rs
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use chrono::Local;
use std::process;
use tokio::io::Error;

pub type LogFile = Mutex<File>;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Info,
    Warn,
    Debug,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Debug => "DEBUG",
        }
    }
}

pub struct Logger {
    log_file: LogFile,
    min_log_level: LogLevel,
}

impl Logger {
    pub async fn new(log_file_path: &str, min_log_level: LogLevel) -> Result<Self, Error> {
        let file = File::create(log_file_path).await?;

        let log_file: LogFile = Mutex::new(file);

        Ok(Logger { log_file, min_log_level })
    }

    async fn _log(&self, level: LogLevel, message: String) {
        if level >= self.min_log_level {
            let mut fd = self.log_file.lock().await;
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let pid = process::id();

            let log_level_str = level.as_str();
            let msg = format!("[{}][PID {}][{}][{}] {}\n", timestamp, pid, log_level_str, pid, message);

            if let Err(e) = fd.write_all(msg.as_bytes()).await {
                eprintln!("Failed to write to log file: {:?}", e);
            }
        }
    }

    pub async fn error(&self, message: String) {
        self._log(LogLevel::Error, message).await;
    }

    pub async fn info(&self, message: String) {
        self._log(LogLevel::Info, message).await;
    }

    pub async fn warn(&self, message: String) {
        self._log(LogLevel::Warn, message).await;
    }

    pub async fn debug(&self, message: String) {
        self._log(LogLevel::Debug, message).await;
    }
}