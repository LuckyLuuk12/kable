use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::hash::Hash;
use strum::{IntoEnumIterator, EnumDiscriminants};
use once_cell::sync::OnceCell;

//*
//* Private API:
//*

// #region Loader Manifests
lazy_static::lazy_static! {
    static ref LOADER_MANIFEST_URLS: HashMap<ManifestKindDiscriminants, &'static str> = {
        let mut m = HashMap::new();
        m.insert(ManifestKindDiscriminants::Vanilla, "https://launchermeta.mojang.com/mc/game/version_manifest.json");
        m.insert(ManifestKindDiscriminants::Fabric, "https://meta.fabricmc.net/v2/versions/loader");
        m.insert(ManifestKindDiscriminants::IrisFabric, "https://meta.fabricmc.net/v2/versions/loader"); // Iris uses Fabric manifest
        m.insert(ManifestKindDiscriminants::Forge, "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json");
        m.insert(ManifestKindDiscriminants::NeoForge, "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge");
        m.insert(ManifestKindDiscriminants::Quilt, "https://meta.quiltmc.org/v3/versions/loader");
        m
    };
}
// #endregion

// #region - Vanilla Manifest
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct VanillaLatest {
    release: String,
    snapshot: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum VanillaType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct VanillaVersion {
    id: String,
    type_: VanillaType,
    url: String,
    time: String,
    release_time: String,
    sha1: String,
    compliance_level: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct VanillaManifest {
  latest: VanillaLatest,
  versions: Vec<VanillaVersion>,
}
// #endregion

// #region Fabric Manifest
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct FabricVersion {
    separator: String,
    build: u32,
    maven: String,
    version: String,
    stable: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct FabricManifest {
    versions: Vec<FabricVersion>,
}
// #endregion

// #region Iris Fabric Manifest - does not have its own manifest, uses Fabric
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct IrisFabricVersion {
    separator: String,
    build: u32,
    maven: String,
    version: String,
    stable: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct IrisFabricManifest {
    versions: Vec<IrisFabricVersion>,
}
// #endregion

// #region Forge Manifest
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct ForgeManifest(IndexMap<String, Vec<String>>);
// #endregion

// #region NeoForge Manifest
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct NeoForgeManifest {
    is_snapshot: bool,
    versions: Vec<String>,
}
// #endregion

// #region Quilt Manifest
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct QuiltVersion {
    separator: String,
    build: u32,
    maven: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
struct QuiltManifest {
    versions: Vec<QuiltVersion>,
}
// #endregion

// #region Combined Manifest types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VersionData {
    /// Unique identifier for this version entry (for internal use, e.g. `.minecraft/versions` directory)
    pub version_id: String,
    /// The id field from the manifest, if present (e.g. Vanilla)
    pub id: Option<String>,
    // Common fields
    pub loader: ManifestKindDiscriminants, // Use the discriminant to identify the loader type
    pub stable: Option<bool>,
    // Vanilla
    #[serde(rename = "type")]
    pub type_: Option<VanillaType>,
    pub url: Option<String>,
    pub time: Option<String>,
    pub release_time: Option<String>,
    pub sha1: Option<String>,
    pub compliance_level: Option<u32>,
    // Fabric/Iris/Quilt
    pub separator: Option<String>,
    pub build: Option<u32>,
    pub maven: Option<String>,
    pub version: Option<String>,
    // NeoForge
    pub is_snapshot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Versions(pub Vec<VersionData>);
impl Versions {
    pub fn new() -> Self {
        Versions(Vec::new())
    }

    pub fn get_version(&self, version_id: &str) -> Option<&VersionData> {
        self.0.iter().find(|v| v.version_id == version_id)
    }

    pub fn extend<I: IntoIterator<Item = VersionData>>(&mut self, iter: I) {
        self.0.extend(iter);
    }

    pub fn iter(&self) -> impl Iterator<Item = &VersionData> {
        self.0.iter()
    }
}

/// Default implementation for VersionData used in the `From` implementations with None defaults
impl Default for VersionData {
    fn default() -> Self {
        VersionData {
            version_id: String::new(),
            id: None,
            loader: ManifestKindDiscriminants::Vanilla, // Default to Vanilla
            stable: None,
            type_: None,
            url: None,
            time: None,
            release_time: None,
            sha1: None,
            compliance_level: None,
            separator: None,
            build: None,
            maven: None,
            version: None,
            is_snapshot: None,
        }
    }
}
// #endregion

// #region Convertion traits
#[derive(EnumDiscriminants, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[strum_discriminants(derive(Hash, Serialize, Deserialize))]
enum ManifestKind {
    Vanilla(VanillaManifest),
    Fabric(FabricManifest),
    IrisFabric(IrisFabricManifest),
    Forge(ForgeManifest),
    NeoForge(NeoForgeManifest),
    Quilt(QuiltManifest),
}
/// Discriminants for ManifestKind, used for matching and serialization
impl ManifestKind {
    /// Uses strum to cast to corresponding Discriminant and get the manifest URL from the static map
    fn manifest_url(&self) -> Result<&'static str, &'static str> {
        LOADER_MANIFEST_URLS.get(self.into())
            .cloned()
            .ok_or("Manifest URL not found")
    }
    /// Uses the manifest URL and serde to deserialize the manifest into the corresponding struct
    fn load_manifest(&self) -> Result<ManifestKind, &'static str> {
        let url = self.manifest_url()?;
        let response = reqwest::blocking::get(url).map_err(|_| "Failed to fetch manifest")?;
        // #!?$ DRY I guess... :(
        let manifest = match self.into() {
            ManifestKindDiscriminants::Vanilla => response.json::<VanillaManifest>().map(ManifestKind::Vanilla),
            ManifestKindDiscriminants::Fabric => response.json::<FabricManifest>().map(ManifestKind::Fabric),
            ManifestKindDiscriminants::IrisFabric => response.json::<IrisFabricManifest>().map(ManifestKind::IrisFabric),
            ManifestKindDiscriminants::Forge => response.json::<ForgeManifest>().map(ManifestKind::Forge),
            ManifestKindDiscriminants::NeoForge => response.json::<NeoForgeManifest>().map(ManifestKind::NeoForge),
            ManifestKindDiscriminants::Quilt => response.json::<QuiltManifest>().map(ManifestKind::Quilt),
            //! NOTE: because of Reflection and macro lacks we cannot dynamically deserialize into the enum variant 
        };
        manifest.map_err(|_| "Failed to deserialize manifest")
    }

    fn to_versions(&self, vanilla: &VanillaManifest) -> Versions {
        match self {
            ManifestKind::Vanilla(manifest) => {
                manifest.versions().into()
            },
            ManifestKind::Fabric(manifest) | ManifestKind::IrisFabric(manifest) | ManifestKind::Forge(manifest) | ManifestKind::NeoForge(manifest) | ManifestKind::Quilt(manifest) => {
                manifest.into_version(vanilla)
            }
        }
    }
}
/// Trait to convert a manifest into a Versions collection
trait FromManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions;
}
/// Takes all available keys from the FabricManifest, loops over vanilla versions and creates fabric-loader-<fabric_version>-<vanilla_version> entries
impl FromManifest for FabricManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions {
        self.versions.iter().flat_map(|v| {
            vanilla.versions().iter().map(move |vv| VersionData {
                version_id: format!("fabric-loader-{}-{}", v.version, vv.version),
                id: None,
                loader: self.into(),
                stable: Some(v.stable),
                separator: Some(v.separator.clone()),
                build: Some(v.build),
                maven: Some(v.maven.clone()),
                version: Some(v.version.clone()),
                ..Default::default()
            })
        }).collect()
    }
}
/// Takes all available keys from the IrisFabricManifest, loops over vanilla versions and creates iris-loader-<iris_version>-<vanilla_version> entries
impl FromManifest for IrisFabricManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions {
        self.versions.iter().flat_map(|v| {
            vanilla.versions().iter().map(move |vv| VersionData {
                version_id: format!("iris-loader-{}-{}", v.version, vv.version),
                loader: self.into(),
                stable: Some(v.stable),
                separator: Some(v.separator.clone()),
                build: Some(v.build),
                maven: Some(v.maven.clone()),
                version: Some(v.version.clone()),
                ..Default::default()
            })
        }).collect()
    }
}
/// Takes the forge manifest in which a list of minecraft versions has a list of forge versions, then version_id = <minecraft_version>-forge-<forge_version>
impl FromManifest for ForgeManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions {
        self.0.iter().flat_map(|(minecraft_version, forge_versions)| {
            vanilla.versions().iter().filter_map(move |vv| {
                if vv.id == *minecraft_version {
                    forge_versions.iter().map(move |forge_version| VersionData {
                        version_id: format!("{}-forge-{}", minecraft_version, forge_version),
                        loader: self.into(),
                        ..Default::default()
                    }).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            })
        }).collect()
    }
}
/// Takes the NeoForge manifest and creates version_id = <minecraft_version>-neoforge-<neoforge_version>
impl FromManifest for NeoForgeManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions {
        self.versions.iter().flat_map(|neoforge_version| {
            vanilla.versions().iter().map(move |vv| VersionData {
                version_id: format!("{}-neoforge-{}", vv.id, neoforge_version),
                loader: self.into(),
                is_snapshot: Some(self.is_snapshot),
                version: Some(neoforge_version.clone()),
                ..Default::default()
            })
        }).collect()
    }
}
/// Takes all available keys from the QuiltManifest, loops over vanilla versions and creates quilt-loader-<quilt_version>-<vanilla_version> entries
impl FromManifest for QuiltManifest {
    fn into_version(&self, vanilla: &VanillaManifest) -> Versions {
        self.versions.iter().flat_map(|quilt_version| {
            vanilla.versions().iter().map(move |vv| VersionData {
                version_id: format!("{}-quilt-{}", vv.id, quilt_version.version),
                loader: self.into(),
                separator: Some(quilt_version.separator.clone()),
                build: Some(quilt_version.build),
                maven: Some(quilt_version.maven.clone()),
                version: Some(quilt_version.version.clone()),
                ..Default::default()
            })
        }).collect()
    }
}
// #endregion

// #region Build versions
macro_rules! add_versions {
    ($manifest:ty, $vanilla:ident, $versions:ident) => {
        $versions.extend(<$manifest>::default().into_version(&$vanilla));
    };
}
static VERSIONS_CACHE: OnceCell<Versions> = OnceCell::new();
pub fn build_versions() -> Versions {
    let vanilla = VanillaManifest::default();
    let mut versions = Versions::new();
    add_versions!(FabricManifest, vanilla, versions);
    add_versions!(IrisFabricManifest, vanilla, versions);
    add_versions!(ForgeManifest, vanilla, versions);
    add_versions!(NeoForgeManifest, vanilla, versions);
    add_versions!(QuiltManifest, vanilla, versions);
    versions
}
// #endregion
