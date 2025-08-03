use crate::installations::kable_profiles::KableInstallation;
use crate::mods::*;

#[tauri::command]
pub async fn get_mods_command(
    provider: ProviderKind,
    offset: usize,
) -> Result<Vec<ModInfoKind>, String> {
    get_mods(provider, offset).await
}

#[tauri::command]
pub async fn download_mod_command(
    provider: ProviderKind,
    mod_id: String,
    version_id: Option<String>,
    installation: KableInstallation,
) -> Result<(), String> {
    download_mod(provider, &mod_id, version_id.as_deref(), &installation).await
}

#[tauri::command]
pub async fn set_provider_filter_command(
    provider: ProviderKind,
    installation: Option<KableInstallation>,
    filter: Option<ModFilter>,
) {
    set_provider_filter(provider, installation.as_ref(), filter);
}

#[tauri::command]
pub async fn set_provider_limit_command(provider: ProviderKind, limit: usize) {
    set_provider_limit(provider, limit);
}

#[tauri::command]
pub async fn clear_provider_cache_command(provider: ProviderKind) {
    clear_provider_cache(provider);
}

#[tauri::command]
pub async fn purge_stale_provider_cache_command(provider: ProviderKind) {
    purge_stale_provider_cache(provider);
}
