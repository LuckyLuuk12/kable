use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Debug => "debug",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,

    pub fn_name: Option<String>,
    pub instance_id: Option<String>,
    pub context: Option<String>,

    pub value: Option<Value>,

    /// Unix timestamp in milliseconds
    pub timestamp: i64,
}

impl LogEvent {
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        fn_name: Option<String>,
        instance_id: Option<String>,
        context: Option<String>,
        value: Option<Value>,
        timestamp: i64,
    ) -> Self {
        Self { level, message: message.into(), fn_name, instance_id, context, value, timestamp }
    }

    /// Strict frontend contract
    pub fn to_frontend(&self) -> Value {
        serde_json::json!({
            "level": self.level.to_string(),
            "message": self.message,
            "fnName": self.fn_name,
            "instanceId": self.instance_id,
            "context": self.context,
            "value": self.value,
            "timestamp": self.timestamp,
        })
    }
}
