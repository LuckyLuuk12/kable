use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldSummary {
    pub id: String,
    pub name: String,
    pub path: String,

    pub icon: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct World {
    pub id: String,
    pub name: String,
    pub path: String,

    pub icon: Option<String>,
    pub last_modified: Option<String>,

    pub level: Option<WorldLevel>,
    pub players: Vec<WorldPlayer>,
    pub dimensions: Vec<WorldDimension>,
    pub datapacks: Vec<WorldDatapack>,

    pub region_storage: Vec<WorldRegionStorage>,

    pub advancements: Vec<WorldAdvancement>,
    pub statistics: Vec<WorldPlayerStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldLevel {
    pub data_version: Option<i32>,
    pub version: Option<WorldVersion>,
    pub level_name: Option<String>,

    pub game_mode: Option<WorldGameMode>,
    pub difficulty: Option<WorldDifficulty>,

    pub hardcore: Option<bool>,
    pub allow_commands: Option<bool>,

    pub seed: Option<String>,

    pub time: Option<WorldTime>,
    pub spawn: Option<WorldSpawn>,
    pub weather: Option<WorldWeather>,
    pub world_border: Option<WorldBorder>,

    pub game_rules: Vec<WorldGameRule>,

    pub nbt: Option<NbtValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldVersion {
    pub name: Option<String>,
    pub id: Option<i32>,
    pub snapshot: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum WorldGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum WorldDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldTime {
    pub game_time: Option<String>,
    pub day_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldSpawn {
    pub x: i32,
    pub y: i32,
    pub z: i32,

    pub angle: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldWeather {
    pub raining: Option<bool>,
    pub rain_time: Option<i32>,

    pub thundering: Option<bool>,
    pub thunder_time: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldBorder {
    pub center_x: Option<f64>,
    pub center_z: Option<f64>,

    pub size: Option<f64>,

    pub damage_per_block: Option<f64>,
    pub safe_zone: Option<f64>,

    pub warning_blocks: Option<i32>,
    pub warning_time: Option<i32>,

    pub lerp_target: Option<f64>,
    pub lerp_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldGameRule {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldPlayer {
    pub uuid: String,
    pub path: String,

    pub name: Option<String>,
    pub dimension: Option<String>,

    pub position: Option<WorldPosition>,
    pub rotation: Option<WorldRotation>,

    pub health: Option<f32>,
    pub food_level: Option<i32>,
    pub food_saturation: Option<f32>,

    pub experience_level: Option<i32>,
    pub experience: Option<f32>,
    pub total_experience: Option<i32>,

    pub game_mode: Option<WorldGameMode>,

    pub inventory: Vec<WorldInventoryItem>,

    pub ender_items: Option<NbtValue>,

    pub nbt: Option<NbtValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldRotation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldInventoryItem {
    pub slot: i8,

    pub item_id: String,

    pub count: i32,

    pub nbt: Option<NbtValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldDimension {
    pub id: String,

    pub kind: WorldDimensionKind,

    pub path: String,

    pub region_storage: Option<WorldRegionStorage>,

    pub chunk_count: String,
    pub region_count: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum WorldDimensionKind {
    Overworld,
    Nether,
    End,
    Custom,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldDatapack {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,

    pub pack_format: Option<i32>,
    pub min_format: Option<i32>,
    pub max_format: Option<i32>,

    pub enabled: bool,

    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldRegionStorage {
    pub path: String,

    pub region_count: String,
    pub chunk_count: String,
    pub total_size: String,

    pub regions: Vec<WorldRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldRegion {
    pub x: i32,
    pub z: i32,

    pub path: String,

    pub size: String,
    pub chunk_count: i32,

    pub chunks: Vec<WorldChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldChunk {
    pub x: i32,
    pub z: i32,

    pub region_x: i32,
    pub region_z: i32,

    pub sector_offset: i32,
    pub sector_count: i32,

    pub timestamp: Option<String>,

    pub compression: Option<WorldChunkCompression>,

    pub compressed_size: Option<i32>,

    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum WorldChunkCompression {
    Gzip,
    Zlib,
    Uncompressed,
    Lz4,
    Custom(u8),
}

impl WorldChunkCompression {
    pub fn from_id(id: u8) -> Result<Self, String> {
        match id {
            1 => Ok(Self::Gzip),
            2 => Ok(Self::Zlib),
            3 => Ok(Self::Uncompressed),
            4 => Ok(Self::Lz4),
            id => Ok(Self::Custom(id)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldAdvancement {
    pub id: String,

    pub player_uuid: String,

    pub done: bool,

    pub criteria: Vec<WorldAdvancementProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldAdvancementProgress {
    pub id: String,

    pub achieved_at: Option<String>,

    pub raw: Option<NbtValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldPlayerStatistics {
    pub uuid: String,

    pub path: String,

    pub statistics: Vec<WorldStatistic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WorldStatistic {
    pub category: String,

    pub name: String,

    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value")]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(String),

    Float(f32),
    Double(f64),

    ByteArray(Vec<i8>),

    String(String),

    List(Vec<NbtValue>),

    Compound(Vec<NbtEntry>),

    IntArray(Vec<i32>),

    LongArray(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NbtEntry {
    pub name: String,
    pub value: NbtValue,
}
