use std::{io::Write, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMsg {
    Stop,
    Play(PathBuf),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMsg {
    TrackStarted(PathBuf),
}

pub fn write_frame<M: Serialize>(mut w: impl Write, msg: &M) -> eyre::Result<()> {
    let body = postcard::to_allocvec(msg)?;
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(&body);
    w.write_all(&frame)?;
    Ok(())
}
