use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type TrackId = usize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
}
