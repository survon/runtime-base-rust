use serde::{Deserialize, Serialize};

use crate::module::config::BaseModuleConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub file: String,
    pub duration_seconds: i32,
    pub artist: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumBindings {
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub genre: String,
    pub credits: String,
    pub tracklist: Vec<Track>,
}

/// Album module (audio collections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: AlbumBindings,
}
