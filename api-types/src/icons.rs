use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet, specta::Type)]
pub struct CustomIconTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub svg_data: String,
    pub preview_svg: Option<String>,
    pub created_at: Option<i32>,
    pub updated_at: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet, specta::Type)]
pub struct IconSettings {
    pub custom_templates: Vec<CustomIconTemplate>,
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
