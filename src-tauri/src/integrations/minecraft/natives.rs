use crate::system::fs;
use std::path::PathBuf;

pub async fn extract_natives(libs: &[PathBuf], version_id: &str) -> Result<PathBuf, String> {
    let mc_dir = fs::mc_dir()?;
    let natives_dir = mc_dir.join("versions").join(version_id).join("natives");

    tokio::fs::create_dir_all(&natives_dir).await.map_err(|e| e.to_string())?;

    for lib in libs {
        if !lib.exists() {
            continue;
        }

        // very simplified: in real version, check classifier = natives-*
        if lib.to_string_lossy().contains("natives") {
            let file = std::fs::File::open(lib).map_err(|e| e.to_string())?;

            let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

            for i in 0..archive.len() {
                let mut f = archive.by_index(i).map_err(|e| e.to_string())?;

                let out_path = natives_dir.join(f.name());

                if let Some(parent) = out_path.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
                }

                if f.is_file() {
                    let mut bytes = Vec::new();
                    std::io::copy(&mut f, &mut bytes).map_err(|e| e.to_string())?;

                    tokio::fs::write(out_path, bytes).await.map_err(|e| e.to_string())?;
                }
            }
        }
    }

    Ok(natives_dir)
}
