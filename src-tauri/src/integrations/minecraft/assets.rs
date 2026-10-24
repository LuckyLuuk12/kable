use crate::integrations::minecraft::versions::types::McVersionManifest;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexFile {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub struct AssetResolver {
    pub mc_root: PathBuf,
    pub assets_dir: PathBuf,
}

impl AssetResolver {
    pub fn new() -> Result<Self, String> {
        let mc_root = crate::system::fs::mc_dir()?;
        let assets_dir = mc_root.join(crate::constants::ASSETS_DIR);

        Ok(Self { mc_root, assets_dir })
    }

    pub async fn ensure_assets(&self, manifest: &McVersionManifest) -> Result<PathBuf, String> {
        let asset_index = manifest.asset_index.as_ref().ok_or("Missing asset index")?;

        let index_id = asset_index.id.as_deref().unwrap_or("unknown");

        let index_path = self.index_path(index_id);

        if let Some(url) = &asset_index.url {
            if !index_path.exists() {
                crate::system::fs::create_dir(index_path.parent().unwrap()).await?;
                crate::system::net::download_to_file(url, &index_path).await?;
            }
        }

        let index = self.read_index(&index_path)?;

        let objects = index.objects;

        let mut tasks = Vec::new();

        for value in objects.values() {
            let hash = value.hash.as_str();

            let size = value.size;

            let (url, path) = self.resolve_asset(hash);

            let expected_hash = hash.to_string();

            tasks.push(async move {
                if path.exists() {
                    if let Ok(actual) = self.sha1_file(&path) {
                        if actual == expected_hash {
                            return Ok(());
                        }
                    }
                }

                crate::system::fs::create_dir(path.parent().unwrap()).await?;
                crate::system::net::download_to_file(&url, &path).await?;

                // optional: size sanity check hook point
                let _ = size;

                Ok::<(), String>(())
            });
        }

        try_join_all(tasks).await?;

        Ok(self.assets_dir.clone())
    }

    fn resolve_asset(&self, hash: &str) -> (String, PathBuf) {
        let prefix = &hash[0..2];

        let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);

        let path = self.assets_dir.join("objects").join(prefix).join(hash);

        (url, path)
    }

    fn index_path(&self, index_id: &str) -> PathBuf {
        self.assets_dir.join("indexes").join(format!("{index_id}.json"))
    }

    fn read_index(&self, path: &Path) -> Result<AssetIndexFile, String> {
        let data = fs::read_to_string(path).map_err(|e| format!("Failed to read asset index: {e}"))?;

        serde_json::from_str(&data).map_err(|e| format!("Invalid asset JSON: {e}"))
    }

    pub fn virtual_assets_path(&self) -> PathBuf {
        self.assets_dir.join("virtual")
    }

    fn sha1_file(&self, path: &Path) -> Result<String, String> {
        use std::io::Read;

        let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;

        let mut hasher = sha1::Sha1::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer).map_err(|e| format!("Read error: {e}"))?;

            if n == 0 {
                break;
            }

            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}
