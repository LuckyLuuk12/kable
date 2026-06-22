use crate::integrations::minecraft::versions::{fabric, forge, neoforge, quilt, vanilla};
use crate::system::{fs, net};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;

fn maven_to_path(name: &str) -> PathBuf {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() != 3 {
        return PathBuf::from(name); // fallback (unsafe but practical)
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    let file = format!("{artifact}-{version}.jar");

    PathBuf::from(group).join(artifact).join(version).join(file)
}

async fn verify_sha1(path: &std::path::Path, expected: &str) -> Result<bool, String> {
    let data = fs::read(path).await?;

    let mut hasher = Sha1::new();
    hasher.update(data);

    let result = format!("{:x}", hasher.finalize());

    Ok(result == expected)
}

fn current_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "osx", // macOS is referred to as "osx" in Minecraft's library rules
        _ => "linux",     // I guess it's safest to fallback to linux...
    }
}

pub async fn ensure_libraries<L>(libs: &[L]) -> Result<(), String>
where
    L: Into<ResolvedLibrary> + Clone,
{
    for lib in libs {
        let lib = lib.clone().into();
        // Check if the library is allowed for the current OS if an OS rule is specified
        if let Some(os_rule) = &lib.os {
            let current_os = current_os();
            if os_rule != &current_os {
                continue; // Skip this library as it is not allowed for the current OS
            }
        }

        let needs_download = match &lib.sha1 {
            None => {
                // no integrity info → fallback to existence check
                !lib.path.exists()
            }
            Some(expected) => {
                match verify_sha1(&lib.path, expected).await {
                    Ok(true) => false, // valid
                    Ok(false) => true, // corrupted
                    Err(_) => true,    // unreadable → re-download
                }
            }
        };

        if !needs_download {
            continue;
        }

        net::download_to_file(&lib.url, &lib.path).await?;

        if let Some(expected) = &lib.sha1 {
            let ok = verify_sha1(&lib.path, expected).await?;
            if !ok {
                return Err(format!("SHA1 mismatch for {}", lib.name));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedLibrary {
    pub name: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    /// This is always a relative path normally from the .minecraft directory.
    pub path: PathBuf,
    /// Vanilla manifest may contain an OS "rule" which is always "Allow"
    pub os: Option<String>,
}

impl From<vanilla::Library> for ResolvedLibrary {
    fn from(lib: vanilla::Library) -> Self {
        ResolvedLibrary {
            // In vanilla, the name is basically the Maven coordinate.
            name: lib.name.clone(),
            url: lib.downloads.artifact.url,
            sha1: Some(lib.downloads.artifact.sha1),
            size: Some(lib.downloads.artifact.size as u64),
            path: match &lib.downloads.artifact.path {
                Some(p) => PathBuf::from(p),
                None => maven_to_path(&lib.name),
            },
            os: lib.rules.as_ref().and_then(|rules| {
                rules
                    .iter()
                    .find_map(|rule| if rule.action == vanilla::Action::Allow { Some(rule.os.name.clone().to_string()) } else { None })
            }),
        }
    }
}

impl From<fabric::Library> for ResolvedLibrary {
    fn from(lib: fabric::Library) -> Self {
        ResolvedLibrary {
            name: lib.name.clone(),
            url: lib.url,
            sha1: lib.sha1.clone(),
            size: lib.size.map(|s| s as u64),
            path: maven_to_path(&lib.name),
            os: None,
        }
    }
}

impl From<forge::Library> for ResolvedLibrary {
    fn from(lib: forge::Library) -> Self {
        ResolvedLibrary {
            name: lib.name.clone(),
            url: lib.downloads.artifact.url,
            sha1: Some(lib.downloads.artifact.sha1.clone()),
            size: Some(lib.downloads.artifact.size as u64),
            path: PathBuf::from(&lib.downloads.artifact.path),
            os: None,
        }
    }
}

impl From<neoforge::Library> for ResolvedLibrary {
    fn from(lib: neoforge::Library) -> Self {
        ResolvedLibrary {
            name: lib.name.clone(),
            url: lib.downloads.artifact.url,
            sha1: Some(lib.downloads.artifact.sha1.clone()),
            size: Some(lib.downloads.artifact.size as u64),
            path: PathBuf::from(&lib.downloads.artifact.path),
            os: None,
        }
    }
}

impl From<quilt::Library> for ResolvedLibrary {
    fn from(lib: quilt::Library) -> Self {
        ResolvedLibrary { name: lib.name.clone(), url: lib.url, sha1: None, size: None, path: maven_to_path(&lib.name), os: None }
    }
}
