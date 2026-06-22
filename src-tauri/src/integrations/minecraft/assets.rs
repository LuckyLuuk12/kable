use crate::integrations::minecraft::versions::vanilla::AssetIndex;
use crate::system::{fs, net};

// Only Vanilla has assets but all loaders inherit and should contain a parent pointing to a vanilla version so we can just use the vanilla asset index for all loaders.
pub async fn ensure_assets(index: &AssetIndex) -> Result<(), String> {
    let mc_dir = fs::mc_dir()?;
    let assets_dir = mc_dir.join(crate::constants::ASSETS_DIR);

    let index_path = assets_dir.join("indexes").join(format!("{}.json", index.id));

    net::download_to_file(&index.url, &index_path).await?;

    let json = fs::read_str(&index_path).await?;
    let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let objects = parsed["objects"].as_object().ok_or("invalid asset index")?;

    for (_name, obj) in objects {
        let hash = obj["hash"].as_str().ok_or("missing hash")?;

        let sub = &hash[0..2];

        let file_path = assets_dir.join("objects").join(sub).join(hash);

        if file_path.exists() {
            continue;
        }
        // TODO: if this url is valid move it to constants!
        let url = format!("https://resources.download.minecraft.net/{}/{}", sub, hash);

        net::download_to_file(&url, &file_path).await?;
    }

    Ok(())
}
