use common::{
    msg::{ClientMsg, ServerMsg, Snapshot}, playback::Track,
};
use interprocess::local_socket::Stream;
use std::{ops::ControlFlow, path::PathBuf, sync::Arc, sync::mpsc::Sender};

use crate::{
    audio::AudioMsg,
    client::{ClientId, Clients},
    playback::PlaybackState,
};

#[derive(Debug)]
pub struct App {
    clients: Clients,
    audio_tx: Sender<AudioMsg>,
    playback_state: PlaybackState,
}

pub enum AppMsg {
    ClientConnected(ClientId, Arc<Stream>),
    ClientDisconnected(ClientId),
    ClientMsg(ClientId, ClientMsg),
    TrackStarted(PathBuf),
}

impl App {
    pub fn new(audio_tx: Sender<AudioMsg>) -> App {
        App {
            clients: Clients::default(),
            audio_tx,
            playback_state: PlaybackState::default(),
        }
    }

    pub fn handle(&mut self, msg: AppMsg) -> ControlFlow<()> {
        match msg {
            AppMsg::ClientConnected(id, stream) => self.clients.add(id, stream),
            AppMsg::ClientDisconnected(id) => {
                self.clients.remove(id);
            }
            AppMsg::ClientMsg(id, msg) => return self.handle_client_msg(id, msg),
            AppMsg::TrackStarted(path) => {
                self.clients.broadcast(&ServerMsg::TrackStarted(path));
            }
        }
        ControlFlow::Continue(())
    }

    fn handle_client_msg(&mut self, id: ClientId, msg: ClientMsg) -> ControlFlow<()> {
        match msg {
            ClientMsg::Handshake => {
                let snapshot = Snapshot {
                    current_track: self.playback_state.current_track().map(|t| t.clone()),
                };
                if let Err(e) = self.clients.send(id, &ServerMsg::Handshake(snapshot)) {
                    eprintln!("failed to send handshake to client {id}: {e}");
                }
            }
            ClientMsg::Shutdown => return ControlFlow::Break(()),
            ClientMsg::AddTrack(path) => {
                self.playback_state.set_current();
                // self.current_track = Some(path.clone());
                self.playback_state.add_track(Track {
                    path,
                    title: "Title".into(),
                    artist: None
                });
                if let Err(e) = self.audio_tx.send(AudioMsg::Play(path)) {
                    eprintln!("failed to send play audio msg: {e}");
                }
            }
        }
        ControlFlow::Continue(())
    }
}
