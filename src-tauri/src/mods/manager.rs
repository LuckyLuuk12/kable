use crate::installations::kable_profiles::KableInstallation;
use crate::mods::modrinth::ModrinthInfo;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait ModProvider {
    /// Set the result page size
    fn set_limit(&mut self, limit: usize);

    /// Get mods/projects at a given offset (pagination)
    async fn get(&mut self, offset: usize) -> Result<Vec<ModInfoKind>, String>;

    /// Apply filters (by installation, loader, version, etc.)
    fn filter(&mut self, installation: Option<&KableInstallation>, filter: Option<ModFilter>);

    /// Download a mod by its identifier
    async fn download(
        &self,
        mod_id: &str,
        version_id: Option<&str>,
        target_dir: &std::path::Path,
    ) -> Result<(), String>;

    fn set_index(&mut self, index: Option<String>);
    fn get_index(&self) -> Option<&String>;

    fn sort_by(&mut self, index: Option<String>) {
        self.set_index(index);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModInfoKind {
    Modrinth(ModrinthInfo),
    // CurseForge(CurseForgeInfo),
    // Add more sources as needed
}

// Generic filter type for all providers
#[derive(Debug, Clone)]
pub enum ModFilter {
    Modrinth(crate::mods::modrinth::FilterFacets),
    // CurseForge(CurseForgeFilter),
    // Add more sources as needed
}
