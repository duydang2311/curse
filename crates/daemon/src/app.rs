use std::sync::mpsc::Sender;

use crate::audio::AudioMsg;

#[derive(Clone, Debug)]
pub struct App {
    pub tx: Sender<AppMsg>,
    pub audio_tx: Sender<AudioMsg>,
}

pub enum AppMsg {
    Shutdown,
}

impl App {
    pub fn new(tx: Sender<AppMsg>, audio_tx: Sender<AudioMsg>) -> App {
        App { tx, audio_tx }
    }
}
