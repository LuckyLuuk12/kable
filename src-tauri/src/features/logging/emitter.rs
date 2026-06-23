use crate::features::logging::process::LogEvent;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Frontend-only emission pipeline:
/// batching, rate limiting, dedupe, IPC emit
pub struct LogEmitter {
    sender: SyncSender<LogEvent>,
}

struct BatchState {
    batch: Vec<Value>,
    recent_hashes: VecDeque<u64>,
    logs_this_second: usize,
    last_reset: Instant,
    last_emit: Instant,
}

impl LogEmitter {
    pub fn new() -> Self {
        let (tx, rx) = sync_channel::<LogEvent>(2048);
        let app_clone = crate::app_handle();

        std::thread::spawn(move || {
            let mut state = BatchState {
                batch: Vec::with_capacity(400),
                recent_hashes: VecDeque::with_capacity(50),
                logs_this_second: 0,
                last_reset: Instant::now(),
                last_emit: Instant::now(),
            };

            let batch_interval = Duration::from_millis(1000);
            let max_per_second = 800usize;
            let dedupe_window = 50usize;

            let hash = |s: &str| -> u64 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut h = DefaultHasher::new();
                s.hash(&mut h);
                h.finish()
            };

            loop {
                match rx.recv_timeout(batch_interval) {
                    Ok(event) => {
                        if state.last_reset.elapsed() >= Duration::from_secs(1) {
                            state.logs_this_second = 0;
                            state.last_reset = Instant::now();
                        }

                        if state.logs_this_second >= max_per_second {
                            continue;
                        }

                        let msg_hash = hash(&event.message);

                        if state.recent_hashes.contains(&msg_hash) {
                            continue;
                        }

                        state.recent_hashes.push_back(msg_hash);
                        if state.recent_hashes.len() > dedupe_window {
                            state.recent_hashes.pop_front();
                        }

                        state.logs_this_second += 1;

                        state.batch.push(event.to_frontend());

                        if state.batch.len() >= 400 {
                            let payload = serde_json::json!({
                                "logs": state.batch.clone()
                            });

                            let _ = app_clone.emit_to("main", "launcher-log-batch", payload);
                            state.batch.clear();
                            state.last_emit = Instant::now();
                        }
                    }

                    Err(RecvTimeoutError::Timeout) => {
                        if !state.batch.is_empty() && state.last_emit.elapsed() >= batch_interval {
                            let payload = serde_json::json!({
                                "logs": state.batch.clone()
                            });

                            let _ = app_clone.emit_to("main", "launcher-log-batch", payload);
                            state.batch.clear();
                            state.last_emit = Instant::now();
                        }
                    }

                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Self { sender: tx }
    }

    pub fn send(&self, event: &LogEvent) {
        let _ = self.sender.try_send(event.clone());
    }

    /// placeholder for future dynamic config support
    pub fn update_settings(&self, _settings: &api_types::settings::CategorizedLauncherSettings) {
        // intentionally empty for now (kept for manager compatibility)
    }
}
