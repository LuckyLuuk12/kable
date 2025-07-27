use serde::{Serialize, Deserialize};
use strum::{EnumIter, IntoEnumIterator};
use serde_json::json;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum LoaderKind {
    Vanilla,
    Fabric,
    IrisFabric,
    Forge,
    NeoForge,
    Quilt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VersionData {
    pub version_id: String,
    pub loader: LoaderKind,
    pub display_name: String,
    pub is_stable: bool,
    pub extra: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Versions(pub Vec<VersionData>);

impl Versions {
    pub fn get_version(&self, version_id: &str) -> Option<&VersionData> {
        self.0.iter().find(|v| v.version_id == version_id)
    }
    pub fn extend<I: IntoIterator<Item = VersionData>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
    pub fn iter(&self) -> impl Iterator<Item = &VersionData> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for Versions {
    type Item = VersionData;
    type IntoIter = std::vec::IntoIter<VersionData>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a> IntoIterator for &'a Versions {
    type Item = &'a VersionData;
    type IntoIter = std::slice::Iter<'a, VersionData>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
impl<'a> IntoIterator for &'a mut Versions {
    type Item = &'a mut VersionData;
    type IntoIter = std::slice::IterMut<'a, VersionData>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter_mut() }
}
impl std::iter::FromIterator<VersionData> for Versions {
    fn from_iter<I: IntoIterator<Item = VersionData>>(iter: I) -> Self {
        Versions(iter.into_iter().collect())
    }
}

pub trait Loader: Send + Sync {
    fn kind(&self) -> LoaderKind;
    /// If the loader needs vanilla versions, they are provided as Some(&[VersionData]), otherwise None.
    fn get_versions<'a>(&'a self, vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>>;
    fn download<'a>(&'a self, version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

// --- Loader Implementations ---

// Vanilla Loader
pub struct VanillaLoader;
impl Loader for VanillaLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::Vanilla }
    fn get_versions<'a>(&'a self, _vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let versions_val = &resp["versions"];
            let versions: &[serde_json::Value] = match versions_val.as_array() {
                Some(arr) => arr,
                None => &[],
            };
            let out = versions.iter().map(|v| {
                let id_val = &v["id"];
                let id = id_val.as_str().unwrap_or("");
                let typ_val = &v["type"];
                let typ = typ_val.as_str().unwrap_or("");
                let is_stable = typ == "release";
                VersionData {
                    version_id: id.to_string(),
                    loader: LoaderKind::Vanilla,
                    display_name: id.to_string(),
                    is_stable,
                    extra: v.clone(),
                }
            }).collect();
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Fabric Loader
pub struct FabricLoader;
impl Loader for FabricLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::Fabric }
    fn get_versions<'a>(&'a self, vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://meta.fabricmc.net/v2/versions/loader";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let fabricv = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
                    let stable = v.get("stable").and_then(|x| x.as_bool()).unwrap_or(false);
                    for mcv in vanilla {
                        out.push(VersionData {
                            version_id: format!("fabric-loader-{}-{}", fabricv, mcv.version_id),
                            loader: LoaderKind::Fabric,
                            display_name: format!("Fabric {} for MC {}", fabricv, mcv.version_id),
                            is_stable: stable,
                            extra: v.clone(),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// IrisFabric Loader (uses Fabric manifest)
pub struct IrisFabricLoader;
impl Loader for IrisFabricLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::IrisFabric }
    fn get_versions<'a>(&'a self, vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://meta.fabricmc.net/v2/versions/loader";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let fabricv = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
                    let stable = v.get("stable").and_then(|x| x.as_bool()).unwrap_or(false);
                    for mcv in vanilla {
                        out.push(VersionData {
                            version_id: format!("iris-fabric-loader-{}-{}", fabricv, mcv.version_id),
                            loader: LoaderKind::IrisFabric,
                            display_name: format!("Iris Fabric {} for MC {}", fabricv, mcv.version_id),
                            is_stable: stable,
                            extra: v.clone(),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Forge Loader
pub struct ForgeLoader;
impl Loader for ForgeLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::Forge }
    fn get_versions<'a>(&'a self, _vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let binding = serde_json::Map::new();
            let obj = resp.as_object().unwrap_or(&binding);
            let mut out = Vec::new();
            for (mc_version, forge_versions) in obj {
                if let Some(forge_arr) = forge_versions.as_array() {
                    for forge_version in forge_arr {
                        let forge_version_str = forge_version.as_str().unwrap_or("");
                        // Extract the part after the first dash for display purposes
                        let forge_version_display = match forge_version_str.find('-') {
                            Some(idx) if idx + 1 < forge_version_str.len() => &forge_version_str[idx + 1..],
                            _ => forge_version_str,
                        };
                        // Forge version_id format: forge-{mc_version}-{forge_version}
                        let version_id = format!("{}-forge-{}", mc_version, forge_version_display);
                        let display_name = format!("Forge {} for MC {}", forge_version_display, mc_version);
                        out.push(VersionData {
                            version_id,
                            loader: LoaderKind::Forge,
                            display_name,
                            is_stable: true,
                            extra: json!({ "forge_version": forge_version_display, "minecraft_version": mc_version }),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// NeoForge Loader
pub struct NeoForgeLoader;
impl Loader for NeoForgeLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::NeoForge }
    fn get_versions<'a>(&'a self, _vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let arr_val = &resp["versions"];
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let is_snapshot = resp["is_snapshot"].as_bool().unwrap_or(false);
            let mut out = Vec::new();
            for v in arr {
                let neoforge_ver = v.as_str().unwrap_or("");
                out.push(VersionData {
                    version_id: format!("neoforge-{}", neoforge_ver),
                    loader: LoaderKind::NeoForge,
                    display_name: format!("NeoForge {}", neoforge_ver),
                    is_stable: !is_snapshot,
                    extra: json!({ "neoforge_version": neoforge_ver, "is_snapshot": is_snapshot }),
                });
            }
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Quilt Loader
pub struct QuiltLoader;
impl Loader for QuiltLoader {
    fn kind(&self) -> LoaderKind { LoaderKind::Quilt }
    fn get_versions<'a>(&'a self, vanilla_versions: Option<&'a [VersionData]>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionData>>> + Send + 'a>> {
        Box::pin(async move {
            let url = "https://meta.quiltmc.org/v3/versions/loader";
            let resp = reqwest::get(url).await?.json::<serde_json::Value>().await?;
            let arr_val = &resp;
            let arr: &[serde_json::Value] = match arr_val.as_array() {
                Some(a) => a,
                None => &[],
            };
            let mut out = Vec::new();
            if let Some(vanilla) = vanilla_versions {
                for v in arr {
                    let quilt_ver = v["version"].as_str().unwrap_or("");
                    let stable = v["stable"].as_bool().unwrap_or(false);
                    for mcv in vanilla {
                        out.push(VersionData {
                            version_id: format!("quilt-loader-{}-{}", quilt_ver, mcv.version_id),
                            loader: LoaderKind::Quilt,
                            display_name: format!("Quilt {} for MC {}", quilt_ver, mcv.version_id),
                            is_stable: stable,
                            extra: json!({ "quilt_version": quilt_ver, "minecraft_version": mcv.version_id, "stable": stable }),
                        });
                    }
                }
            }
            Ok(out)
        })
    }
    fn download<'a>(&'a self, _version_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

// Loader factory
pub fn loader_for_kind(kind: LoaderKind) -> Box<dyn Loader> {
    match kind {
        LoaderKind::Vanilla => Box::new(VanillaLoader),
        LoaderKind::Fabric => Box::new(FabricLoader),
        LoaderKind::IrisFabric => Box::new(IrisFabricLoader),
        LoaderKind::Forge => Box::new(ForgeLoader),
        LoaderKind::NeoForge => Box::new(NeoForgeLoader),
        LoaderKind::Quilt => Box::new(QuiltLoader),
    }
}

// Build all versions for all loaders (async)
pub async fn build_versions() -> Versions {
    let mut versions = Versions::default();
    // First, get vanilla versions (needed for some loaders)
    let vanilla_loader = VanillaLoader;
    let vanilla_versions = vanilla_loader.get_versions(None).await.unwrap_or_default();
    for kind in LoaderKind::iter() {
        let loader = loader_for_kind(kind);
        let vers = if kind == LoaderKind::Vanilla {
            loader.get_versions(None).await
        } else {
            loader.get_versions(Some(&vanilla_versions)).await
        };
        if let Ok(vers) = vers {
            versions.extend(vers);
        }
    }
    versions
}
