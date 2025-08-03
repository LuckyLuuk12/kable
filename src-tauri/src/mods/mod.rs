pub mod cache;
// pub mod curseforge;
pub mod manager;
pub mod modrinth;

pub use self::cache::*;
// pub use self::curseforge::*;
pub use self::manager::*;
pub use self::modrinth::*;

use crate::installations::kable_profiles::KableInstallation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Modrinth,
    // CurseForge, // TODO
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

static MODRINTH: Lazy<Mutex<ModrinthProvider>> =
    Lazy::new(|| Mutex::new(ModrinthProvider::default()));

pub fn set_provider_filter(
    provider: ProviderKind,
    installation: Option<&KableInstallation>,
    filter: Option<ModFilter>,
) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().filter(installation, filter);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub fn set_provider_limit(provider: ProviderKind, limit: usize) {
    match provider {
        ProviderKind::Modrinth => {
            MODRINTH.lock().unwrap().set_limit(limit);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub async fn get_mods(provider: ProviderKind, offset: usize) -> Result<Vec<ModInfoKind>, String> {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.get(offset).await
        }
        //ProviderKind::CurseForge => todo!(),
    }
}

pub async fn download_mod(
    provider: ProviderKind,
    mod_id: &str,
    version_id: Option<&str>,
    target_dir: &std::path::Path,
) -> Result<(), String> {
    match provider {
        ProviderKind::Modrinth => {
            let prov = {
                let prov_guard = MODRINTH.lock().unwrap();
                prov_guard.clone()
            };
            prov.download(mod_id, version_id, target_dir).await
        }
        //ProviderKind::CurseForge => todo!(),
    }
}

pub fn clear_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.clear();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}

pub fn purge_stale_provider_cache(provider: ProviderKind) {
    match provider {
        ProviderKind::Modrinth => {
            let mut prov = MODRINTH.lock().unwrap();
            prov.cache.purge_stale();
            let _ = prov.cache.save_to_disk(&prov.cache_path);
        } //ProviderKind::CurseForge => { /* TODO */ }
    }
}
