use tauri::command;
use tokio::fs as async_fs;
use base64::Engine;

/// Resolve an image key to a filesystem path (user-provided) or fallback static path
#[command]
pub async fn resolve_image_path(key: String) -> Result<String, String> {
    // Allowed extensions to try in order
    let exts = ["png", "jpg", "jpeg", "webp", "svg", "gif", "ico"];

    // Try launcher config images directory: <kable_launcher_dir>/config/images/
    let launcher_dir = crate::get_kable_launcher_dir()?;
    let images_dir = launcher_dir.join("config").join("images");

    // Ensure the images directory exists so callers and the UI can save files there later
    if !images_dir.exists() {
        async_fs::create_dir_all(&images_dir).await.map_err(|e| format!("Failed to create images directory: {}", e))?;
    }

    if images_dir.exists() {
        // If key already points to a filename with extension, check directly
        let candidate = images_dir.join(&key);
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().to_string());
        }

        // More robust: scan the images directory for any file whose stem matches the key
        // (case-insensitive). Collect matches and prefer extensions according to `exts` order.
        let mut matches: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(mut rd) = async_fs::read_dir(&images_dir).await {
            while let Ok(Some(entry)) = rd.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem.eq_ignore_ascii_case(&key) {
                            matches.push(path);
                        }
                    }
                }
            }
        }

        if !matches.is_empty() {
            // prefer by extension order specified in `exts`
            for ext in &exts {
                for p in &matches {
                    if let Some(p_ext) = p.extension().and_then(|e| e.to_str()) {
                        if p_ext.eq_ignore_ascii_case(ext) {
                            // return as data URL (base64) to avoid file:// restrictions in the webview
                            if let Ok(bytes) = async_fs::read(p).await {
                                let mime = match p_ext.to_ascii_lowercase().as_str() {
                                    "png" => "image/png",
                                    "jpg" | "jpeg" => "image/jpeg",
                                    "webp" => "image/webp",
                                    "svg" => "image/svg+xml",
                                    "gif" => "image/gif",
                                    "ico" => "image/x-icon",
                                    _ => "application/octet-stream",
                                };
                                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                return Ok(format!("data:{};base64,{}", mime, b64));
                            } else {
                                // if reading failed, fall back to returning path to try other candidates
                                return Ok(p.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            // if none matched preferred exts, return the first match (try to read as data URL)
            if let Some(p) = matches.first() {
                if let Some(p_ext) = p.extension().and_then(|e| e.to_str()) {
                    if let Ok(bytes) = async_fs::read(p).await {
                        let mime = match p_ext.to_ascii_lowercase().as_str() {
                            "png" => "image/png",
                            "jpg" | "jpeg" => "image/jpeg",
                            "webp" => "image/webp",
                            "svg" => "image/svg+xml",
                            "gif" => "image/gif",
                            "ico" => "image/x-icon",
                            _ => "application/octet-stream",
                        };
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        return Ok(format!("data:{};base64,{}", mime, b64));
                    } else {
                        return Ok(p.to_string_lossy().to_string());
                    }
                } else {
                    return Ok(p.to_string_lossy().to_string());
                }
            }
        }

        // Otherwise try common extensions (keeps previous behavior as a fallback)
        for ext in &exts {
            let file = images_dir.join(format!("{}.{}", key, ext));
            if file.exists() {
                return Ok(file.to_string_lossy().to_string());
            }
        }
    }

    // Fall back to app static assets under /img/<key>.<ext>
    // Prefer webp then png for better compression / modern assets
    Ok(format!("/img/{}.webp", key))
}
