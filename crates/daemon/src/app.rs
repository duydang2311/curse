use std::{ops::ControlFlow, path::PathBuf, sync::Arc, sync::mpsc::Sender};

use common::msg::{ClientMsg, ServerMsg, Snapshot};
use interprocess::local_socket::Stream;

use crate::{
    audio::AudioMsg,
    client::{ClientId, Clients},
};

#[derive(Debug)]
pub struct App {
    clients: Clients,
    audio_tx: Sender<AudioMsg>,
    current_track: Option<PathBuf>,
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
            current_track: None,
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
                    current_track: self.current_track.clone(),
                };
                if let Err(e) = self.clients.send(id, &ServerMsg::Handshake(snapshot)) {
                    eprintln!("failed to send handshake to client {id}: {e}");
                }
            }
            ClientMsg::Shutdown => return ControlFlow::Break(()),
            ClientMsg::Play(path) => {
                self.current_track = Some(path.clone());
                if let Err(e) = self.audio_tx.send(AudioMsg::Play(path)) {
                    eprintln!("failed to send play audio msg: {e}");
                }
            }
        }
        ControlFlow::Continue(())
    }
}
