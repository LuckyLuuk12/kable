use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
    pub fullscreen: bool,
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
        }
    }
}

fn get_window_state_path() -> Result<PathBuf, String> {
    let launcher_dir = crate::settings::get_launcher_data_dir().map_err(|e| e.to_string())?;
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
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let maximized = window.is_maximized().map_err(|e| e.to_string())?;
    let fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
    
    Ok(WindowState {
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        maximized,
        fullscreen,
    })
}

#[tauri::command]
pub async fn apply_window_state(window: WebviewWindow, state: WindowState) -> Result<(), String> {
    // Set size first
    let size = PhysicalSize::new(state.width, state.height);
    window.set_size(size).map_err(|e| e.to_string())?;
    
    // Set position if not centering (-1, -1)
    if state.x >= 0 && state.y >= 0 {
        let position = PhysicalPosition::new(state.x, state.y);
        window.set_position(position).map_err(|e| e.to_string())?;
    } else {
        // Center the window
        window.center().map_err(|e| e.to_string())?;
    }
    
    // Set maximized/fullscreen state
    if state.fullscreen {
        window.set_fullscreen(true).map_err(|e| e.to_string())?;
    } else if state.maximized {
        window.maximize().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

pub fn setup_window_state_handlers(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").ok_or("Main window not found")?;
    
    // Clone window for the closures
    let save_window = window.clone();
    
    // Save window state on resize
    window.on_window_event(move |event| {
        match event {
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
        }
    });
    
    Ok(())
}
