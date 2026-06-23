use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::integrations::minecraft::versions::types::{Library, LibraryDownloads};

pub struct NativeResolver {
    pub mc_root: PathBuf,
    pub natives_dir: PathBuf,
    pub libraries_dir: PathBuf,
}

impl NativeResolver {
    pub fn new() -> Result<Self, String> {
        let mc_root = crate::system::fs::mc_dir()?;

        let natives_dir = mc_root.join(crate::constants::NATIVES_DIR);
        let libraries_dir = mc_root.join(crate::constants::LIBRARIES_DIR);

        Ok(Self { mc_root, natives_dir, libraries_dir })
    }

    pub async fn ensure_natives(&self, libs: &[Library], version_id: &str) -> Result<PathBuf, String> {
        let target_dir = self.natives_dir.join(version_id);

        crate::system::fs::create_dir(&target_dir).await?;

        let mut seen = HashSet::new();

        for lib in libs {
            let Some(download) = &lib.downloads else {
                continue;
            };

            let Some((url, jar_path)) = self.resolve_native_artifact(lib, download)? else {
                continue;
            };

            let full_jar_path = self.mc_root.join(&jar_path);

            if !seen.insert(full_jar_path.clone()) {
                continue;
            }

            if !self.file_exists(&full_jar_path) {
                crate::system::net::download_to_file(&url, &full_jar_path).await?;
            }

            self.extract_natives(&full_jar_path, &target_dir)?;
        }

        Ok(target_dir)
    }

    fn resolve_native_artifact(&self, lib: &Library, downloads: &LibraryDownloads) -> Result<Option<(String, PathBuf)>, String> {
        let artifact = match &downloads.artifact {
            Some(a) => a,
            None => return Ok(None),
        };

        let Some(name) = &lib.name else {
            return Ok(None);
        };

        let classifier = self.native_classifier();

        if classifier.is_none() {
            return Ok(None);
        }

        let classifier = classifier.unwrap();

        let base = name.split(':').collect::<Vec<&str>>();

        if base.len() < 3 {
            return Ok(None);
        }

        let group = base[0];
        let artifact_id = base[1];
        let version = base[2];

        let jar_name = format!("{}/{}/{}/{}-{}-{}.jar", group.replace('.', "/"), artifact_id, version, artifact_id, version, classifier);

        let url = match &artifact.url {
            Some(u) => u.clone(),
            None => return Ok(None),
        };

        Ok(Some((url, self.libraries_dir.join(jar_name))))
    }

    fn native_classifier(&self) -> Option<&'static str> {
        match std::env::consts::OS {
            "windows" => Some("natives-windows"),
            "linux" => Some("natives-linux"),
            "macos" => Some("natives-osx"),
            _ => None,
        }
    }

    fn extract_natives(&self, jar_path: &Path, target_dir: &Path) -> Result<(), String> {
        let file = std::fs::File::open(jar_path).map_err(|e| format!("Failed to open native jar: {e}"))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {e}"))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| format!("Zip read error: {e}"))?;

            let name = entry.name().to_string();

            if !self.is_native_file(&name) {
                continue;
            }

            let out_path = target_dir.join(name);

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dirs: {e}"))?;
            }

            let mut outfile = std::fs::File::create(&out_path).map_err(|e| format!("Failed to write native: {e}"))?;

            std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("Failed to extract native: {e}"))?;
        }

        Ok(())
    }

    fn is_native_file(&self, path: &str) -> bool {
        !(path.starts_with("META-INF/") || path.ends_with(".sha1") || path.ends_with(".sf") || path.ends_with(".RSA"))
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
