use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct IconTemplate {
    pub id: String,
    pub name: String,
    pub r#type: IconTemplateType,
    pub default_icon_type: IconType,
    pub description: Option<String>,
    pub author: Vec<String>,
    pub version: Option<String>,
    pub fallback_icon: String,
    pub icons: HashMap<String, IconData>,
    pub preview_svg: Option<String>,
    pub created_at: Option<i32>,
    pub updated_at: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(untagged, rename_all = "snake_case")] // ! NOTE: untagged means we CAN NOT cache this type...
#[repr(u8)]
pub enum IconData {
    Full { icon: String, r#type: IconType },
    Legacy(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum IconTemplateType {
    Builtin,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum IconType {
    Emoji,
    Svg,
    CssClass,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet, specta::Type)]
pub struct IconSettings {
    pub custom_templates: Vec<IconTemplate>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CustomIconTemplate {
//     pub name: String,
//     pub description: Option<String>,
//     pub author: Option<String>,
//     pub svg_path: String,
//     pub view_box: Option<String>,
//     pub colors: Option<std::collections::HashMap<String, String>>,
// }
