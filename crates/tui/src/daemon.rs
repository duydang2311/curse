use common::{
    get_socket_name,
    msg::{ClientMsg, ServerMsg, write_frame},
};
use interprocess::local_socket::{Stream, prelude::*};
use serde::Serialize;
use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, mpsc::Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::app::AppEvent;

#[derive(Debug, Clone)]
pub struct Daemon {
    stream: Option<Arc<Stream>>,
}

impl Daemon {
    pub fn new() -> Daemon {
        Daemon { stream: None }
    }

    pub fn set_stream(&mut self, stream: Option<Arc<Stream>>) {
        self.stream = stream;
    }

    pub fn play(&mut self, path: PathBuf) -> eyre::Result<()> {
        self.write_frame(&ClientMsg::Play(path))?;
        Ok(())
    }

    fn write_frame<M: Serialize>(&mut self, msg: &M) -> eyre::Result<()> {
        match &self.stream {
            Some(s) => {
                let res = write_frame(&**s, msg)?;
                println!("write_frame: {:?}", res);
                Ok(())
            }
            None => {
                println!("None");
                Ok(())
            }
        }
    }
}

fn connect_daemon() -> eyre::Result<Stream> {
    Ok(Stream::connect(get_socket_name()?)?)
}

pub fn spawn_daemon_thread(tx: Sender<AppEvent>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match connect_daemon() {
                Ok(s) => {
                    let s = Arc::new(s);
                    println!("daemon connect ok");
                    tx.send(AppEvent::DaemonConnected(s.clone()))
                        .expect("fail to send daemon connected event");
                    read_daemon(&*s);
                    println!("daemon.stream = None");
                    tx.send(AppEvent::DaemonDisconnected(s.clone()))
                        .expect("fail to send daemon disconnected event");
                }
                Err(e) => {
                    println!("daemon connect error: {}", e);
                    // TODO: toast: failed to connect to daemon, retrying in 1 second
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    })
}

pub fn read_daemon(mut s: &Stream) {
    loop {
        let mut len_buf = [0u8; 4];
        if s.read_exact(&mut len_buf).is_err() {
            return;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if s.read_exact(&mut buf).is_err() {
            return;
        }

        let msg: ServerMsg = match postcard::from_bytes(&buf) {
            Ok(msg) => msg,
            Err(e) => {
                // TODO: add toast err
                continue;
            }
        };
    }
}
