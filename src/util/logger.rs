use chrono::Utc;
use serde_json::{Map, Value, json};
use std::{env, fs::OpenOptions, io::Write, sync::Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn from_env() -> Self {
        match env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".into())
            .to_ascii_uppercase()
            .as_str()
        {
            "DEBUG" => Self::Debug,
            "WARN" => Self::Warn,
            "ERROR" => Self::Error,
            _ => Self::Info,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Logger {
    account_id: Option<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_account(&self, id: impl Into<String>) -> Self {
        Self {
            account_id: Some(id.into()),
        }
    }
    pub fn debug(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Debug, msg.as_ref(), None);
    }
    pub fn info(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Info, msg.as_ref(), None);
    }
    pub fn warn(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Warn, msg.as_ref(), None);
    }
    pub fn error(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Error, msg.as_ref(), None);
    }

    pub fn log(&self, level: LogLevel, msg: &str, extra: Option<Map<String, Value>>) {
        if level < LogLevel::from_env() {
            return;
        }
        let mut entry = Map::new();
        entry.insert("ts".into(), json!(Utc::now().to_rfc3339()));
        entry.insert("level".into(), json!(level.as_str()));
        entry.insert("msg".into(), json!(msg));
        if let Some(account) = &self.account_id {
            entry.insert("account".into(), json!(account));
        }
        if let Some(extra) = extra {
            entry.extend(extra);
        }
        let line = Value::Object(entry).to_string();
        write_line(&line);
    }
}

static WRITE_LOCK: Mutex<()> = Mutex::new(());

fn write_line(line: &str) {
    let _guard = WRITE_LOCK.lock().ok();
    if let Ok(path) = env::var("WEIXIN_LOG_FILE")
        && let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = writeln!(f, "{line}");
        return;
    }
    println!("{line}");
}

pub fn logger() -> Logger {
    Logger::new()
}
