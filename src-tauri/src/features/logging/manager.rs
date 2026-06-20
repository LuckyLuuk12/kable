use crate::features::logging::emitter::LogEmitter;
use crate::features::logging::file::LogFileWriter;
use crate::features::logging::process::{LogEvent, LogLevel};
use api_types::settings::CategorizedLauncherSettings;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

/// Central orchestration layer
/// Owns emitter + file pipeline and routes events
pub struct LogManager {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    emitter: Option<LogEmitter>,
    file: Option<LogFileWriter>,
}

impl LogManager {
    pub fn init(app: &AppHandle) {
        let manager = Self::global();
        let mut inner = manager.inner.lock().unwrap();

        inner.emitter = Some(LogEmitter::new(app.clone()));
        inner.file = Some(LogFileWriter::new(app));
    }

    /// Core routing entrypoint
    pub fn emit(event: LogEvent) {
        let manager = Self::global();
        let inner = manager.inner.lock().unwrap();

        if let Some(emitter) = &inner.emitter {
            emitter.send(&event);
        }

        if let Some(file) = &inner.file {
            file.write(&event);
        }
    }

    /// Console fallback (no state, no dependencies)
    pub fn console_emit(level: LogLevel, message: &str, instance_id: Option<&str>) {
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

    pub fn update_settings(settings: &CategorizedLauncherSettings) {
        let manager = Self::global();
        let inner = manager.inner.lock().unwrap();

        if let Some(file) = &inner.file {
            file.update_settings(settings);
        }

        if let Some(emitter) = &inner.emitter {
            emitter.update_settings(settings);
        }
    }

    pub fn cleanup() -> Result<(), String> {
        let manager = Self::global();
        let inner = manager.inner.lock().unwrap();

        if let Some(file) = &inner.file {
            file.cleanup().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn export(instance_id: Option<String>) -> Result<(), String> {
        let manager = Self::global();
        let inner = manager.inner.lock().unwrap();

        if let Some(file) = &inner.file {
            file.export(instance_id).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn global() -> &'static Self {
        use std::sync::OnceLock;

        static INSTANCE: OnceLock<LogManager> = OnceLock::new();

        INSTANCE.get_or_init(|| LogManager { inner: Arc::new(Mutex::new(Inner { emitter: None, file: None })) })
    }
}
