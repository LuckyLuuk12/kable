use std::path::PathBuf;
use std::process::Command;

/// Attempts to find a working Java executable, either from the provided path or common install locations.
///
/// Used by all loader modules to locate Java for launching Minecraft.
///
/// ### Arguments
/// * `java_path` - Optional user-specified Java path.
///
/// ### Returns
/// Ok(path to Java executable) or Err if not found.
pub fn find_java_executable(java_path: Option<&String>) -> Result<String, String> {
    if let Some(path) = java_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            if PathBuf::from(trimmed).exists() {
                return Ok(trimmed.to_string());
            }
            crate::logging::Logger::warn_global(
                &format!(
                    "Specified Java path does not exist: '{}'. Attempting auto-detection.",
                    trimmed
                ),
                None,
            );
        }
    }

    // Platform-specific detection
    #[cfg(target_os = "windows")]
    {
        find_java_windows()
    }

    #[cfg(target_os = "macos")]
    {
        find_java_macos()
    }

    #[cfg(target_os = "linux")]
    {
        find_java_linux()
    }
}

#[cfg(target_os = "windows")]
fn find_java_windows() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    // Try PowerShell Get-Command first (checks PATH)
    if let Ok(output) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Command javaw.exe -ErrorAction SilentlyContinue).Source",
        ])
        .creation_flags(0x08000000)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(path) = stdout.lines().next() {
                let path = path.trim();
                // Skip Oracle javapath stub launcher (it's not a real installation)
                if !path.is_empty()
                    && PathBuf::from(path).exists()
                    && !path.contains("Common Files\\Oracle\\Java\\javapath")
                {
                    crate::logging::Logger::debug_global(
                        &format!("Found javaw.exe via PowerShell: {}", path),
                        None,
                    );
                    return Ok(path.to_string());
                } else if path.contains("javapath") {
                    crate::logging::Logger::debug_global(
                        "Skipping Oracle javapath stub, searching for real Java installation...",
                        None,
                    );
                }
            }
        }
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
        found_javas.sort_by(|a, b| b.1.cmp(&a.1)); // Descending order (highest version first)
        let chosen = &found_javas[0].0;
        crate::logging::Logger::debug_global(
            &format!(
                "Found {} Java installation(s), chose: {}",
                found_javas.len(),
                chosen.display()
            ),
            None,
        );
        return Ok(chosen.to_string_lossy().to_string());
    }

    Err("Java not found. Please install Java 17+ or specify the Java path in settings.".to_string())
}

#[cfg(target_os = "macos")]
fn find_java_macos() -> Result<String, String> {
    // Try /usr/libexec/java_home (macOS standard tool)
    if let Ok(output) = Command::new("/usr/libexec/java_home").output() {
        if output.status.success() {
            let java_home = String::from_utf8_lossy(&output.stdout);
            let java_path = PathBuf::from(java_home.trim()).join("bin").join("java");
            if java_path.exists() {
                crate::logging::Logger::debug_global(
                    &format!("Found java via java_home: {}", java_path.display()),
                    None,
                );
                return Ok(java_path.to_string_lossy().to_string());
            }
        }
    }

    // Try PATH
    if let Ok(output) = Command::new("which").arg("java").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.trim();
            if !path.is_empty() && PathBuf::from(path).exists() {
                crate::logging::Logger::debug_global(
                    &format!("Found java via which: {}", path),
                    None,
                );
                return Ok(path.to_string());
            }
        }
    }

    // Scan common macOS installation directories
    let java_root_dirs = vec![
        "/Library/Java/JavaVirtualMachines",
        "/System/Library/Java/JavaVirtualMachines",
    ];

    let mut found_javas: Vec<(PathBuf, u32)> = Vec::new();

    for root in java_root_dirs {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let java_path = path.join("Contents").join("Home").join("bin").join("java");
                    if java_path.exists() {
                        if let Ok(output) = Command::new(&java_path).arg("-version").output() {
                            if output.status.success() {
                                let version_info = String::from_utf8_lossy(&output.stderr);
                                let version = extract_java_version(&version_info);
                                found_javas.push((java_path, version));
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_javas.is_empty() {
        found_javas.sort_by(|a, b| b.1.cmp(&a.1));
        let chosen = &found_javas[0].0;
        crate::logging::Logger::debug_global(
            &format!(
                "Found {} Java installation(s), chose: {}",
                found_javas.len(),
                chosen.display()
            ),
            None,
        );
        return Ok(chosen.to_string_lossy().to_string());
    }

    Err("Java not found. Please install Java 17+ or specify the Java path in settings.".to_string())
}

#[cfg(target_os = "linux")]
fn find_java_linux() -> Result<String, String> {
    // Try PATH first
    if let Ok(output) = Command::new("which").arg("java").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.trim();
            if !path.is_empty() && PathBuf::from(path).exists() {
                crate::logging::Logger::debug_global(
                    &format!("Found java via which: {}", path),
                    None,
                );
                return Ok(path.to_string());
            }
        }
    }

    // Try update-alternatives (Debian/Ubuntu)
    if let Ok(output) = Command::new("update-alternatives")
        .args(&["--query", "java"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Value:") {
                    let path = line.trim_start_matches("Value:").trim();
                    if !path.is_empty() && PathBuf::from(path).exists() {
                        crate::logging::Logger::debug_global(
                            &format!("Found java via update-alternatives: {}", path),
                            None,
                        );
                        return Ok(path.to_string());
                    }
                }
            }
        }
    }

    // Scan common Linux installation directories
    let java_root_dirs = vec![
        "/usr/lib/jvm",
        "/usr/lib64/jvm",
        "/usr/java",
        "/opt/java",
        "/opt/jdk",
    ];

    let mut found_javas: Vec<(PathBuf, u32)> = Vec::new();

    for root in java_root_dirs {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let java_path = path.join("bin").join("java");
                    if java_path.exists() {
                        if let Ok(output) = Command::new(&java_path).arg("-version").output() {
                            if output.status.success() {
                                let version_info = String::from_utf8_lossy(&output.stderr);
                                let version = extract_java_version(&version_info);
                                found_javas.push((java_path, version));
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_javas.is_empty() {
        found_javas.sort_by(|a, b| b.1.cmp(&a.1));
        let chosen = &found_javas[0].0;
        crate::logging::Logger::debug_global(
            &format!(
                "Found {} Java installation(s), chose: {}",
                found_javas.len(),
                chosen.display()
            ),
            None,
        );
        return Ok(chosen.to_string_lossy().to_string());
    }

    Err("Java not found. Please install Java 17+ or specify the Java path in settings.".to_string())
}

/// Extract major Java version from version output string
fn extract_java_version(version_str: &str) -> u32 {
    use regex::Regex;
    let re = Regex::new(r#"version "(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:_(\d+))?"#).unwrap();
    if let Some(cap) = re.captures(version_str) {
        if let Some(major) = cap.get(1) {
            if let Ok(num) = major.as_str().parse::<u32>() {
                // Handle old Java versioning (1.8.x -> version 8)
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

/// Auto-detect the Java executable path without requiring user input.
/// Returns the path to the detected Java executable or an error if not found.
pub fn auto_detect_java() -> Result<String, String> {
    find_java_executable(None)
}
