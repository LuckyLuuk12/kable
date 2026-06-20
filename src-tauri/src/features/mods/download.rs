use crate::integrations::modrinth::client::download_project;
use crate::system::fs::{create_dir, kable_dir};
use api_types::{mods::Project, profiles::KableProfile};
use std::path::Path;

pub async fn download(profile: KableProfile, project: Project) -> Result<(), String> {
    let absolute_path = kable_dir()?.join(Path::new(&profile.dedicated_mods_folder.expect("invalid mods folder path")));
    let parent_folder = create_dir(absolute_path).await?;
    download_project(&project, Some(&profile.version.id.to_string()), parent_folder).await
}
