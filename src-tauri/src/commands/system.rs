use crate::LogLevel;
use crate::Logger;
use tauri::command;

#[command]
pub async fn open_url(url: String) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("🌐 Opening URL: {}", url), None);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL on Windows: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL on macOS: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL on Linux: {}", e))?;
    }

    Logger::console_log(LogLevel::Info, "✅ URL opened successfully", None);

    Ok(())
}

#[command]
pub async fn open_path(path: String) -> Result<(), String> {
    Logger::console_log(LogLevel::Info, &format!("📁 Opening path: {}", path), None);

    #[cfg(target_os = "windows")]
    {
        // Use start with empty title to allow paths with spaces
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open path on Windows: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open path on macOS: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open path on Linux: {}", e))?;
    }

    Logger::console_log(LogLevel::Info, "✅ Path opened successfully", None);

    Ok(())
}
