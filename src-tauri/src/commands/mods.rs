use crate::installations::kable_profiles::KableInstallation;
use crate::{mods::*, ModJarInfo};

#[tauri::command]
pub async fn get_mods(provider: ProviderKind, offset: usize) -> Result<Vec<ModInfoKind>, String> {
    crate::mods::get_mods(provider, offset).await
}

#[tauri::command]
pub async fn download_mod(
    provider: ProviderKind,
    mod_id: String,
    version_id: Option<String>,
    installation: KableInstallation,
) -> Result<(), String> {
    crate::mods::download_mod(provider, &mod_id, version_id.as_deref(), &installation).await
}

#[tauri::command]
pub async fn set_provider_filter(
    provider: ProviderKind,
    installation: Option<KableInstallation>,
    filter: Option<ModFilter>,
) {
    crate::mods::set_provider_filter(provider, installation.as_ref(), filter);
}

#[tauri::command]
pub async fn set_provider_limit(provider: ProviderKind, limit: usize) {
    crate::mods::set_provider_limit(provider, limit);
}

#[tauri::command]
pub async fn clear_provider_cache(provider: ProviderKind) {
    crate::mods::clear_provider_cache(provider);
}

#[tauri::command]
pub async fn purge_stale_provider_cache(provider: ProviderKind) {
    crate::mods::purge_stale_provider_cache(provider);
}

#[tauri::command]
pub async fn get_extended_mod_info(mod_jar_info: ModJarInfo) -> Result<ExtendedModInfo, String> {
    crate::mods::get_extended_mod_info(mod_jar_info).await
}
