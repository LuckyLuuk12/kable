use crate::logging::{LogLevel, Logger};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowState {
    /// Window width (inner size on Windows to avoid title bar issues)
    pub width: u32,
    /// Window height (inner size on Windows to avoid title bar issues)  
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
    pub fullscreen: bool,
    pub monitor_name: Option<String>,
    pub monitor_position: Option<(i32, i32)>, // Monitor's top-left position
    pub monitor_size: Option<(u32, u32)>,     // Monitor's size
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1080,
            height: 720,
            x: -1, // -1 means center
            y: -1, // -1 means center
            maximized: false,
            fullscreen: false,
            monitor_name: None,
            monitor_position: None,
            monitor_size: None,
        }
    }
}

fn get_window_state_path() -> Result<PathBuf, String> {
    let launcher_dir = crate::get_kable_launcher_dir().map_err(|e| e.to_string())?;
    Ok(launcher_dir.join("window_state.json"))
}

#[tauri::command]
pub async fn load_window_state() -> Result<WindowState, String> {
    let state_path = get_window_state_path()?;

    if !state_path.exists() {
        return Ok(WindowState::default());
    }

    let contents = fs::read_to_string(state_path).map_err(|e| e.to_string())?;
    let state: WindowState = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    Ok(state)
}

#[tauri::command]
pub async fn save_window_state(state: WindowState) -> Result<(), String> {
    let state_path = get_window_state_path()?;

    // Ensure the parent directory exists
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(state_path, contents).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_current_window_state(window: WebviewWindow) -> Result<WindowState, String> {
    // On Windows, use inner_size to avoid title bar issues
    let size = if cfg!(target_os = "windows") {
        window.inner_size().map_err(|e| e.to_string())?
    } else {
        window.outer_size().map_err(|e| e.to_string())?
    };

    let position = window.outer_position().map_err(|e| e.to_string())?;
    let maximized = window.is_maximized().map_err(|e| e.to_string())?;
    let fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;

    // Get current monitor information
    let current_monitor = window.current_monitor().map_err(|e| e.to_string())?;
    let (monitor_name, monitor_position, monitor_size) = if let Some(monitor) = current_monitor {
        let name = monitor
            .name()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let monitor_pos = monitor.position();
        let monitor_sz = monitor.size();
        (
            Some(name),
            Some((monitor_pos.x, monitor_pos.y)),
            Some((monitor_sz.width, monitor_sz.height)),
        )
    } else {
        (None, None, None)
    };

    Ok(WindowState {
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        maximized,
        fullscreen,
        monitor_name,
        monitor_position,
        monitor_size,
    })
}

#[tauri::command]
pub async fn apply_window_state(window: WebviewWindow, state: WindowState) -> Result<(), String> {
    // Try to find the monitor that the window was on
    let target_monitor = if let (Some(saved_name), Some(saved_pos), Some(saved_size)) = (
        &state.monitor_name,
        state.monitor_position,
        state.monitor_size,
    ) {
        // Get all available monitors
        let monitors = window.available_monitors().map_err(|e| e.to_string())?;

        // First, try to find by exact name match
        let mut found_monitor = monitors
            .iter()
            .find(|monitor| monitor.name().map(|s| s.as_str()).unwrap_or("") == saved_name);

        // If no exact name match, try to find by position and size
        if found_monitor.is_none() {
            found_monitor = monitors.iter().find(|monitor| {
                let pos = monitor.position();
                let size = monitor.size();
                (pos.x, pos.y) == saved_pos && (size.width, size.height) == saved_size
            });
        }

        // If still no match, try to find by size only (in case monitor was moved)
        if found_monitor.is_none() {
            found_monitor = monitors.iter().find(|monitor| {
                let size = monitor.size();
                (size.width, size.height) == saved_size
            });
        }

        found_monitor.cloned()
    } else {
        None
    };

    // Set size first - use appropriate method for Windows
    let size = PhysicalSize::new(state.width, state.height);

    // On Windows, we might need to account for decorations differently
    if cfg!(target_os = "windows") {
        // For Windows, we're using inner_size when saving state, so set_size should work correctly
        window.set_size(size).map_err(|e| e.to_string())?;
    } else {
        window.set_size(size).map_err(|e| e.to_string())?;
    }

    // Handle position based on whether we found the target monitor
    if let Some(monitor) = target_monitor {
        // Monitor found - adjust position relative to the found monitor
        if state.x >= 0 && state.y >= 0 {
            // Calculate the relative position within the original monitor
            if let (Some(saved_pos), Some(_saved_size)) =
                (state.monitor_position, state.monitor_size)
            {
                let rel_x = state.x - saved_pos.0;
                let rel_y = state.y - saved_pos.1;

                // Apply the relative position to the current monitor
                let monitor_pos = monitor.position();
                let new_x = monitor_pos.x + rel_x;
                let new_y = monitor_pos.y + rel_y;

                // Ensure the window doesn't go off-screen
                let monitor_size = monitor.size();
                let clamped_x = new_x
                    .max(monitor_pos.x)
                    .min(monitor_pos.x + monitor_size.width as i32 - state.width as i32);
                let clamped_y = new_y
                    .max(monitor_pos.y)
                    .min(monitor_pos.y + monitor_size.height as i32 - state.height as i32);

                let position = PhysicalPosition::new(clamped_x, clamped_y);
                window.set_position(position).map_err(|e| e.to_string())?;
            } else {
                // Fallback: use absolute position but clamp to monitor bounds
                let monitor_pos = monitor.position();
                let monitor_size = monitor.size();
                let clamped_x = state
                    .x
                    .max(monitor_pos.x)
                    .min(monitor_pos.x + monitor_size.width as i32 - state.width as i32);
                let clamped_y = state
                    .y
                    .max(monitor_pos.y)
                    .min(monitor_pos.y + monitor_size.height as i32 - state.height as i32);

                let position = PhysicalPosition::new(clamped_x, clamped_y);
                window.set_position(position).map_err(|e| e.to_string())?;
            }
        } else {
            // Center on the found monitor
            let monitor_pos = monitor.position();
            let monitor_size = monitor.size();
            let center_x = monitor_pos.x + (monitor_size.width as i32 - state.width as i32) / 2;
            let center_y = monitor_pos.y + (monitor_size.height as i32 - state.height as i32) / 2;

            let position = PhysicalPosition::new(center_x, center_y);
            window.set_position(position).map_err(|e| e.to_string())?;
        }
    } else {
        // Monitor not found - fallback to primary monitor behavior
        if state.x >= 0 && state.y >= 0 {
            // Try to use the saved position, but it might end up on the primary monitor
            let position = PhysicalPosition::new(state.x, state.y);
            window.set_position(position).map_err(|e| e.to_string())?;
        } else {
            // Center on primary monitor
            window.center().map_err(|e| e.to_string())?;
        }
    }

    // Set maximized/fullscreen state
    if state.fullscreen {
        window.set_fullscreen(true).map_err(|e| e.to_string())?;
    } else if state.maximized {
        window.maximize().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_monitor_info(window: WebviewWindow) -> Result<Vec<serde_json::Value>, String> {
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;

    let monitor_info: Vec<serde_json::Value> = monitors
        .iter()
        .map(|monitor| {
            let name = monitor
                .name()
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            let position = monitor.position();
            let size = monitor.size();
            let scale_factor = monitor.scale_factor();

            serde_json::json!({
                "name": name,
                "position": [position.x, position.y],
                "size": [size.width, size.height],
                "scale_factor": scale_factor
            })
        })
        .collect();

    Ok(monitor_info)
}

#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
        Logger::console_log(
            LogLevel::Info,
            "Main window shown after initialization",
            None,
        );
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

pub fn setup_window_state_handlers(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Clone window for the closures
    let save_window = window.clone();

    // Save window state on resize
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) => {
            let window = save_window.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(state) = get_current_window_state(window).await {
                    let _ = save_window_state(state).await;
                }
            });
        }
        tauri::WindowEvent::CloseRequested { .. } => {
            let window = save_window.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(state) = get_current_window_state(window).await {
                    let _ = save_window_state(state).await;
                }
            });
        }
        _ => {}
    });

    Ok(())
}
