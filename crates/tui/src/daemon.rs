use crate::app::AppMsg;
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

pub fn spawn_daemon_thread(daemon: Daemon, tx: Sender<AppMsg>) -> JoinHandle<eyre::Result<()>> {
    thread::spawn(move || -> eyre::Result<()> {
        loop {
            match connect_daemon() {
                Ok(s) => {
                    let s = Arc::new(s);
                    daemon.set_stream(Some(s.clone()));
                    tx.send(AppMsg::DaemonConnected)?;
                    let result = read_daemon(&s, &tx);
                    daemon.set_stream(None);
                    result?;
                    tx.send(AppMsg::DaemonDisconnected)?;
                }
                Err(e) => tx.send(AppMsg::Error(e.to_string()))?,
            }
            thread::sleep(Duration::from_secs(1));
        }
    })
}

fn read_daemon(mut s: &Stream, tx: &Sender<AppMsg>) -> Result<(), SendError<AppMsg>> {
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

impl From<ServerMsg> for AppMsg {
    fn from(msg: ServerMsg) -> AppMsg {
        match msg {
            ServerMsg::TrackStarted(track) => AppMsg::TrackStarted(track),
        }
    }
}
