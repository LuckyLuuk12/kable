use crate::integrations::minecraft::versions::vanilla::Library;
use crate::system::{fs, net};
use std::path::PathBuf;

pub fn library_allowed(lib: &Library) -> bool {
    let Some(rules) = &lib.rules else {
        return true;
    };

    rules.iter().any(|r| matches!(r.action, crate::integrations::minecraft::versions::vanilla::Action::Allow))
}

pub async fn ensure_libraries(libs: &[Library]) -> Result<Vec<PathBuf>, String> {
    let mc_dir = fs::mc_dir()?;
    let lib_dir = mc_dir.join("libraries"); // TODO: make this a constant as well

    let mut classpath = Vec::new();

    for lib in libs {
        if !library_allowed(lib) {
            continue;
        }

        let artifact = &lib.downloads.artifact;

        let path = lib_dir.join(artifact.path.as_ref().cloned().unwrap_or_else(|| default_lib_path(&lib.name)));

        if !path.exists() {
            net::download_to_file(&artifact.url, &path).await?;
        }

        classpath.push(path);
    }

    Ok(classpath)
}

fn default_lib_path(name: &str) -> String {
    // group:artifact:version -> group/artifact/version/artifact-version.jar
    let parts: Vec<&str> = name.split(':').collect();

    if parts.len() != 3 {
        return name.to_string();
    }

    let (group, artifact, version) = (parts[0], parts[1], parts[2]);

    format!("{}/{}/{}/{}-{}.jar", group.replace('.', "/"), artifact, version, artifact, version)
}
