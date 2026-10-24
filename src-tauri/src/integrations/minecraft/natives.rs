use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::integrations::minecraft::versions::types::Library;

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
            if !self.is_allowed(lib) {
                continue;
            }

            let Some((url, jar_path)) = self.resolve_native_artifact(lib)? else {
                continue;
            };

            let full_jar_path = self.mc_root.join(&jar_path);

            if !seen.insert(full_jar_path.clone()) {
                continue;
            }

            if !self.file_exists(&full_jar_path) {
                if let Some(parent) = full_jar_path.parent() {
                    crate::system::fs::create_dir(parent).await?;
                }

                crate::system::net::download_to_file(&url, &full_jar_path).await?;
            }

            self.extract_natives(&full_jar_path, &target_dir)?;
        }

        Ok(target_dir)
    }

    fn resolve_native_artifact(&self, lib: &Library) -> Result<Option<(String, PathBuf)>, String> {
        let Some(os) = self.native_os() else {
            return Ok(None);
        };

        let Some(natives) = &lib.natives else {
            return Ok(None);
        };

        let Some(classifier) = natives.get(os) else {
            return Ok(None);
        };

        let Some(downloads) = &lib.downloads else {
            return Ok(None);
        };

        let Some(classifiers) = &downloads.classifiers else {
            return Ok(None);
        };

        let Some(artifact) = classifiers.get(classifier) else {
            return Ok(None);
        };

        let Some(url) = &artifact.url else {
            return Ok(None);
        };

        let Some(path) = &artifact.path else {
            return Ok(None);
        };

        Ok(Some((url.clone(), PathBuf::from(path))))
    }

    fn native_os(&self) -> Option<&'static str> {
        match std::env::consts::OS {
            "windows" => Some("windows"),
            "linux" => Some("linux"),
            "macos" => Some("osx"),
            _ => None,
        }
    }

    fn extract_natives(&self, jar_path: &Path, target_dir: &Path) -> Result<(), String> {
        let file = std::fs::File::open(jar_path).map_err(|e| format!("Failed to open native jar {}: {e}", jar_path.display()))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read native jar {}: {e}", jar_path.display()))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| format!("Zip read error in {}: {e}", jar_path.display()))?;

            if entry.is_dir() {
                continue;
            }

            let name = entry.name();

            if !Self::is_native_file(name) {
                continue;
            }

            let relative_path = Path::new(name);

            if relative_path.is_absolute()
                || relative_path.components().any(|component| matches!(component, std::path::Component::ParentDir))
            {
                return Err(format!("Unsafe path in native jar {}: {name}", jar_path.display()));
            }

            let out_path = target_dir.join(relative_path);

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create native directory {}: {e}", parent.display()))?;
            }

            let out_str = out_path.to_string_lossy();

            let mut outfile = std::fs::File::create(&out_path).map_err(|e| format!("Failed to write native {out_str}: {e}"))?;

            std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("Failed to extract native {out_str}: {e}"))?;
        }

        Ok(())
    }

    fn is_native_file(path: &str) -> bool {
        path.ends_with(".dll") || path.ends_with(".so") || path.ends_with(".dylib")
    }

    fn is_allowed(&self, lib: &Library) -> bool {
        let Some(rules) = &lib.rules else {
            return true;
        };

        let mut allowed = false;

        for rule in rules {
            if !self.rule_matches(rule) {
                continue;
            }

            match rule.action.as_deref() {
                Some("allow") => allowed = true,
                Some("disallow") => allowed = false,
                _ => {}
            }
        }

        allowed
    }

    fn rule_matches(&self, rule: &crate::integrations::minecraft::versions::types::LibraryRule) -> bool {
        let Some(os) = &rule.os else {
            return true;
        };

        let current_os = std::env::consts::OS;

        if let Some(name) = &os.name {
            let matches = match name.as_str() {
                "windows" => current_os == "windows",
                "linux" => current_os == "linux",
                "osx" | "mac" | "macos" => current_os == "macos",
                _ => false,
            };

            if !matches {
                return false;
            }
        }

        if let Some(arch) = &os.arch {
            let current_arch = std::env::consts::ARCH;

            if arch != current_arch {
                return false;
            }
        }

        true
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.is_file()
    }
}
