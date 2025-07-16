use tauri::{AppHandle, Emitter};
use serde_json::json;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Global app handle for logging from anywhere
static GLOBAL_APP_HANDLE: Mutex<Option<Arc<AppHandle>>> = Mutex::new(None);

/// Initialize the global logger with the app handle
pub fn init_global_logger(app: &AppHandle) {
    let mut handle = GLOBAL_APP_HANDLE.lock().unwrap();
    *handle = Some(Arc::new(app.clone()));
}

/// Log levels for the launcher
#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warning => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Debug => write!(f, "debug"),
        }
    }
}

/// Centralized logging system for the launcher
pub struct Logger;

impl Logger {
    /// Log a message to the frontend logging system
    pub fn log(app: &AppHandle, level: LogLevel, message: &str, instance_id: Option<&str>) {
        let log_data = json!({
            "level": level.to_string(),
            "message": message,
            "instanceId": instance_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Err(e) = app.emit_to("main", "launcher-log", log_data) {
            eprintln!("Failed to emit log event: {}", e);
        }
    }

    /// Helper for functions without AppHandle access - logs to both console and frontend
    pub fn console_log(level: LogLevel, message: &str, instance_id: Option<&str>) {
        // Always log to console first (fallback)
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let instance_str = instance_id.map(|id| format!(" [{}]", id)).unwrap_or_default();
        
        match level {
            LogLevel::Error => eprintln!("[{}] ERROR{}: {}", timestamp, instance_str, message),
            LogLevel::Warning => eprintln!("[{}] WARN{}: {}", timestamp, instance_str, message),
            LogLevel::Info => println!("[{}] INFO{}: {}", timestamp, instance_str, message),
            LogLevel::Debug => println!("[{}] DEBUG{}: {}", timestamp, instance_str, message),
        }
        
        // Also try to log to frontend if app handle is available
        if let Ok(handle_guard) = GLOBAL_APP_HANDLE.lock() {
            if let Some(app_handle) = handle_guard.as_ref() {
                Self::log(app_handle, level, message, instance_id);
            }
        }
    }

    /// Log an info message
    pub fn info(app: &AppHandle, message: &str, instance_id: Option<&str>) {
        Self::log(app, LogLevel::Info, message, instance_id);
    }

    /// Log a warning message
    pub fn warn(app: &AppHandle, message: &str, instance_id: Option<&str>) {
        Self::log(app, LogLevel::Warning, message, instance_id);
    }

    /// Log an error message
    pub fn error(app: &AppHandle, message: &str, instance_id: Option<&str>) {
        Self::log(app, LogLevel::Error, message, instance_id);
    }

    /// Log a debug message
    pub fn debug(app: &AppHandle, message: &str, instance_id: Option<&str>) {
        Self::log(app, LogLevel::Debug, message, instance_id);
    }

    /// Log a formatted message (like println! but to the logging system)
    pub fn log_fmt(app: &AppHandle, level: LogLevel, args: fmt::Arguments<'_>, instance_id: Option<&str>) {
        Self::log(app, level, &format!("{}", args), instance_id);
    }
    
    /// Convenient global logging methods that work without AppHandle
    pub fn info_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Info, message, instance_id);
    }
    
    pub fn warn_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Warning, message, instance_id);
    }
    
    pub fn error_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Error, message, instance_id);
    }
    
    pub fn debug_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Debug, message, instance_id);
    }
}

/// Convenience macros for logging (similar to println! but sends to frontend)
#[macro_export]
macro_rules! log_info {
    ($app:expr, $instance_id:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Info, format_args!($($arg)*), $instance_id)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($app:expr, $instance_id:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Warning, format_args!($($arg)*), $instance_id)
    };
}

#[macro_export]
macro_rules! log_error {
    ($app:expr, $instance_id:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Error, format_args!($($arg)*), $instance_id)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($app:expr, $instance_id:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Debug, format_args!($($arg)*), $instance_id)
    };
}

/// For functions that don't have instance_id, provide convenience versions
#[macro_export]
macro_rules! log_info_global {
    ($app:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Info, format_args!($($arg)*), None)
    };
}

#[macro_export]
macro_rules! log_warn_global {
    ($app:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Warning, format_args!($($arg)*), None)
    };
}

#[macro_export]
macro_rules! log_error_global {
    ($app:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Error, format_args!($($arg)*), None)
    };
}

#[macro_export]
macro_rules! log_debug_global {
    ($app:expr, $($arg:tt)*) => {
        $crate::logging::Logger::log_fmt($app, $crate::logging::LogLevel::Debug, format_args!($($arg)*), None)
    };
}

/// Export logs to a file for debugging or support purposes
#[tauri::command]
pub async fn export_logs(instance_id: Option<String>) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    
    let log_content = if let Some(ref id) = instance_id {
        format!("Logs for instance: {}\n\n[Sample log entries for instance {}]", id, id)
    } else {
        "Global launcher logs\n\n[Sample global log entries]".to_string()
    };
    
    let filename = if let Some(ref id) = instance_id {
        format!("kable_logs_{}.txt", id)
    } else {
        "kable_logs_global.txt".to_string()
    };
    
    let mut file = File::create(&filename)
        .map_err(|e| format!("Failed to create log file: {}", e))?;
    
    file.write_all(log_content.as_bytes())
        .map_err(|e| format!("Failed to write log file: {}", e))?;
    
    Logger::info_global(&format!("Logs exported to: {}", filename), instance_id.as_deref());
    Ok(())
}

/// Standalone convenience functions for easier usage (backward compatibility)
pub fn info(message: &str) {
    Logger::info_global(message, None);
}

pub fn error(message: &str) {
    Logger::error_global(message, None);
}

pub fn debug(message: &str) {
    Logger::debug_global(message, None);
}
