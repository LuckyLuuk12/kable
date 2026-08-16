// use std::path::PathBuf;

use api_types::profiles::KableProfile;

/// converts a MRPack zip file into a KableProfile assuming the string passed is a valid path to a MRPack zip file, and returns an error if it is not
pub async fn into(_mrpack: String) -> Result<KableProfile, String> {
    // TODO: Implement this...
    Err("Not implemented yet".to_string())

    // let mrpack_path = std::path::PathBuf::from(mrpack);
    // if !mrpack_path.exists() {
    //     return Err(format!("MRPack file does not exist: {}", mrpack));
    // }

    // let mrpack_data = crate::features::modpack::load_mrpack(&mrpack_path).await?;
    // let mut profile = crate::features::profiles::create::create_profile_from_mrpack(&mrpack_data).await?;

    // // Set the profile name to the MRPack name
    // profile.metadata.name = mrpack_data.name.clone();

    // Ok(profile)
}

/*
A mrpack contains, as described here: https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack
- modrinth.index.json (required): contains all info we will definitely need to recreate the profile, including version id, loader, mods, resource packs, shaders, etc.
- overrides/ (optional): can contain things like config folder, options.txt which would usually be copied into the .minecraft folder
- client-overrides/ (optional): in case overrides wasn't enough I guess?
- server-overrides/ (optional): we ignore this as our launcher only support client-side launching (YET) server mangement might become a thing in the future, but for now we ignore this
*/
