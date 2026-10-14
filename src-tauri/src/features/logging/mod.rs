pub mod emitter;
pub mod file;
pub mod logger;
pub mod manager;
pub mod process;

pub use logger::Logger;
pub use process::{LogEvent, LogLevel};

/// Initialize logging system (call once during app startup)
pub fn init() {
    logger::Logger::init();
}

//? -------------------------
//? Proc-macro bridge layer
//? -------------------------

pub fn log_success(fn_name: &str, value: Option<String>, instance_id: Option<String>, context: String) {
    let message = match value {
        Some(v) => format!("{} | {}", fn_name, v),
        None => fn_name.to_string(),
    };

    manager::LogManager::console_emit(process::LogLevel::Info, &message, instance_id.as_deref());

    let _ = context;
}

pub fn log_error(fn_name: &str, error: &dyn std::fmt::Display, instance_id: Option<String>, context: String) {
    let message = format!("{} | error: {}", fn_name, error);

    manager::LogManager::console_emit(process::LogLevel::Error, &message, instance_id.as_deref());

    let _ = context;
}
