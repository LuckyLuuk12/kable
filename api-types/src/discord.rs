use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ActivityPriority {
    Idle,
    Browsing,
    Managing,
    Playing,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct PresenceState {
//     pub enabled: bool,
//     pub priority: ActivityPriority,
//     pub details: Option<String>,
// }

#[derive(Debug, Clone, Serialize, Deserialize, facet::Facet)]
pub struct PresenceState {
    pub state: String,
    pub details: String,
    pub priority: ActivityPriority,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub start_timestamp: Option<i64>,
}
