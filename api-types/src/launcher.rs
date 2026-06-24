use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, facet::Facet, specta::Type)]
pub struct LaunchResult {
    pub pid: u32,
    pub runtime_id: String,
    pub command: String,
}
