/// This module contains generated types from https://app.quicktype.io/ for the .minecraft/versions/<version>/<version>.json files, which are used to launch Minecraft.
/// It will also contain some helper functions to load and "merge" these manifests which is required for Forge and Fabric, as they inherit from a vanilla version and override some fields.
pub mod fabric;
pub mod forge;
// pub mod iris_fabric;
pub mod neoforge;
pub mod quilt;
pub mod vanilla;

pub enum VersionManifest {
    Vanilla(vanilla::VanillaVersionManifest),
    Forge(forge::ForgeVersionManifest),
    Fabric(fabric::FabricVersionManifest),
    IrisFabric(fabric::FabricVersionManifest), // Iris Fabric is just Fabric with Iris mod preinstalled
    NeoForge(neoforge::NeoforgeVersionManifest),
    Quilt(quilt::QuiltVersionManifest),
}

pub async fn load_version_manifest(version: api_types::profiles::ProfileVersion) -> Result<VersionManifest, String> {
    let contents =
        crate::system::fs::read_str(crate::system::fs::mc_dir()?.join("versions").join(&version.id).join(format!("{}.json", version.id)))
            .await?;
    // Now that we have the contents we match on version.loader to determine which type to deserialize into, and then we deserialize the contents into that type and return it wrapped in the VersionManifest enum.
    match version.loader {
        api_types::profiles::LoaderKind::Vanilla => {
            Ok(VersionManifest::Vanilla(serde_json::from_str(&contents).map_err(|e| e.to_string())?))
        }
        api_types::profiles::LoaderKind::Forge => Ok(VersionManifest::Forge(serde_json::from_str(&contents).map_err(|e| e.to_string())?)),
        api_types::profiles::LoaderKind::Fabric => Ok(VersionManifest::Fabric(serde_json::from_str(&contents).map_err(|e| e.to_string())?)),
        api_types::profiles::LoaderKind::IrisFabric => {
            Ok(VersionManifest::IrisFabric(serde_json::from_str(&contents).map_err(|e| e.to_string())?))
        }
        api_types::profiles::LoaderKind::NeoForge => {
            Ok(VersionManifest::NeoForge(serde_json::from_str(&contents).map_err(|e| e.to_string())?))
        }
        api_types::profiles::LoaderKind::Quilt => Ok(VersionManifest::Quilt(serde_json::from_str(&contents).map_err(|e| e.to_string())?)),
    }
}
