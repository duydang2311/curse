use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::Sender},
};

use crate::{audio::AudioMsg, client::Clients};

#[derive(Debug, Clone)]
pub struct App {
    pub tx: Sender<AppMsg>,
    pub clients: Clients,
    pub audio_tx: Sender<AudioMsg>,
    state: Arc<Mutex<AppState>>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    current_track: Option<PathBuf>,
}

pub enum AppMsg {
    Shutdown,
}

impl App {
    pub fn new(tx: Sender<AppMsg>, audio_tx: Sender<AudioMsg>) -> App {
        App {
            tx,
            audio_tx,
            clients: Clients::default(),
            state: Arc::new(Mutex::new(AppState {
                current_track: None,
            })),
        }
    }

    pub fn set_current_track(&self, path: PathBuf) {
        self.state.lock().unwrap().current_track = Some(path);
    }

    pub fn current_track(&self) -> Option<PathBuf> {
        self.state.lock().unwrap().current_track.clone()
    }
}
