use crate::elm;
use common::{
    get_socket_name,
    msg::{ClientMsg, ServerMsg, write_frame},
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
                    daemon.set_stream(Some(s.clone()));
                    tx.send(elm::Message::DaemonConnected)?;
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

fn read_daemon(mut s: &Stream, tx: &Sender<elm::Message>) -> Result<(), SendError<elm::Message>> {
    loop {
        let mut len_buf = [0u8; 4];
        if s.read_exact(&mut len_buf).is_err() {
            return Ok(());
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if s.read_exact(&mut buf).is_err() {
            return Ok(());
        }

        if let Ok(msg) = postcard::from_bytes::<ServerMsg>(&buf) {
            tx.send(msg.into())?;
        }
    }
}

impl From<ServerMsg> for elm::Message {
    fn from(msg: ServerMsg) -> elm::Message {
        match msg {
            ServerMsg::TrackStarted(track) => elm::Message::TrackStarted(track),
        }
    }
}
