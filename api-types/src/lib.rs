pub mod auth;
pub mod discord;
pub mod icons;
pub mod launcher;
pub mod logging;
pub mod maps;
pub mod mojang;
pub mod mrpack;
pub mod profiles;
pub mod projects;
pub mod resourcepacks;
pub mod settings;
pub mod shaders;
pub mod skins;
pub mod sounds;
pub mod symlinks;
pub mod updater;
pub mod worlds;

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, facet::Facet)]
pub struct Timestamp(pub DateTime<Utc>);

impl specta::Type for Timestamp {
    fn definition(_types: &mut specta::Types) -> specta::datatype::DataType {
        specta::datatype::DataType::Primitive(specta::datatype::Primitive::str)
    }
}
