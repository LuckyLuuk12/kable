/// ! I have no clue how to do this on Linux and MacOS (and Windows actually as well but it works...), so if you know how to do this, please contribute!
use crate::features::logging::Logger;
use std::path::PathBuf;
use std::process::Command;

/// Attempts to find a working Java executable, either from the provided path or common install locations.
pub fn find_java_executable(java_path: Option<&String>) -> Result<String, String> {
    if let Some(path) = java_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            if std::path::Path::new(trimmed).exists() {
                return Ok(trimmed.to_string());
            }
            Logger::warn_global(&format!("Specified Java path does not exist: '{}'. Attempting auto-detection.", trimmed), None);
        }
    }

    // Platform-specific detection
    #[cfg(target_os = "windows")]
    {
        #[allow(clippy::needless_return)]
        return find_java_windows();
    }

    #[cfg(target_os = "macos")]
    {
        #[allow(clippy::needless_return)]
        return find_java_macos();
    }

    #[cfg(target_os = "linux")]
    {
        #[allow(clippy::needless_return)]
        return find_java_linux();
    }
}

#[cfg(target_os = "windows")]
fn find_java_windows() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    // Try PowerShell Get-Command first (checks PATH)
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-Command javaw.exe -ErrorAction SilentlyContinue).Source"])
        .creation_flags(0x08000000)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(path) = stdout.lines().next() {
                let path = path.trim();
                // Skip Oracle javapath stub launcher
                if !path.is_empty() && std::path::Path::new(path).exists() && !path.contains("Common Files\\Oracle\\Java\\javapath") {
                    Logger::debug_global(&format!("Found javaw.exe via PowerShell: {}", path), None);
                    return Ok(path.to_string());
                }
            }
        }
    }

    // Try Minecraft Launcher's bundled runtimes
    if let Some(path) = find_minecraft_launcher_java_windows() {
        Logger::debug_global(&format!("Found javaw.exe in Minecraft Launcher runtime: {}", path.display()), None);
        return Ok(path.to_string_lossy().to_string());
    }

    // Scan common Windows installation directories
    let java_root_dirs = vec![
        "C:\\Program Files\\Java",
        "C:\\Program Files\\Eclipse Adoptium",
        "C:\\Program Files\\Microsoft",
        "C:\\Program Files\\Zulu",
        "C:\\Program Files (x86)\\Java",
    ];

    let mut found_javas: Vec<(PathBuf, u32)> = Vec::new();

    for root in java_root_dirs {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let javaw_path = path.join("bin").join("javaw.exe");
                    if javaw_path.exists() {
                        if let Ok(output) = Command::new(&javaw_path).arg("-version").output() {
                            if output.status.success() {
                                let version_info = String::from_utf8_lossy(&output.stderr);
                                let version = extract_java_version(&version_info);
                                found_javas.push((javaw_path, version));
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_javas.is_empty() {
        found_javas.sort_by_key(|b| std::cmp::Reverse(b.1));
        let chosen = &found_javas[0].0;
        return Ok(chosen.to_string_lossy().to_string());
    }

    Err("Java not found. Please install Java 17+ or specify the Java path in settings.".to_string())
}

#[cfg(target_os = "windows")]
fn find_minecraft_launcher_java_windows() -> Option<PathBuf> {
    let runtime_roots = ["C:\\Program Files (x86)\\Minecraft Launcher\\runtime", "C:\\Program Files\\Minecraft Launcher\\runtime"];

    let mut found_javas: Vec<(PathBuf, u32)> = Vec::new();

    for runtime_root in runtime_roots {
        if let Ok(runtime_variants) = std::fs::read_dir(runtime_root) {
            for runtime_variant in runtime_variants.flatten() {
                let runtime_variant_path = runtime_variant.path();
                if !runtime_variant_path.is_dir() {
                    continue;
                }

                if let Ok(platform_dirs) = std::fs::read_dir(&runtime_variant_path) {
                    for platform_dir in platform_dirs.flatten() {
                        let platform_path = platform_dir.path();
                        if !platform_path.is_dir() {
                            continue;
                        }

                        if let Ok(runtime_dirs) = std::fs::read_dir(&platform_path) {
                            for runtime_dir in runtime_dirs.flatten() {
                                let javaw_path = runtime_dir.path().join("bin").join("javaw.exe");
                                if let Some(version) = probe_windows_java_version(&javaw_path) {
                                    found_javas.push((javaw_path, version));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if found_javas.is_empty() {
        None
    } else {
        found_javas.sort_by_key(|b| std::cmp::Reverse(b.1));
        Some(found_javas[0].0.clone())
    }
}

#[cfg(target_os = "windows")]
fn probe_windows_java_version(javaw_path: &std::path::Path) -> Option<u32> {
    if !javaw_path.exists() {
        return None;
    }

    if let Ok(output) = Command::new(javaw_path).arg("-version").output() {
        if output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version_info = if stderr.trim().is_empty() { stdout } else { stderr };
            return Some(extract_java_version(&version_info));
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn find_java_macos() -> Result<String, String> {
    if let Ok(output) = Command::new("/usr/libexec/java_home").output() {
        if output.status.success() {
            let java_home = String::from_utf8_lossy(&output.stdout);
            let java_path = std::path::Path::new(java_home.trim()).join("bin").join("java");
            if java_path.exists() {
                return Ok(java_path.to_string_lossy().to_string());
            }
        }
    }

    Err("Java not found on macOS".to_string())
}

#[cfg(target_os = "linux")]
fn find_java_linux() -> Result<String, String> {
    if let Ok(output) = Command::new("which").arg("java").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.trim();
            if !path.is_empty() && std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
    }

    Err("Java not found on Linux".to_string())
}

fn extract_java_version(version_str: &str) -> u32 {
    use regex::Regex;
    let re = Regex::new(r#"version "(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:_(\d+))?"#).unwrap();
    if let Some(cap) = re.captures(version_str) {
        if let Some(major) = cap.get(1) {
            if let Ok(num) = major.as_str().parse::<u32>() {
                if num == 1 {
                    if let Some(minor) = cap.get(2) {
                        return minor.as_str().parse().unwrap_or(0);
                    }
                }
                return num;
            }
        }
    }
    0
}

pub fn auto_detect_java() -> Result<String, String> {
    find_java_executable(None)
}
