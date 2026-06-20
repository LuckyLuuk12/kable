use crate::logging::process::LogEvent;
use crate::settings::CategorizedLauncherSettings;
use chrono::{DateTime, Utc};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{sync_channel, RecvTimeoutError, SyncSender, TrySendError},
    Arc, Mutex,
};
use std::time::Duration;

/// Persistent logging pipeline:
/// - file writing
/// - rotation
/// - compression
/// - cleanup
///
/// No frontend knowledge.
pub struct LogFileWriter {
    sender: SyncSender<LogEvent>,
    config: Arc<Mutex<FileConfig>>,
}

#[derive(Clone)]
struct FileConfig {
    enable_persistent: bool,
    enable_compression: bool,
    size_limit_mb: u64,
    retention_days: u64,
    logs_dir: PathBuf,
}

impl LogFileWriter {
    pub fn new(_app: tauri::AppHandle) -> Self {
        let (tx, rx) = sync_channel::<LogEvent>(1024);

        let logs_dir = crate::kable_dir().unwrap().join("logs");

        let config = FileConfig { enable_persistent: true, enable_compression: true, size_limit_mb: 10, retention_days: 30, logs_dir };

        let config_arc = Arc::new(Mutex::new(config.clone()));
        let config_thread = config_arc.clone();

        std::thread::spawn(move || {
            for event in rx.iter() {
                let cfg = config_thread.lock().unwrap().clone();

                if !cfg.enable_persistent {
                    continue;
                }

                let filename = format!(
                    "{}-{}.log",
                    if event.instance_id.is_some() { "installations" } else { "launcher" },
                    event.timestamp / 86_400_000
                );

                let path = if let Some(ref id) = event.instance_id {
                    cfg.logs_dir.join("installations").join(id).join(&filename)
                } else {
                    cfg.logs_dir.join("launcher").join(&filename)
                };

                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                Self::rotate_if_needed(&cfg, &path);

                let line = format!("[{}] {} {}\n", event.timestamp, event.level, event.message);

                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
                    let _ = file.write_all(line.as_bytes());
                }
            }
        });

        Self { sender: tx, config: config_arc }
    }

    pub fn write(&self, event: &LogEvent) {
        let _ = self.sender.try_send(event.clone());
    }

    pub fn update_settings(&self, settings: &CategorizedLauncherSettings) {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.enable_persistent = settings.logging.enable_persistent_logging;
            cfg.enable_compression = settings.logging.enable_log_compression;
            cfg.size_limit_mb = settings.logging.log_file_size_limit_mb;
            cfg.retention_days = settings.logging.log_retention_days;
        }
    }

    pub fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = self.config.lock().unwrap().clone();
        let cutoff = Utc::now() - chrono::Duration::days(cfg.retention_days as i64);

        Self::cleanup_dir(&cfg.logs_dir.join("launcher"), cutoff)?;
        Self::cleanup_dir(&cfg.logs_dir.join("installations"), cutoff)?;

        Ok(())
    }

    pub fn export(&self, instance_id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = self.config.lock().unwrap().clone();

        let export_dir = cfg.logs_dir.join("exports");
        fs::create_dir_all(&export_dir)?;

        let file_name = match instance_id {
            Some(id) => format!("logs_{}.txt", id),
            None => "logs_global.txt".to_string(),
        };

        let export_path = export_dir.join(file_name);

        fs::write(export_path, "export placeholder\n")?;

        Ok(())
    }

    fn rotate_if_needed(cfg: &FileConfig, path: &Path) {
        if let Ok(meta) = fs::metadata(path) {
            if meta.len() > cfg.size_limit_mb * 1024 * 1024 {
                let _ = Self::compress(cfg, path);
            }
        }
    }

    fn compress(cfg: &FileConfig, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !cfg.enable_compression {
            return Ok(());
        }

        let data = fs::read(path)?;
        let mut archive = File::create(path.with_extension("log.7z"))?;

        let mut encoder = sevenz_rust::SevenZWriter::new(&mut archive)?;
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        encoder.push_archive_entry(sevenz_rust::SevenZArchiveEntry::from_path(&name, name), Some(std::io::Cursor::new(data)))?;

        encoder.finish()?;

        let _ = fs::remove_file(path);
        Ok(())
    }

    fn cleanup_dir(dir: &Path, cutoff: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(meta) = fs::metadata(&path) {
                    if let Ok(created) = meta.created() {
                        let dt: DateTime<Utc> = created.into();
                        if dt < cutoff {
                            let _ = fs::remove_file(path);
                        }
                    }
                }
            } else if path.is_dir() {
                Self::cleanup_dir(&path, cutoff)?;
                let _ = fs::remove_dir(path);
            }
        }

        Ok(())
    }
}
