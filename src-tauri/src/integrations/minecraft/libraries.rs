use std::{collections::HashSet, path::PathBuf};

use crate::{
    constants::LIBRARIES_DIR,
    integrations::minecraft::versions::types::LibraryRule,
    system::{fs, net},
};

use crate::integrations::minecraft::versions::types::{DownloadArtifact, Library};

// ============================================================
// RESOLVER
// ============================================================

pub struct LibraryResolver {
    mc_root: PathBuf,
    libraries_dir: PathBuf,
}

impl LibraryResolver {
    pub fn new() -> Result<Self, String> {
        let mc_root = crate::system::fs::mc_dir()?;
        let libraries_dir = mc_root.join(LIBRARIES_DIR);

        Ok(Self { mc_root, libraries_dir })
    }

    // --------------------------------------------------------
    // CLASSPATH
    // --------------------------------------------------------

    pub fn resolve_classpath(&self, libs: &[Library]) -> Result<String, String> {
        let mut seen = HashSet::<PathBuf>::new();
        let mut entries: Vec<String> = Vec::new();

        for lib in libs {
            if !self.is_allowed(lib) {
                continue;
            }

            let Some(path) = self.resolve_library_path(lib)? else {
                continue;
            };

            let full_path = self.libraries_dir.join(&path);

            if seen.insert(full_path.clone()) {
                entries.push(full_path.to_string_lossy().to_string());
            }
        }

        Ok(entries.join(if cfg!(windows) { ";" } else { ":" }))
    }

    // --------------------------------------------------------
    // DOWNLOADS
    // --------------------------------------------------------

    pub async fn ensure_downloaded(&self, libs: &[Library]) -> Result<(), String> {
        for lib in libs {
            if self.is_native_library(lib) {
                continue;
            }

            if !self.is_allowed(lib) {
                continue;
            }

            let Some((url, rel_path)) = self.resolve_download(lib)? else {
                continue;
            };

            let full_path = self.libraries_dir.join(&rel_path);

            if full_path.exists() {
                continue;
            }

            if let Some(parent) = full_path.parent() {
                fs::create_dir(parent).await?;
            }

            net::download_to_file(&url, &full_path).await?;
        }

        Ok(())
    }

    // ============================================================
    // PATH RESOLUTION
    // ============================================================

    fn resolve_library_path(&self, lib: &Library) -> Result<Option<PathBuf>, String> {
        // Priority 1: artifact path (correct for Forge/NeoForge/Fabric)
        if let Some(artifact) = self.get_artifact(lib) {
            if let Some(path) = &artifact.path {
                return Ok(Some(PathBuf::from(path)));
            }
        }

        // Priority 2: maven-style name
        if let Some(name) = &lib.name {
            return Ok(Some(PathBuf::from(self.maven_to_path(name))));
        }

        Ok(None)
    }

    fn resolve_download(&self, lib: &Library) -> Result<Option<(String, PathBuf)>, String> {
        let artifact = self.get_artifact(lib);

        // ----------------------------------------------------
        // Case 1: full artifact download exists
        // ----------------------------------------------------
        if let Some(artifact) = artifact {
            if let (Some(url), Some(path)) = (&artifact.url, &artifact.path) {
                return Ok(Some((url.clone(), PathBuf::from(path))));
            }
        }

        // ----------------------------------------------------
        // Case 2: fallback to library.url + maven path
        // ----------------------------------------------------
        if let (Some(url), Some(name)) = (&lib.url, &lib.name) {
            let path = self.maven_to_path(name);
            return Ok(Some((url.clone(), PathBuf::from(path))));
        }

        Ok(None)
    }

    // ============================================================
    // ARTIFACT ACCESS
    // ============================================================

    fn get_artifact<'a>(&self, lib: &'a Library) -> Option<&'a DownloadArtifact> {
        lib.downloads.as_ref()?.artifact.as_ref()
    }

    // ============================================================
    // MAVEN RESOLUTION
    // ============================================================

    fn maven_to_path(&self, name: &str) -> String {
        let parts: Vec<&str> = name.split(':').collect();

        if parts.len() < 3 {
            return format!("{}.jar", name);
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];

        let classifier = parts.get(3);

        match classifier {
            Some(c) => format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, c),
            None => format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version),
        }
    }

    // ============================================================
    // RULE FILTER
    // ============================================================

    fn is_allowed(&self, lib: &Library) -> bool {
        let Some(rules) = &lib.rules else {
            return true;
        };

        let mut allowed = None;

        for rule in rules {
            let matches = self.rule_matches(rule);

            if matches {
                match rule.action.as_deref() {
                    Some("allow") => allowed = Some(true),
                    Some("disallow") => allowed = Some(false),
                    _ => {}
                }
            }
        }

        allowed.unwrap_or(true)
    }

    fn rule_matches(&self, rule: &LibraryRule) -> bool {
        let Some(os) = &rule.os else {
            return true;
        };

        let current = std::env::consts::OS;

        if let Some(name) = &os.name {
            match name.as_str() {
                "windows" => current == "windows",
                "linux" => current == "linux",
                "osx" | "mac" | "macos" => current == "macos",
                _ => return false,
            }
        } else {
            true
        }
    }

    fn is_native_library(&self, lib: &Library) -> bool {
        let Some(name) = &lib.name else {
            return false;
        };

        name.contains("natives-")
    }
}
