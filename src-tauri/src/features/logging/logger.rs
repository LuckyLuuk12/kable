use crate::features::logging::manager::LogManager;
use crate::features::logging::process::{LogEvent, LogLevel};
use serde_json::json;
use std::fmt;
use tauri::AppHandle;

/// Pure facade API layer
/// - no IO
/// - no threads
/// - no batching
/// - no persistence awareness
pub struct Logger;

impl Logger {
    pub fn init(app: &AppHandle) {
        LogManager::init(app);
    }

    pub fn log(level: LogLevel, message: &str, instance_id: Option<&str>) {
        let app = crate::app_handle();
        let event =
            LogEvent::new(level, message, None, instance_id.map(|s| s.to_string()), None, None, chrono::Utc::now().timestamp_millis());

        LogManager::emit(event);
        let _ = app;
    }

    pub fn log_fmt(level: LogLevel, args: fmt::Arguments<'_>, instance_id: Option<&str>) {
        Self::log(level, &format!("{}", args), instance_id);
    }

    pub fn console_log(level: LogLevel, message: &str, instance_id: Option<&str>) {
        match level {
            LogLevel::Info => println!("[INFO] {}", message),
            LogLevel::Warn => println!("[WARN] {}", message),
            LogLevel::Error => eprintln!("[ERROR] {}", message),
            LogLevel::Debug => println!("[DEBUG] {}", message),
        }

        if let Some(id) = instance_id {
            println!("instance: {}", id);
        }
    }

    pub fn info(message: &str, instance_id: Option<&str>) {
        Self::log(LogLevel::Info, message, instance_id);
    }

    pub fn warn(message: &str, instance_id: Option<&str>) {
        Self::log(LogLevel::Warn, message, instance_id);
    }

    pub fn error(message: &str, instance_id: Option<&str>) {
        Self::log(LogLevel::Error, message, instance_id);
    }

    pub fn debug(message: &str, instance_id: Option<&str>) {
        Self::log(LogLevel::Debug, message, instance_id);
    }

    pub fn info_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Info, message, instance_id);
    }

    pub fn warn_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Warn, message, instance_id);
    }

    pub fn error_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Error, message, instance_id);
    }

    pub fn debug_global(message: &str, instance_id: Option<&str>) {
        Self::console_log(LogLevel::Debug, message, instance_id);
    }

    /// proc-macro compatibility layer
    pub fn log_success(fn_name: &str, value: Option<String>, instance_id: Option<String>, _context: String) {
        let message = match value {
            Some(v) => format!("{} | {}", fn_name, v),
            None => fn_name.to_string(),
        };

        LogManager::console_emit(LogLevel::Info, &message, instance_id.as_deref());
    }

    pub fn log_error(fn_name: &str, error: &dyn std::fmt::Display, instance_id: Option<String>, _context: String) {
        let message = format!("{} | error: {}", fn_name, error);

        LogManager::console_emit(LogLevel::Error, &message, instance_id.as_deref());
    }

    pub fn update_log_config(settings: &api_types::settings::CategorizedLauncherSettings) {
        LogManager::update_settings(settings);
    }

    pub fn cleanup_logs() -> Result<(), String> {
        LogManager::cleanup()
    }

    pub fn export_logs(instance_id: Option<String>) -> Result<(), String> {
        LogManager::export(instance_id)
    }
}
