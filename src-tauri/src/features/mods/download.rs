// Placeholder: mod download logic

use crate::system::fs::{ensure_parent_dir_exists_async, get_minecraft_kable_dir};
use crate::{ensure_folder, integrations::modrinth::client::download_project};
use api_types::{mods::Project, profiles::KableProfile};

pub async fn download(profile: KableProfile, project: Project) -> Result<(), String> {
    let absolute_path = get_minecraft_kable_dir().join(profile.dedicated_mods_folder);
    let parent_folder = ensure_parent_dir_exists_async(absolute_path).await?;
    download_project(&project, Some(&profile.version.id.to_string()), parent_folder).await
}
