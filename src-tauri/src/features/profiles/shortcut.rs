use std::fs;
use std::path::Path;

use api_types::profiles::KableProfile;

/// Creates a desktop shortcut that launches Kable directly into a profile.
///
/// Returns the path to the created shortcut.
pub fn create_desktop_shortcut(profile: &KableProfile) -> Result<String, String> {
    let desktop = dirs::desktop_dir().ok_or_else(|| "Failed to locate the desktop directory".to_string())?;

    fs::create_dir_all(&desktop).map_err(|e| format!("Failed to access desktop directory: {e}"))?;

    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get Kable executable path: {e}"))?;

    let name = sanitize_filename(&profile.metadata.name);
    let shortcut_path = desktop.join(format!("{name}{}", shortcut_extension()));

    #[cfg(target_os = "windows")]
    {
        let mut shortcut = mslnk::ShellLink::new(exe_path.clone()).map_err(|e| format!("Failed to create shortcut: {e}"))?;

        shortcut.set_arguments(Some(format!("--launch-profile \"{}\"", profile.id)));
        shortcut.set_name(Some(profile.metadata.name.clone()));
        shortcut.set_working_dir(exe_path.parent().and_then(Path::to_str).map(str::to_owned));

        shortcut.create_lnk(&shortcut_path).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;

        // `open --args` forwards the arguments to the application.
        let script = format!(
            "#!/bin/sh\nexec open \"{}\" --args --launch-profile \"{}\"\n",
            escape_shell_argument(&exe_path.to_string_lossy()),
            escape_shell_argument(&profile.id),
        );

        fs::write(&shortcut_path, script).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;

        let mut permissions = fs::metadata(&shortcut_path).map_err(|e| format!("Failed to read shortcut permissions: {e}"))?.permissions();
        permissions.set_mode(0o755);

        fs::set_permissions(&shortcut_path, permissions).map_err(|e| format!("Failed to make shortcut executable: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Exec=\"{}\" --launch-profile \"{}\"\n\
             Terminal=false\n\
             Categories=Game;\n",
            escape_desktop_entry(&profile.metadata.name),
            escape_desktop_entry(&exe_path.to_string_lossy()),
            escape_desktop_entry(&profile.id),
        );

        fs::write(&shortcut_path, desktop_entry).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;

        let mut permissions = fs::metadata(&shortcut_path).map_err(|e| format!("Failed to read shortcut permissions: {e}"))?.permissions();
        permissions.set_mode(0o755);

        fs::set_permissions(&shortcut_path, permissions).map_err(|e| format!("Failed to make shortcut executable: {e}"))?;
    }

    Ok(shortcut_path.to_string_lossy().into_owned())
}

#[cfg(target_os = "windows")]
fn shortcut_extension() -> &'static str {
    ".lnk"
}

#[cfg(target_os = "macos")]
fn shortcut_extension() -> &'static str {
    ".command"
}

#[cfg(target_os = "linux")]
fn shortcut_extension() -> &'static str {
    ".desktop"
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '_' } else { c })
        .collect();

    let sanitized = sanitized.trim().trim_matches('.');

    if sanitized.is_empty() {
        "Kable Profile".to_string()
    } else {
        sanitized.to_string()
    }
}

#[cfg(target_os = "macos")]
fn escape_shell_argument(value: &str) -> String {
    value.replace('\'', "'\\''")
}

#[cfg(target_os = "linux")]
fn escape_desktop_entry(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(' ', "\\ ")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
}
