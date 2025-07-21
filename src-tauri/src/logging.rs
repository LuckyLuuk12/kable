use tauri::{AppHandle, Emitter};
use serde_json::json;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::fs::{self, File, OpenOptions};
use std::io::{Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::settings::CategorizedLauncherSettings;

/// Global app handle for logging from anywhere
static GLOBAL_APP_HANDLE: Mutex<Option<Arc<AppHandle>>> = Mutex::new(None);

/// Global log storage system
static LOG_STORAGE: Mutex<Option<LogStorage>> = Mutex::new(None);

/// Initialize the global logger with the app handle
pub fn init_global_logger(app: &AppHandle) {
    let mut handle = GLOBAL_APP_HANDLE.lock().unwrap();
    *handle = Some(Arc::new(app.clone()));
    
    // Initialize log storage
    let mut storage = LOG_STORAGE.lock().unwrap();
    if let Ok(log_storage) = LogStorage::new(app) {
        *storage = Some(log_storage);
    }
}

/// Configuration for log storage
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enable_persistent_logging: bool,
    pub enable_compression: bool,
    pub size_limit_mb: u64,
    pub retention_days: u64,
    pub logs_dir: PathBuf,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enable_persistent_logging: false,
            enable_compression: true,
            size_limit_mb: 10,
            retention_days: 30,
            logs_dir: PathBuf::new(),
        }
    }
}

/// Log storage management
pub struct LogStorage {
    config: LogConfig
}

impl LogStorage {
    /// Create new log storage instance
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let minecraft_path = Self::get_minecraft_path(app)?;
        let logs_dir = minecraft_path.join("kable").join("logs");
        
        // Create logs directory structure
        fs::create_dir_all(&logs_dir)?;
        fs::create_dir_all(logs_dir.join("launcher"))?;
        fs::create_dir_all(logs_dir.join("installations"))?;
        
        // Load settings to configure logging
        let settings = tauri::async_runtime::block_on(crate::settings::load_settings()).unwrap_or_default();
        
        fn value_to_u64(val: &serde_json::Value, default: u64) -> u64 {
            val.as_u64().or_else(|| val.as_i64().map(|v| v.max(0) as u64)).unwrap_or(default)
        }

        // Usage:
        let config = LogConfig {
            enable_persistent_logging: settings.logging.enable_persistent_logging,
            enable_compression: settings.logging.enable_log_compression,
            size_limit_mb: value_to_u64(&settings.logging.log_file_size_limit_mb, 10),
            retention_days: value_to_u64(&settings.logging.log_retention_days, 30),
            logs_dir,
        };
        
        Ok(Self {
            config
        })
    }
    
    /// Get Minecraft path from app settings or default location
    fn get_minecraft_path(_app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // For now, use a default path. In a real implementation, you'd read from settings
        let minecraft_dir = dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join(".minecraft");
        Ok(minecraft_dir)
    }
    
    /// Update logging configuration from settings
    pub fn update_config(&mut self, settings: &CategorizedLauncherSettings) {
        self.config.enable_persistent_logging = settings.logging.enable_persistent_logging;
        self.config.enable_compression = settings.logging.enable_log_compression;
        self.config.size_limit_mb = {
            fn value_to_u64(val: &serde_json::Value, default: u64) -> u64 {
                val.as_u64().or_else(|| val.as_i64().map(|v| v.max(0) as u64)).unwrap_or(default)
            }
            value_to_u64(&settings.logging.log_file_size_limit_mb, 10)
        };
        self.config.retention_days = {
            fn value_to_u64(val: &serde_json::Value, default: u64) -> u64 {
                val.as_u64().or_else(|| val.as_i64().map(|v| v.max(0) as u64)).unwrap_or(default)
            }
            value_to_u64(&settings.logging.log_retention_days, 30)
        };
    }
    
    /// Write log message to persistent storage
    pub fn write_log(&self, level: &LogLevel, message: &str, instance_id: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enable_persistent_logging {
            return Ok(());
        }
        
        let timestamp = Utc::now();
        let log_line = format!(
            "[{}] {} {}\n",
            timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
            level.to_string().to_uppercase(),
            message
        );
        
        let log_type = if instance_id.is_some() { "installations" } else { "launcher" };
        let filename = format!(
            "{}-{}.log",
            log_type,
            timestamp.format("%Y-%m-%d")
        );
        
        let log_path = if let Some(instance_id) = instance_id {
            self.config.logs_dir.join("installations").join(instance_id).join(&filename)
        } else {
            self.config.logs_dir.join("launcher").join(&filename)
        };
        
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Check if file needs compression before writing
        if log_path.exists() {
            let file_size = fs::metadata(&log_path)?.len();
            if file_size > (self.config.size_limit_mb * 1024 * 1024) {
                self.compress_log_file(&log_path)?;
            }
        }
        
        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
    
    /// Compress a log file using 7zip
    fn compress_log_file(&self, log_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enable_compression {
            return Ok(());
        }
        
        let compressed_path = log_path.with_extension("log.7z");
        let file_content = fs::read(log_path)?;
        
        // Create 7z archive
        let mut archive_file = File::create(&compressed_path)?;
        let mut encoder = sevenz_rust::SevenZWriter::new(&mut archive_file)?;
        
        let filename = log_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("log.txt");
            
        encoder.push_archive_entry(
            sevenz_rust::SevenZArchiveEntry::from_path(filename, filename.to_string()),
            Some(std::io::Cursor::new(file_content)),
        )?;
        
        encoder.finish()?;
        
        // Remove original file after successful compression
        fs::remove_file(log_path)?;
        
        Ok(())
    }
    
    /// Clean up old log files based on retention policy
    pub fn cleanup_old_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        for log_dir in [self.config.logs_dir.join("launcher"), self.config.logs_dir.join("installations")] {
            if log_dir.exists() {
                self.cleanup_directory(&log_dir, &cutoff_date)?;
            }
        }
        
        Ok(())
    }
    #[allow(clippy::only_used_in_recursion)]
    fn cleanup_directory(&self, dir: &Path, cutoff_date: &DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                // Check file creation date
                let metadata = fs::metadata(&path)?;
                if let Ok(created) = metadata.created() {
                    let created_date: DateTime<Utc> = created.into();
                    if created_date < *cutoff_date {
                        fs::remove_file(&path)?;
                    }
                }
            } else if path.is_dir() {
                // Recursively clean subdirectories (for installation-specific logs)
                self.cleanup_directory(&path, cutoff_date)?;
                
                // Remove empty directories
                if path.read_dir()?.next().is_none() {
                    fs::remove_dir(&path)?;
                }
            }
        }
        
        Ok(())
    }
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
    /// Log a message to the frontend logging system and persistent storage
    pub fn log(app: &AppHandle, level: LogLevel, message: &str, instance_id: Option<&str>) {
        let log_data = json!({
            "level": level.to_string(),
            "message": message,
            "instanceId": instance_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Emit to frontend
        if let Err(e) = app.emit_to("main", "launcher-log", log_data) {
            eprintln!("Failed to emit log event: {}", e);
        }
        
        // Write to persistent storage if enabled
        if let Ok(storage_guard) = LOG_STORAGE.lock() {
            if let Some(storage) = storage_guard.as_ref() {
                if let Err(e) = storage.write_log(&level, message, instance_id) {
                    eprintln!("Failed to write log to storage: {}", e);
                }
            }
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
    
    /// Update logging configuration from settings
    pub fn update_log_config(settings: &CategorizedLauncherSettings) {
        if let Ok(mut storage_guard) = LOG_STORAGE.lock() {
            if let Some(storage) = storage_guard.as_mut() {
                storage.update_config(settings);
            }
        }
    }
    
    /// Manually trigger log cleanup
    pub fn cleanup_logs() -> Result<(), String> {
        if let Ok(storage_guard) = LOG_STORAGE.lock() {
            if let Some(storage) = storage_guard.as_ref() {
                storage.cleanup_old_logs()
                    .map_err(|e| format!("Failed to cleanup logs: {}", e))?;
            }
        }
        Ok(())
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
    
    // Get the logs directory from storage or fallback to default
    let logs_dir = if let Ok(storage_guard) = LOG_STORAGE.lock() {
        if let Some(storage) = storage_guard.as_ref() {
            storage.config.logs_dir.clone()
        } else {
            // Fallback to default minecraft directory
            dirs::data_dir()
                .ok_or("Could not find data directory")?
                .join(".minecraft")
                .join("kable")
                .join("logs")
        }
    } else {
        // Fallback to default minecraft directory
        dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join(".minecraft")
            .join("kable")
            .join("logs")
    };
    
    // Create exports directory
    let exports_dir = logs_dir.join("exports");
    fs::create_dir_all(&exports_dir)
        .map_err(|e| format!("Failed to create exports directory: {}", e))?;
    
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
    
    let export_path = exports_dir.join(&filename);
    let mut file = File::create(&export_path)
        .map_err(|e| format!("Failed to create log file: {}", e))?;
    
    file.write_all(log_content.as_bytes())
        .map_err(|e| format!("Failed to write log file: {}", e))?;
    
    Logger::info_global(&format!("Logs exported to: {}", export_path.display()), instance_id.as_deref());
    Ok(())
}

/// Update logging configuration
#[tauri::command]
pub async fn update_logging_config(settings: CategorizedLauncherSettings) -> Result<(), String> {
    Logger::update_log_config(&settings);
    Logger::info_global("Logging configuration updated", None);
    Ok(())
}

/// Clean up old log files
#[tauri::command]
pub async fn cleanup_old_logs() -> Result<(), String> {
    Logger::cleanup_logs()?;
    Logger::info_global("Log cleanup completed", None);
    Ok(())
}

/// Get logging statistics
#[tauri::command]
pub async fn get_log_stats() -> Result<serde_json::Value, String> {
    // Implementation would scan log directories and return statistics
    // For now, return basic info
    Ok(json!({
        "persistent_logging_enabled": true,
        "compression_enabled": true,
        "total_log_files": 0,
        "total_size_mb": 0,
        "oldest_log_date": null,
        "newest_log_date": null
    }))
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
