use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, facet::Facet)]
pub struct CustomIconTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub svg_data: String,
    pub preview_svg: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, facet::Facet)]
pub struct IconSettings {
    pub custom_templates: Vec<CustomIconTemplate>,
}
