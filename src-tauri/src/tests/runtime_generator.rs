use crate::integrations::minecraft::versions::McVersionManifest;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;
use zip::write::SimpleFileOptions;

pub struct TestContext {
    _temp_dir: TempDir,
    pub root: PathBuf,
    pub mc_dir: PathBuf,
}

impl TestContext {
    pub async fn new() -> Result<Self, String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

        let root = temp_dir.path().to_path_buf();
        let mc_dir = root.join(".minecraft");

        fs::create_dir_all(mc_dir.join("versions")).await.map_err(|e| e.to_string())?;

        fs::create_dir_all(mc_dir.join("libraries")).await.map_err(|e| e.to_string())?;

        fs::create_dir_all(mc_dir.join("assets").join("indexes")).await.map_err(|e| e.to_string())?;

        fs::create_dir_all(mc_dir.join("assets").join("objects")).await.map_err(|e| e.to_string())?;

        Ok(Self { _temp_dir: temp_dir, root, mc_dir })
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.mc_dir.join("versions")
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.mc_dir.join("libraries")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.mc_dir.join("assets")
    }

    pub async fn create_version(&self, id: &str, manifest: &McVersionManifest) -> Result<PathBuf, String> {
        let version_dir = self.versions_dir().join(id);

        fs::create_dir_all(&version_dir).await.map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;

        let path = version_dir.join(format!("{id}.json"));

        fs::write(&path, json).await.map_err(|e| e.to_string())?;

        Ok(path)
    }

    pub async fn create_library_jar(&self, relative_path: &str) -> Result<PathBuf, String> {
        self.create_jar(self.libraries_dir().join(relative_path), &[("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n")]).await
    }

    pub async fn create_native_jar(&self, relative_path: &str) -> Result<PathBuf, String> {
        #[cfg(windows)]
        let native_name = "test.dll";

        #[cfg(target_os = "linux")]
        let native_name = "test.so";

        #[cfg(target_os = "macos")]
        let native_name = "test.dylib";

        self.create_jar(
            self.libraries_dir().join(relative_path),
            &[("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"), (native_name, b"fake-native")],
        )
        .await
    }

    async fn create_jar(&self, path: PathBuf, entries: &[(&str, &[u8])]) -> Result<PathBuf, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        let entries: Vec<(String, Vec<u8>)> = entries.iter().map(|(name, contents)| (name.to_string(), contents.to_vec())).collect();

        let path_clone = path.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let file = std::fs::File::create(&path_clone).map_err(|e| e.to_string())?;

            let mut zip = zip::ZipWriter::new(file);

            for (name, contents) in entries {
                zip.start_file(name, SimpleFileOptions::default()).map_err(|e| e.to_string())?;

                use std::io::Write;
                zip.write_all(&contents).map_err(|e| e.to_string())?;
            }

            zip.finish().map_err(|e| e.to_string())?;

            Ok(())
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(path)
    }

    pub async fn create_asset(&self, logical_name: &str, contents: &[u8], index_id: &str) -> Result<String, String> {
        let hash = {
            let mut hasher = Sha1::new();
            hasher.update(contents);
            format!("{:x}", hasher.finalize())
        };

        let prefix = &hash[..2];

        let object_path = self.assets_dir().join("objects").join(prefix).join(&hash);

        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        fs::write(&object_path, contents).await.map_err(|e| e.to_string())?;

        let index_path = self.assets_dir().join("indexes").join(format!("{index_id}.json"));

        let index_json = serde_json::json!({
            "objects": {
                logical_name: {
                    "hash": hash,
                    "size": contents.len()
                }
            }
        });

        fs::write(&index_path, serde_json::to_vec_pretty(&index_json).map_err(|e| e.to_string())?)
            .await
            .map_err(|e| e.to_string())?;

        Ok(hash)
    }

    pub async fn read_manifest(&self, id: &str) -> Result<McVersionManifest, String> {
        let path = self.versions_dir().join(id).join(format!("{id}.json"));

        let contents = fs::read_to_string(path).await.map_err(|e| e.to_string())?;

        serde_json::from_str(&contents).map_err(|e| e.to_string())
    }

    pub async fn write_file(&self, path: impl AsRef<Path>, contents: &[u8]) -> Result<(), String> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        fs::write(path, contents).await.map_err(|e| e.to_string())
    }
}
