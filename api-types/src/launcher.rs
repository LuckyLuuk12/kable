use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct LaunchResult {
    pub pid: u32,
    pub command: String,
}
