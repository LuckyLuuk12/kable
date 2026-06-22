use serde::{Deserialize, Serialize};

use crate::integrations::minecraft::versions::vanilla::VanillaVersionManifest;

/// This module contains generated types from https://app.quicktype.io/ for the .minecraft/versions/<version>/<version>.json files, which are used to launch Minecraft.
/// It will also contain some helper functions to load and "merge" these manifests which is required for Forge and Fabric, as they inherit from a vanilla version and override some fields.
pub mod fabric;
pub mod forge;
// pub mod iris_fabric;
pub mod neoforge;
pub mod quilt;
pub mod vanilla;

/// This struct represents the merged version manifest for a Minecraft version, which is used to launch the game.
/// It contains the version manifest for the version being launched, as well as the parent version manifest if it exists (for Forge and Fabric).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifests {
    pub version: VersionManifest,
    pub parent_version: Option<VanillaVersionManifest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VersionManifest {
    Vanilla(vanilla::VanillaVersionManifest),
    Forge(forge::ForgeVersionManifest),
    Fabric(fabric::FabricVersionManifest),
    IrisFabric(fabric::FabricVersionManifest), // Iris Fabric is just Fabric with Iris mod preinstalled
    NeoForge(neoforge::NeoforgeVersionManifest),
    Quilt(quilt::QuiltVersionManifest),
}

pub async fn load_version_manifest(loader_contents: String) -> Result<Manifests, String> {
    // Now that we have the contents we match on version.loader to determine which type to deserialize into, and then we deserialize the contents into that type and return it wrapped in the VersionManifest enum.
    let loader_version_manifest = match version.loader {
        api_types::profiles::LoaderKind::Vanilla => {
            VersionManifest::Vanilla(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
        api_types::profiles::LoaderKind::Forge => {
            VersionManifest::Forge(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
        api_types::profiles::LoaderKind::Fabric => {
            VersionManifest::Fabric(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
        api_types::profiles::LoaderKind::IrisFabric => {
            VersionManifest::IrisFabric(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
        api_types::profiles::LoaderKind::NeoForge => {
            VersionManifest::NeoForge(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
        api_types::profiles::LoaderKind::Quilt => {
            VersionManifest::Quilt(serde_json::from_str(&loader_contents).map_err(|e| e.to_string())?)
        }
    };
    // If loader kind was vanilla we can return with None as parent:
    if version.loader == api_types::profiles::LoaderKind::Vanilla || version.minecraft_version.is_none() {
        return Ok(Manifests { version: loader_version_manifest, parent_version: None });
    }
    // Otherwise we need to load the version.minecraft_version as the parent version manifest, and return it in the Manifests struct:
    let parent_loader_contents = crate::system::fs::read_str(
        crate::system::fs::mc_dir()?
            .join("versions")
            .join(&version.minecraft_version.clone().expect("No minecraft version was found!"))
            .join(format!("{}.json", version.minecraft_version.expect("No minecraft version was found!"))),
    )
    .await?;
    let parent_version_manifest = serde_json::from_str(&parent_loader_contents).map_err(|e| e.to_string())?;
    Ok(Manifests { version: loader_version_manifest, parent_version: Some(parent_version_manifest) })
}
