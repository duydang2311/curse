use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMsg {
    Stop,
    Play(PathBuf),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMsg {}
