use crate::elm;
use common::{
    get_socket_name,
    msg::{ClientMsg, ServerMsg, read_frame, write_frame},
};
use eyre::eyre;
use interprocess::local_socket::{Stream, prelude::*};
use serde::Serialize;
use std::{
    io::Read,
    path::PathBuf,
    sync::{
        Arc, Mutex, PoisonError,
        mpsc::{SendError, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Debug, Clone, Default)]
pub struct Daemon {
    stream: Arc<Mutex<Option<Arc<Stream>>>>,
}

impl Daemon {
    pub fn play(&self, path: PathBuf) -> eyre::Result<()> {
        self.send(&ClientMsg::Play(path))
    }

    fn send<M: Serialize>(&self, msg: &M) -> eyre::Result<()> {
        let stream = self
            .stream
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .ok_or_else(|| eyre!("not connected to daemon"))?;
        write_frame(&*stream, msg)
    }

    fn set_stream(&self, stream: Option<Arc<Stream>>) {
        *self.stream.lock().unwrap_or_else(PoisonError::into_inner) = stream;
    }
}

fn connect_daemon() -> eyre::Result<Stream> {
    Ok(Stream::connect(get_socket_name()?)?)
}

pub fn spawn_daemon_thread(
    daemon: Daemon,
    tx: Sender<elm::Message>,
) -> JoinHandle<eyre::Result<()>> {
    thread::spawn(move || -> eyre::Result<()> {
        loop {
            match connect_daemon() {
                Ok(s) => {
                    let s = Arc::new(s);
                    let ServerMsg::Handshake(snapshot) =
                        write_frame(&*s, &ClientMsg::Handshake).and_then(|_| read_frame(&*s))?
                    else {
                        continue;
                    };
                    daemon.set_stream(Some(s.clone()));
                    tx.send(elm::Message::Handshake(snapshot))?;
                    let result = read_daemon(&s, &tx);
                    daemon.set_stream(None);
                    result?;
                    tx.send(elm::Message::DaemonDisconnected)?;
                }
                Err(e) => tx.send(elm::Message::Error(e.to_string()))?,
            }
            thread::sleep(Duration::from_secs(1));
        }
    })
}

fn read_daemon(s: &Stream, tx: &Sender<elm::Message>) -> Result<(), SendError<elm::Message>> {
    loop {
        let msg: ServerMsg = match read_frame(s) {
            Ok(msg) => msg,
            Err(_) => {
                return Ok(());
            }
        };
        tx.send(msg.into())?;
    }
}

impl From<ServerMsg> for elm::Message {
    fn from(msg: ServerMsg) -> elm::Message {
        match msg {
            ServerMsg::Handshake(snapshot) => elm::Message::Handshake(snapshot),
            ServerMsg::TrackStarted(track) => elm::Message::TrackStarted(track),
        }
    }
}
