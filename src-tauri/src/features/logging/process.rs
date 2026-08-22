use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Debug => "debug",
        };

        write!(f, "{value}")
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

    /// Unix timestamp in milliseconds.
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

    /// Strict frontend contract.
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

#[derive(Debug, Clone)]
struct ParsedLogPrefix {
    context: String,
    level: LogLevel,
}

/// Stateful parser for Minecraft/Log4j-style process output.
///
/// Expected format:
/// `[HH:mm:ss] [thread/LEVEL]: message`
///
/// Lines without a recognized prefix are treated as continuation lines
/// of the previous structured log entry when possible.
#[derive(Debug, Default)]
pub struct ProcessLogParser {
    current_context: Option<String>,
    current_level: Option<LogLevel>,
}

impl ProcessLogParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_line(&mut self, line: &str) -> LogEvent {
        let timestamp = current_timestamp();

        if let Some(prefix) = parse_log_prefix(line) {
            self.current_context = Some(prefix.context.clone());
            self.current_level = Some(prefix.level.clone());

            let message = extract_log_message(line);

            return LogEvent::new(prefix.level, message, None, None, Some(prefix.context), None, timestamp);
        }

        let level = self.current_level.clone().unwrap_or(LogLevel::Info);
        let context = self.current_context.clone();

        LogEvent::new(level, line, None, None, context, None, timestamp)
    }

    pub fn reset(&mut self) {
        self.current_context = None;
        self.current_level = None;
    }
}

pub fn parse_process_line(line: &str) -> LogEvent {
    ProcessLogParser::new().parse_line(line)
}

fn parse_log_prefix(line: &str) -> Option<ParsedLogPrefix> {
    let remainder = line.strip_prefix('[')?;

    let timestamp_end = remainder.find("] [")?;
    let remainder = &remainder[timestamp_end + 3..];

    let prefix_end = remainder.find("]: ")?;
    let prefix = &remainder[..prefix_end];

    let level_start = prefix.rfind('/')?;

    let context = prefix[..level_start].trim();
    let level = prefix[level_start + 1..].trim();

    if context.is_empty() || level.is_empty() {
        return None;
    }

    let level = match level {
        "TRACE" => LogLevel::Debug,
        "DEBUG" => LogLevel::Debug,
        "INFO" => LogLevel::Info,
        "WARN" | "WARNING" => LogLevel::Warn,
        "ERROR" | "FATAL" => LogLevel::Error,
        _ => return None,
    };

    Some(ParsedLogPrefix { context: context.to_string(), level })
}

fn extract_log_message(line: &str) -> String {
    let Some(remainder) = line.strip_prefix('[') else {
        return line.to_string();
    };

    let Some(timestamp_end) = remainder.find("] [") else {
        return line.to_string();
    };

    let remainder = &remainder[timestamp_end + 3..];

    let Some(message_start) = remainder.find("]: ") else {
        return line.to_string();
    };

    remainder[message_start + 3..].to_string()
}

fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis() as i64).unwrap_or_default()
}
