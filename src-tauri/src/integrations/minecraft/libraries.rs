use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use crate::{
    constants::LIBRARIES_DIR,
    integrations::minecraft::versions::types::{DownloadArtifact, Library, LibraryRule},
    system::{fs, net},
    Logger,
};

pub struct LibraryResolver {
    pub mc_root: PathBuf,
    pub libraries_dir: PathBuf,
}

impl LibraryResolver {
    pub fn new() -> Result<Self, String> {
        let mc_root = fs::mc_dir()?;
        let libraries_dir = mc_root.join(LIBRARIES_DIR);

        Ok(Self { mc_root, libraries_dir })
    }

    pub fn resolve_classpath(&self, libs: &[Library]) -> Result<String, String> {
        let mut seen = HashSet::<PathBuf>::new();
        let mut entries = Vec::new();

        for lib in libs {
            if self.is_native_library(lib) || !self.is_allowed(lib) {
                continue;
            }

            let Some(path) = self.resolve_library_path(lib)? else {
                return Err(format!("Cannot resolve classpath path for library '{}'", lib.name.as_deref().unwrap_or("<unnamed>")));
            };

            Self::validate_relative_path(&path)?;

            let full_path = self.libraries_dir.join(&path);

            if seen.insert(full_path.clone()) {
                entries.push(full_path.to_string_lossy().into_owned());
            }
        }

        Ok(entries.join(if cfg!(windows) { ";" } else { ":" }))
    }

    pub async fn ensure_downloaded(&self, libs: &[Library]) -> Result<(), String> {
        for lib in libs {
            if self.is_native_library(lib) || !self.is_allowed(lib) {
                continue;
            }

            let Some((url, rel_path, artifact)) = self.resolve_download(lib)? else {
                return Err(format!(
                    "Cannot resolve download for library '{}': no artifact URL or Maven repository URL and coordinates",
                    lib.name.as_deref().unwrap_or("<unnamed>")
                ));
            };

            Self::validate_relative_path(&rel_path)?;

            let full_path = self.libraries_dir.join(&rel_path);

            if full_path.is_file() {
                if let Some(artifact) = artifact {
                    if self.verify_artifact(&full_path, artifact).await? {
                        continue;
                    }

                    Logger::warn_global(
                        format!("Library {} failed integrity verification; downloading again", lib.name.as_deref().unwrap_or("<unnamed>"))
                            .as_str(),
                        None,
                    );

                    fs::remove_file(&full_path).await?;
                } else {
                    continue;
                }
            }

            if let Some(parent) = full_path.parent() {
                fs::create_dir(parent).await?;
            }

            net::download_to_file(&url, &full_path).await?;

            if !full_path.is_file() {
                return Err(format!("Library download completed but file does not exist: {}", full_path.display()));
            }

            if let Some(artifact) = artifact {
                if !self.verify_artifact(&full_path, artifact).await? {
                    fs::remove_file(&full_path).await?;

                    return Err(format!(
                        "Downloaded library '{}' failed integrity verification",
                        lib.name.as_deref().unwrap_or("<unnamed>")
                    ));
                }
            }
        }

        Ok(())
    }

    fn resolve_library_path(&self, lib: &Library) -> Result<Option<PathBuf>, String> {
        if let Some(artifact) = self.get_artifact(lib) {
            if let Some(path) = &artifact.path {
                let path = PathBuf::from(path);
                Self::validate_relative_path(&path)?;
                return Ok(Some(path));
            }
        }

        if let Some(name) = &lib.name {
            let path = PathBuf::from(self.maven_to_path(name)?);
            Self::validate_relative_path(&path)?;
            return Ok(Some(path));
        }

        Ok(None)
    }

    fn resolve_download<'a>(&self, lib: &'a Library) -> Result<Option<(String, PathBuf, Option<&'a DownloadArtifact>)>, String> {
        if let Some(artifact) = self.get_artifact(lib) {
            if let (Some(url), Some(path)) = (&artifact.url, &artifact.path) {
                let path = PathBuf::from(path);
                Self::validate_relative_path(&path)?;

                return Ok(Some((url.clone(), path, Some(artifact))));
            }
        }

        let (Some(repository_url), Some(name)) = (&lib.url, &lib.name) else {
            return Ok(None);
        };

        let relative_path = PathBuf::from(self.maven_to_path(name)?);
        Self::validate_relative_path(&relative_path)?;

        let relative_url_path = relative_path.to_string_lossy().replace('\\', "/");
        let download_url = format!("{}/{}", repository_url.trim_end_matches('/'), relative_url_path);

        Logger::debug_global(
            format!("Resolved library '{}' to download URL: {} and relative path: {}", name, download_url, relative_path.display())
                .as_str(),
            None,
        );

        Ok(Some((download_url, relative_path, None)))
    }

    fn get_artifact<'a>(&self, lib: &'a Library) -> Option<&'a DownloadArtifact> {
        lib.downloads.as_ref()?.artifact.as_ref()
    }

    fn maven_to_path(&self, name: &str) -> Result<String, String> {
        let parts: Vec<&str> = name.split(':').collect();

        if parts.len() < 3 || parts.len() > 4 {
            return Err(format!("Invalid Maven library coordinates: '{}'", name));
        }

        let group = parts[0];
        let artifact = parts[1];
        let version = parts[2];

        if group.is_empty()
            || artifact.is_empty()
            || version.is_empty()
            || [group, artifact, version].iter().any(|part| part.contains('/') || part.contains('\\') || *part == "." || *part == "..")
        {
            return Err(format!("Invalid Maven library coordinates: '{}'", name));
        }

        let group_path = group.replace('.', "/");

        let file_name = match parts.get(3) {
            Some(classifier) if !classifier.is_empty() => {
                format!("{artifact}-{version}-{classifier}.jar")
            }
            Some(_) => {
                return Err(format!("Invalid Maven library classifier: '{}'", name));
            }
            None => format!("{artifact}-{version}.jar"),
        };

        Ok(format!("{group_path}/{artifact}/{version}/{file_name}"))
    }

    fn validate_relative_path(path: &Path) -> Result<(), String> {
        if path.as_os_str().is_empty()
            || path.is_absolute()
            || path.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
        {
            return Err(format!("Invalid library relative path: '{}'", path.display()));
        }

        Ok(())
    }

    async fn verify_artifact(&self, path: &Path, artifact: &DownloadArtifact) -> Result<bool, String> {
        let bytes = fs::read(path).await?;

        if let Some(expected_size) = artifact.size {
            if bytes.len() as i64 != expected_size {
                return Ok(false);
            }
        }

        if let Some(expected_sha1) = &artifact.sha1 {
            use sha1::{Digest, Sha1};

            let mut hasher = Sha1::new();
            hasher.update(&bytes);
            let actual = hex::encode(hasher.finalize());

            if !actual.eq_ignore_ascii_case(expected_sha1) {
                return Ok(false);
            }
        }

        if let Some(expected_sha256) = &artifact.sha256 {
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = hex::encode(hasher.finalize());

            if !actual.eq_ignore_ascii_case(expected_sha256) {
                return Ok(false);
            }
        }

        if let Some(expected_sha512) = &artifact.sha512 {
            use sha2::{Digest, Sha512};

            let mut hasher = Sha512::new();
            hasher.update(&bytes);
            let actual = hex::encode(hasher.finalize());

            if !actual.eq_ignore_ascii_case(expected_sha512) {
                return Ok(false);
            }
        }

        if let Some(expected_md5) = &artifact.md5 {
            use md5::{Digest, Md5};

            let mut hasher = Md5::new();
            hasher.update(&bytes);
            let actual = hex::encode(hasher.finalize());

            if !actual.eq_ignore_ascii_case(expected_md5) {
                return Ok(false);
            }
        }

        Ok(true)
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

        let current_os = match std::env::consts::OS {
            "windows" => "windows",
            "linux" => "linux",
            "macos" => "osx",
            other => other,
        };

        if let Some(name) = &os.name {
            if name != current_os {
                return false;
            }
        }

        if let Some(arch) = &os.arch {
            if arch != std::env::consts::ARCH {
                return false;
            }
        }

        if let Some(version_regex) = &os.version {
            let current_version = std::env::var("OS_VERSION").unwrap_or_default();

            if !current_version.is_empty() {
                match regex::Regex::new(version_regex) {
                    Ok(regex) if regex.is_match(&current_version) => {}
                    _ => return false,
                }
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
