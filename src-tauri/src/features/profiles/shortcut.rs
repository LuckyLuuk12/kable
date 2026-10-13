use std::fs;
use std::path::{Path, PathBuf};

use api_types::profiles::KableProfile;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

/// Creates a desktop shortcut that launches Kable directly into a profile.
///
/// If the profile has valid Base64-encoded image data, the shortcut uses it
/// as its icon on platforms supported below. Missing or invalid icons do not
/// prevent shortcut creation.
///
/// Returns the path to the created shortcut.
pub fn create_desktop_shortcut(profile: &KableProfile) -> Result<String, String> {
    let desktop = dirs::desktop_dir().ok_or_else(|| "Failed to locate the desktop directory".to_string())?;

    fs::create_dir_all(&desktop).map_err(|e| format!("Failed to access desktop directory: {e}"))?;

    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get Kable executable path: {e}"))?;

    let name = sanitize_filename(&profile.metadata.name);
    let shortcut_path = desktop.join(format!("{name}{}", shortcut_extension()));

    // Icon processing is deliberately best-effort. Imported profiles can
    // contain malformed or unsupported image data.
    let icon_path = profile.metadata.icon.as_deref().and_then(|icon| create_profile_icon(icon, &profile.id));

    #[cfg(target_os = "windows")]
    {
        let mut shortcut = mslnk::ShellLink::new(exe_path.clone()).map_err(|e| format!("Failed to create shortcut: {e}"))?;

        shortcut.set_arguments(Some(format!("--launch-profile \"{}\"", profile.id)));
        shortcut.set_name(Some(profile.metadata.name.clone()));
        shortcut.set_working_dir(exe_path.parent().and_then(Path::to_str).map(str::to_owned));

        if let Some(icon_path) = &icon_path {
            shortcut.set_icon_location(Some(icon_path.to_string_lossy().into_owned()));
        }

        shortcut.create_lnk(&shortcut_path).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;

        // `open --args` forwards arguments to the application.
        // A .command script does not directly support assigning a custom
        // image as its Finder icon, so the validated icon is not applied here.
        let script = format!(
            "#!/bin/sh\nexec open \"{}\" --args --launch-profile \"{}\"\n",
            escape_shell_double_quoted(&exe_path.to_string_lossy()),
            escape_shell_double_quoted(&profile.id),
        );

        fs::write(&shortcut_path, script).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;

        let mut permissions = fs::metadata(&shortcut_path).map_err(|e| format!("Failed to read shortcut permissions: {e}"))?.permissions();
        permissions.set_mode(0o755);

        fs::set_permissions(&shortcut_path, permissions).map_err(|e| format!("Failed to make shortcut executable: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;

        let icon_entry = icon_path
            .as_ref()
            .map(|path| format!("Icon={}\n", escape_desktop_entry(path.to_string_lossy().as_ref())))
            .unwrap_or_default();

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Exec=\"{}\" --launch-profile \"{}\"\n\
             {}\
             Terminal=false\n\
             Categories=Game;\n",
            escape_desktop_entry(&profile.metadata.name),
            escape_desktop_entry(&exe_path.to_string_lossy()),
            escape_desktop_entry(&profile.id),
            icon_entry,
        );

        fs::write(&shortcut_path, desktop_entry).map_err(|e| format!("Failed to write desktop shortcut: {e}"))?;

        let mut permissions = fs::metadata(&shortcut_path).map_err(|e| format!("Failed to read shortcut permissions: {e}"))?.permissions();
        permissions.set_mode(0o755);

        fs::set_permissions(&shortcut_path, permissions).map_err(|e| format!("Failed to make desktop shortcut executable: {e}"))?;
    }

    Ok(shortcut_path.to_string_lossy().into_owned())
}

/// Decodes, validates, and saves a profile icon.
///
/// Supports raw Base64 image data and data URIs such as
/// `data:image/png;base64,...`. Returns None for malformed or unsupported
/// image data or when the icon cannot be written.
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn create_profile_icon(encoded: &str, profile_id: &str) -> Option<PathBuf> {
    use std::io::Cursor;

    const MAX_ENCODED_ICON_SIZE: usize = 10 * 1024 * 1024;
    const MAX_DIMENSION: u32 = 8192;
    const MAX_PIXELS: u64 = 16_777_216;

    let encoded = encoded.trim();

    if encoded.is_empty() || encoded.len() > MAX_ENCODED_ICON_SIZE {
        return None;
    }

    let payload = if encoded.starts_with("data:") {
        let (header, payload) = encoded.split_once(',')?;

        if !header.ends_with(";base64") {
            return None;
        }

        payload
    } else {
        encoded
    };

    let bytes = STANDARD.decode(payload).ok()?;

    if bytes.is_empty() || bytes.len() > MAX_ENCODED_ICON_SIZE {
        return None;
    }

    let image = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format().ok()?.decode().ok()?;

    let (width, height) = (image.width(), image.height());

    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return None;
    }

    let icon_dir = dirs::data_local_dir()?.join("Kable").join("profile-icons");
    fs::create_dir_all(&icon_dir).ok()?;

    let safe_id = sanitize_filename(profile_id);

    #[cfg(target_os = "windows")]
    {
        let path = icon_dir.join(format!("{safe_id}.ico"));
        let mut icon = ico::IconDir::new(ico::ResourceType::Icon);

        // Multiple sizes help Windows render the icon cleanly in different
        // desktop, Explorer, and shortcut views.
        for size in [16u32, 24, 32, 48, 64, 128, 256] {
            let resized = image::imageops::resize(&image, size, size, image::imageops::FilterType::Lanczos3);

            let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
            let entry = ico::IconDirEntry::encode(&icon_image).ok()?;
            icon.add_entry(entry);
        }

        let file = fs::File::create(&path).ok()?;
        icon.write(file).ok()?;

        Some(path)
    }

    #[cfg(target_os = "linux")]
    {
        let path = icon_dir.join(format!("{safe_id}.png"));
        image.save_with_format(&path, image::ImageFormat::Png).ok()?;
        Some(path)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn create_profile_icon(_encoded: &str, _profile_id: &str) -> Option<PathBuf> {
    None
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
fn escape_shell_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"").replace('$', "\\$").replace('`', "\\`")
}

#[cfg(target_os = "linux")]
fn escape_desktop_entry(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}
