/// Detailed grouping for mod/resourcepack/shaderpack files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileDetailedGroup {
    pub disabled: Vec<PackFileInfo>,
    pub optional: Vec<PackFileInfo>,
    pub to_be_installed: Vec<PackFileInfo>,
}

/// Full manifest grouping for frontend use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrPackDetailed {
    pub mods: PackFileDetailedGroup,
    pub resourcepacks: PackFileDetailedGroup,
    pub shaderpacks: PackFileDetailedGroup,
}

/// Classify files into disabled, optional, to_be_installed for each type, using installation info for install state
pub fn list_pack_files_detailed(
    index: &MrpackIndex,
    profile_dir: Option<&std::path::Path>,
) -> MrPackDetailed {
    use std::path::Path;

    // Helper to check if a file is already installed and/or would overwrite
    fn get_install_state(
        file: &MrpackFile,
        subfolder: &str,
        profile_dir: Option<&Path>,
    ) -> (bool, bool) {
        if let Some(profile_dir) = profile_dir {
            let rel_path = Path::new(&file.path)
                .strip_prefix(subfolder)
                .unwrap_or_else(|_| Path::new(&file.path));
            let dest = profile_dir.join(subfolder).join(rel_path);
            if dest.exists() {
                // Compare hash if possible
                if let Ok(mut f) = std::fs::File::open(&dest) {
                    let mut buf = Vec::new();
                    use std::io::Read;
                    if f.read_to_end(&mut buf).is_ok() && verify_bytes(&buf, &file.hashes).is_ok() {
                        // Already present and matches
                        return (true, false);
                    }
                }
                // Exists but hash mismatch
                return (true, true);
            }
        }
        (false, false)
    }

    fn classify(
        files: &[MrpackFile],
        kind: &str,
        profile_dir: Option<&Path>,
    ) -> PackFileDetailedGroup {
        let mut disabled = Vec::new();
        let mut optional = Vec::new();
        let mut to_be_installed = Vec::new();
        for f in files {
            let (already_installed, overwrite) = get_install_state(f, kind, profile_dir);
            let info = PackFileInfo {
                path: f.path.clone(),
                file_size: f.file_size,
                hashes: f.hashes.clone(),
                downloads: f.downloads.clone(),
                env: f.env.clone(),
                already_installed,
                overwrite,
            };
            // Disabled: mods with .disabled in path, or env.client/server == "unsupported"
            let is_disabled = f.path.contains(".disabled")
                || f.env.as_ref().is_some_and(|env| {
                    env.client.as_deref() == Some("unsupported")
                        || env.server.as_deref() == Some("unsupported")
                });
            // Optional: mods with env.client/server == "optional"
            let is_optional = f.env.as_ref().is_some_and(|env| {
                env.client.as_deref() == Some("optional")
                    || env.server.as_deref() == Some("optional")
            });
            if is_disabled {
                disabled.push(info);
            } else if is_optional {
                optional.push(info);
            } else {
                to_be_installed.push(info);
            }
        }
        PackFileDetailedGroup {
            disabled,
            optional,
            to_be_installed,
        }
    }

    let mut mods = Vec::new();
    let mut resourcepacks = Vec::new();
    let mut shaderpacks = Vec::new();
    for f in &index.files {
        if f.path.starts_with("mods/") {
            mods.push(f.clone());
        } else if f.path.starts_with("resourcepacks/") {
            resourcepacks.push(f.clone());
        } else if f.path.starts_with("shaderpacks/") {
            shaderpacks.push(f.clone());
        }
    }
    MrPackDetailed {
        mods: classify(&mods, "mods", profile_dir),
        resourcepacks: classify(&resourcepacks, "resourcepacks", profile_dir),
        shaderpacks: classify(&shaderpacks, "shaderpacks", profile_dir),
    }
}
/// Generate a unique modpack_id from name, version_id, and format_version
pub fn generate_modpack_id(index: &MrpackIndex) -> String {
    // Use a simple format: name-versionId-formatVersion, sanitized for filesystem
    let name = index
        .name
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "_");
    let version = index
        .version_id
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "_");
    format!("{}-{}-v{}", name, version, index.format_version)
}

/// List all files in the pack, grouped by type (mod/resourcepack/shaderpack/other), with env info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileInfo {
    pub path: String,
    pub file_size: u64,
    pub hashes: std::collections::HashMap<String, String>,
    pub downloads: Vec<String>,
    pub env: Option<MrpackEnv>,
    /// true if this file is already installed in the target profile
    pub already_installed: bool,
    /// true if this file would overwrite an existing file (hash mismatch)
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileGroups {
    pub mods: Vec<PackFileInfo>,
    pub resourcepacks: Vec<PackFileInfo>,
    pub shaderpacks: Vec<PackFileInfo>,
    pub others: Vec<PackFileInfo>,
}

/// Classify files by folder (mods/, resourcepacks/, shaderpacks/, else others)
pub fn list_pack_files(index: &MrpackIndex) -> PackFileGroups {
    let mut mods = Vec::new();
    let mut resourcepacks = Vec::new();
    let mut shaderpacks = Vec::new();
    let mut others = Vec::new();
    for f in &index.files {
        let info = PackFileInfo {
            path: f.path.clone(),
            file_size: f.file_size,
            hashes: f.hashes.clone(),
            downloads: f.downloads.clone(),
            env: f.env.clone(),
            already_installed: false,
            overwrite: false,
        };
        if f.path.starts_with("mods/") {
            mods.push(info);
        } else if f.path.starts_with("resourcepacks/") {
            resourcepacks.push(info);
        } else if f.path.starts_with("shaderpacks/") {
            shaderpacks.push(info);
        } else {
            others.push(info);
        }
    }
    PackFileGroups {
        mods,
        resourcepacks,
        shaderpacks,
        others,
    }
}

/// Diff pack files against a profile's folders, returning conflicts and new files
pub fn diff_pack_files(
    pack_files: &[PackFileInfo],
    profile_dir: &Path,
    subfolder: &str,
) -> anyhow::Result<(Vec<PackFileInfo>, Vec<PackFileInfo>)> {
    // Returns (conflicts, new_files)
    let mut conflicts = Vec::new();
    let mut new_files = Vec::new();
    let target_dir = profile_dir.join(subfolder);
    for file in pack_files {
        let rel_path = Path::new(&file.path)
            .strip_prefix(subfolder)
            .unwrap_or_else(|_| Path::new(&file.path));
        let dest = target_dir.join(rel_path);
        if dest.exists() {
            // Compare hash if possible
            if let Ok(mut f) = File::open(&dest) {
                let mut buf = Vec::new();
                if f.read_to_end(&mut buf).is_ok() && verify_bytes(&buf, &file.hashes).is_ok() {
                    // Already present and matches
                    continue;
                }
            }
            conflicts.push(file.clone());
        } else {
            new_files.push(file.clone());
        }
    }
    Ok((conflicts, new_files))
}

/// Copy selected files from extracted mrpack (in standardized temp dir) to profile, skipping or overwriting as needed.
/// This should only be called after user selection, using the temp dir from get_temp_dir(instance_id, modpack_id).
pub fn copy_selected_pack_files(
    extracted_dir: &Path,
    profile_dir: &Path,
    files: &[PackFileInfo],
    subfolder: &str,
    overwrite: bool,
) -> anyhow::Result<()> {
    let src_base = extracted_dir.join(subfolder);
    let dst_base = profile_dir.join(subfolder);
    for file in files {
        let rel_path = Path::new(&file.path)
            .strip_prefix(subfolder)
            .unwrap_or_else(|_| Path::new(&file.path));
        let src = src_base.join(rel_path);
        let dst = dst_base.join(rel_path);
        if !overwrite && dst.exists() {
            continue;
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, &dst)?;
    }
    Ok(())
}

/// Extract overrides (like options.txt) to the correct config/resourcepack/shaderpack folder in the profile.
/// This should use the temp dir from get_temp_dir(instance_id, modpack_id) after extraction.
pub fn extract_overrides_to_profile(
    extracted_dir: &Path,
    profile_dir: &Path,
) -> anyhow::Result<()> {
    let overrides = extracted_dir.join("overrides");
    if !overrides.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&overrides) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(&overrides)?;
            // Place options.txt and config files in config, packs in their folders
            let dest = if rel == Path::new("options.txt") {
                profile_dir.join("config").join(rel)
            } else if rel.starts_with("resourcepacks/") {
                profile_dir
                    .join("resourcepacks")
                    .join(rel.strip_prefix("resourcepacks/").unwrap())
            } else if rel.starts_with("shaderpacks/") {
                profile_dir
                    .join("shaderpacks")
                    .join(rel.strip_prefix("shaderpacks/").unwrap())
            } else if rel.starts_with("mods/") {
                profile_dir
                    .join("mods")
                    .join(rel.strip_prefix("mods/").unwrap())
            } else {
                profile_dir.join(rel)
            };
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha1::Sha1;
use sha2::{Digest, Sha512};

pub const MRPACK_INDEX_FILE: &str = "modrinth.index.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
    pub path: String,
    pub hashes: std::collections::HashMap<String, String>,
    pub downloads: Vec<String>,
    pub file_size: u64,
    #[serde(default)]
    pub env: Option<MrpackEnv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackEnv {
    pub client: Option<String>,
    pub server: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Client,
    Server,
}

// -----------------------------
// Core API
// -----------------------------

/// Extracts mrpack into a temp directory and returns path
pub fn extract_mrpack(mrpack_path: &Path, out_dir: &Path) -> anyhow::Result<()> {
    let file = File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = safe_join(out_dir, file.name())?;

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Load modrinth.index.json
pub fn load_index(dir: &Path) -> anyhow::Result<MrpackIndex> {
    let path = dir.join(MRPACK_INDEX_FILE);
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

/// Install all files (mods, etc.)
pub fn install_files(index: &MrpackIndex, base_output: &Path, side: Side) -> anyhow::Result<()> {
    use std::path::Path;
    for file in &index.files {
        if !should_include(file, side) {
            continue;
        }

        let rel_path = file.path.clone();
        let target_path: std::path::PathBuf;

        // Handle mods with .disabled in the path
        if rel_path.starts_with("mods/") && rel_path.contains(".disabled") {
            // Remove .disabled from filename
            let path_obj = Path::new(&rel_path);
            let file_name = path_obj
                .file_name()
                .map(|f| f.to_string_lossy().replace(".disabled", ""))
                .unwrap_or_default();
            // Place in mods/disabled/
            let sub_path = path_obj.strip_prefix("mods/").unwrap_or(path_obj);
            let sub_path = sub_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new("").to_path_buf());
            target_path = base_output
                .join("mods/disabled")
                .join(sub_path)
                .join(file_name);
        }
        // Handle resourcepacks
        else if rel_path.starts_with("resourcepacks/") {
            // Place in dedicated resourcepacks folder under 'individual'
            // Use: resourcepacks/<id>/individual/<rest_of_path>
            // Try to extract <rest_of_path> after resourcepacks/
            let rest = rel_path.strip_prefix("resourcepacks/").unwrap_or(&rel_path);
            // Use the modpack id as the folder name
            let modpack_id = generate_modpack_id(index);
            target_path = base_output
                .join("resourcepacks")
                .join(modpack_id)
                .join("individual")
                .join(rest);
        }
        // Default: use path as is
        else {
            target_path = safe_join(base_output, &rel_path)?;
        }

        // Skip if already valid
        if target_path.exists() && verify_file(&target_path, &file.hashes)? {
            continue;
        }

        download_and_write(file, &target_path)?;
    }

    Ok(())
}

/// Copy overrides into instance dir
pub fn apply_overrides(extracted_dir: &Path, instance_dir: &Path) -> anyhow::Result<()> {
    let overrides = extracted_dir.join("overrides");
    if overrides.exists() {
        copy_dir_all(&overrides, instance_dir)?;
    }

    Ok(())
}

// -----------------------------
// Internal helpers
// -----------------------------

fn should_include(file: &MrpackFile, side: Side) -> bool {
    if let Some(env) = &file.env {
        match side {
            Side::Client => {
                if let Some(v) = &env.client {
                    return v != "unsupported";
                }
            }
            Side::Server => {
                if let Some(v) = &env.server {
                    return v != "unsupported";
                }
            }
        }
    }
    true
}

fn download_and_write(file: &MrpackFile, target_path: &Path) -> anyhow::Result<()> {
    let url = file
        .downloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("No download URL"))?;

    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;

    // Verify hash before writing
    verify_bytes(&bytes, &file.hashes)?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut f = File::create(target_path)?;
    f.write_all(&bytes)?;

    Ok(())
}

fn verify_file(
    path: &Path,
    hashes: &std::collections::HashMap<String, String>,
) -> anyhow::Result<bool> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(verify_bytes(&buf, hashes).is_ok())
}

fn verify_bytes(
    bytes: &[u8],
    hashes: &std::collections::HashMap<String, String>,
) -> anyhow::Result<()> {
    if let Some(expected) = hashes.get("sha1") {
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        let result = hex::encode(hasher.finalize());
        if &result != expected {
            return Err(anyhow::anyhow!("SHA1 mismatch"));
        }
    }

    if let Some(expected) = hashes.get("sha512") {
        let mut hasher = Sha512::new();
        hasher.update(bytes);
        let result = hex::encode(hasher.finalize());
        if &result != expected {
            return Err(anyhow::anyhow!("SHA512 mismatch"));
        }
    }

    Ok(())
}

/// Prevent path traversal (VERY IMPORTANT)
fn safe_join(base: &Path, unsafe_path: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(unsafe_path);

    if path.is_absolute() || unsafe_path.contains("..") {
        return Err(anyhow::anyhow!("Invalid path in mrpack"));
    }

    Ok(base.join(path))
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());

        if ty.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
