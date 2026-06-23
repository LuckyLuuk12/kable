pub mod types;
pub use types::*;

pub async fn load_version_manifest(raw: String) -> Result<McVersionManifest, String> {
    let manifest: McVersionManifest = serde_json::from_str(&raw).map_err(|e| format!("Failed to parse version manifest: {}", e))?;
    Ok(manifest)
}
