use crate::playback::{Track, TrackId};
use eyre::{Context};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Handshake,
    Shutdown,
    Play(TrackId),
    AddTrack(PathBuf)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    Handshake(Snapshot),
    TrackStarted(PathBuf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub current_track: Option<Track>,
}

pub fn write_frame<M: Serialize>(mut w: impl Write, msg: &M) -> eyre::Result<()> {
    let body = postcard::to_allocvec(msg)?;
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(&body);
    w.write_all(&frame)?;
    Ok(())
}

pub fn read_frame<M: DeserializeOwned>(mut s: impl Read) -> eyre::Result<M> {
    let mut len_buf = [0u8; 4];
    s.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    s.read_exact(&mut buf)?;
    postcard::from_bytes::<M>(&buf).wrap_err("failed to deserialize frame")
}
