use crate::Logger;
use api_types::profiles::KableProfile;
use serde_json::Value;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

pub type RuntimeId = uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ProcessEvent {
    GameLaunched(KableProfile),
    Stdout(String),
    Stderr(String),
    Exit(i32),

    Custom { key: &'static str, data: Value },
}

pub trait ProcessSink: Send + Sync + 'static {
    fn handle(&self, runtime_id: RuntimeId, event: ProcessEvent);
}

#[derive(Clone)]
pub struct ProcessPipeline {
    sinks: Arc<[Arc<dyn ProcessSink>]>,
}

impl ProcessPipeline {
    pub fn new() -> Self {
        Self { sinks: Arc::from([]) }
    }

    pub fn with_sink(self, sink: Arc<dyn ProcessSink>) -> Self {
        let mut vec = Vec::with_capacity(self.sinks.len() + 1);

        vec.extend(self.sinks.iter().cloned());
        vec.push(sink);

        Self { sinks: Arc::from(vec) }
    }

    pub fn emit(&self, runtime_id: RuntimeId, event: ProcessEvent) {
        for sink in self.sinks.iter() {
            sink.handle(runtime_id, event.clone());
        }
    }
}

impl Default for ProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProcessRuntime {
    pub id: RuntimeId,
    pub child: Child,
    pub profile: KableProfile,
}

impl ProcessRuntime {
    pub async fn run(self, pipeline: ProcessPipeline) -> Result<(), String> {
        let mut child = self.child;

        let stdout = child.stdout.take().ok_or("missing stdout")?;
        let stderr = child.stderr.take().ok_or("missing stderr")?;

        let id = self.id;

        tokio::spawn(Self::pipe_stdout(stdout, id, pipeline.clone()));
        tokio::spawn(Self::pipe_stderr(stderr, id, pipeline.clone()));
        tokio::spawn(Self::wait_exit(child, id, pipeline.clone()));

        Ok(())
    }

    async fn pipe_stdout(stdout: impl tokio::io::AsyncRead + Unpin, id: RuntimeId, pipeline: ProcessPipeline) {
        let mut lines = BufReader::new(stdout).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            pipeline.emit(id, ProcessEvent::Stdout(line));
        }
    }

    async fn pipe_stderr(stderr: impl tokio::io::AsyncRead + Unpin, id: RuntimeId, pipeline: ProcessPipeline) {
        let mut lines = BufReader::new(stderr).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            pipeline.emit(id, ProcessEvent::Stderr(line));
        }
    }

    async fn wait_exit(mut child: Child, id: RuntimeId, pipeline: ProcessPipeline) {
        let status = child.wait().await;

        let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);

        pipeline.emit(id, ProcessEvent::Exit(code));
    }
}

pub struct TauriSink;

impl ProcessSink for TauriSink {
    fn handle(&self, runtime_id: RuntimeId, event: ProcessEvent) {
        let Ok(app) = crate::app_handle() else {
            return;
        };

        match event {
            ProcessEvent::GameLaunched(profile) => {
                let _ = app.emit(
                    "game-launched",
                    serde_json::json!({
                        "runtimeId": runtime_id,
                        "profile": profile
                    }),
                );
            }

            ProcessEvent::Stdout(line) => {
                let _ = app.emit(
                    "game-process-event",
                    serde_json::json!({
                        "runtimeId": runtime_id,
                        "type": "output",
                        "data": line
                    }),
                );

                Logger::info(&line, Some(&runtime_id.to_string()));
            }

            ProcessEvent::Stderr(line) => {
                let _ = app.emit(
                    "game-process-event",
                    serde_json::json!({
                        "runtimeId": runtime_id,
                        "type": "error",
                        "data": line
                    }),
                );

                Logger::error(&line, Some(&runtime_id.to_string()));
            }

            ProcessEvent::Exit(code) => {
                let _ = app.emit(
                    "game-process-event",
                    serde_json::json!({
                        "runtimeId": runtime_id,
                        "type": "exit",
                        "data": { "exitCode": code }
                    }),
                );
            }

            ProcessEvent::Custom { key, data } => {
                let _ = app.emit(
                    "game-process-event",
                    serde_json::json!({
                        "runtimeId": runtime_id,
                        "type": key,
                        "data": data
                    }),
                );
            }
        }
    }
}

pub fn handle_injections(profile: KableProfile, child: Child) -> (RuntimeId, ProcessRuntime, ProcessPipeline) {
    let runtime_id = uuid::Uuid::new_v4();

    let pipeline = ProcessPipeline::new().with_sink(Arc::new(TauriSink));

    let runtime = ProcessRuntime { id: runtime_id, child, profile };

    (runtime_id, runtime, pipeline)
}
