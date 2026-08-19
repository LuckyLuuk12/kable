use std::{collections::HashSet, path::PathBuf};

use crate::{
    constants::LIBRARIES_DIR,
    integrations::minecraft::versions::types::{DownloadArtifact, Library, LibraryRule},
    system::{fs, net},
};

pub struct LibraryResolver {
    pub mc_root: PathBuf,
    pub libraries_dir: PathBuf,
}

impl LibraryResolver {
    pub fn new() -> Result<Self, String> {
        let mc_root = crate::system::fs::mc_dir()?;
        let libraries_dir = mc_root.join(LIBRARIES_DIR);

        Ok(Self { mc_root, libraries_dir })
    }

    pub fn resolve_classpath(&self, libs: &[Library]) -> Result<String, String> {
        let mut seen = HashSet::<PathBuf>::new();
        let mut entries = Vec::new();

        for lib in libs {
            if self.is_native_library(lib) {
                continue;
            }

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

            if full_path.is_file() {
                continue;
            }

            if let Some(parent) = full_path.parent() {
                fs::create_dir(parent).await?;
            }

            net::download_to_file(&url, &full_path).await?;
        }

        Ok(())
    }

    fn resolve_library_path(&self, lib: &Library) -> Result<Option<PathBuf>, String> {
        if let Some(artifact) = self.get_artifact(lib) {
            if let Some(path) = &artifact.path {
                return Ok(Some(PathBuf::from(path)));
            }
        }

        if let Some(name) = &lib.name {
            return Ok(Some(PathBuf::from(self.maven_to_path(name))));
        }

        Ok(None)
    }

    fn resolve_download(&self, lib: &Library) -> Result<Option<(String, PathBuf)>, String> {
        if let Some(artifact) = self.get_artifact(lib) {
            if let (Some(url), Some(path)) = (&artifact.url, &artifact.path) {
                return Ok(Some((url.clone(), PathBuf::from(path))));
            }
        }

        if let (Some(url), Some(name)) = (&lib.url, &lib.name) {
            let path = self.maven_to_path(name);

            return Ok(Some((url.clone(), PathBuf::from(path))));
        }

        Ok(None)
    }

    fn get_artifact<'a>(&self, lib: &'a Library) -> Option<&'a DownloadArtifact> {
        lib.downloads.as_ref()?.artifact.as_ref()
    }

    fn maven_to_path(&self, name: &str) -> String {
        let parts: Vec<&str> = name.split(':').collect();

        if parts.len() < 3 {
            return format!("{name}.jar");
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];

        match parts.get(3) {
            Some(classifier) => {
                format!("{group}/{artifact}/{version}/{artifact}-{version}-{classifier}.jar")
            }
            None => {
                format!("{group}/{artifact}/{version}/{artifact}-{version}.jar")
            }
        }
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

    fn rule_matches(&self, rule: &LibraryRule) -> bool {
        let Some(os) = &rule.os else {
            return true;
        };

        let current_os = std::env::consts::OS;

        if let Some(name) = &os.name {
            match name.as_str() {
                "windows" if current_os == "windows" => {}
                "linux" if current_os == "linux" => {}
                "osx" | "mac" | "macos" if current_os == "macos" => {}
                _ => return false,
            }
        }

        if let Some(arch) = &os.arch {
            if arch != std::env::consts::ARCH {
                return false;
            }
        }

        true
    }

    fn is_native_library(&self, lib: &Library) -> bool {
        let Some(natives) = &lib.natives else {
            return false;
        };

        let Some(os) = self.native_os() else {
            return false;
        };

        natives.contains_key(os)
    }

    fn native_os(&self) -> Option<&'static str> {
        match std::env::consts::OS {
            "windows" => Some("windows"),
            "linux" => Some("linux"),
            "macos" => Some("osx"),
            _ => None,
        }
    }
}
