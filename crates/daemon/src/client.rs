use crate::app::AppMsg;
use common::msg::{ClientMsg, ServerMsg, write_frame};
use eyre::eyre;
use interprocess::local_socket::{Listener, Stream, prelude::*};
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    sync::{Arc, mpsc::Sender},
    thread::{self, JoinHandle},
};

pub type ClientId = usize;

#[derive(Debug, Default)]
pub struct Clients {
    streams: HashMap<ClientId, Arc<Stream>>,
}

impl Clients {
    pub fn add(&mut self, id: ClientId, stream: Arc<Stream>) {
        self.streams.insert(id, stream);
    }

    pub fn remove(&mut self, id: ClientId) -> Option<Arc<Stream>> {
        self.streams.remove(&id)
    }

    pub fn send(&self, id: ClientId, msg: &ServerMsg) -> eyre::Result<()> {
        let stream = self
            .streams
            .get(&id)
            .ok_or_else(|| eyre!("unknown client {id}"))?;
        write_frame(&**stream, msg)
    }

    pub fn broadcast(&mut self, msg: &ServerMsg) {
        self.streams
            .retain(|id, stream| match write_frame(&**stream, msg) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("failed to write to client {id}: {e}");
                    false
                }
            });
    }
}

pub fn spawn_client_thread(listener: Listener, tx: Sender<AppMsg>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut next_id: ClientId = 0;
        for conn in listener.incoming().filter_map(|conn| {
            conn.map_err(|e| eprintln!("incoming connection failed: {e}"))
                .ok()
        }) {
            let id = next_id;
            next_id += 1;

            let conn = Arc::new(conn);
            if tx.send(AppMsg::ClientConnected(id, conn.clone())).is_err() {
                break;
            }
            thread::spawn({
                let tx = tx.clone();
                move || {
                    handle_client(id, &conn, &tx);
                    let _ = tx.send(AppMsg::ClientDisconnected(id));
                }
            });
        }
    })
}

fn handle_client(id: ClientId, stream: &Stream, tx: &Sender<AppMsg>) {
    let mut s = BufReader::new(stream);
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(e) = s.read_exact(&mut len_buf) {
            eprintln!("failed to read len buffer: {:?}", e);
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if let Err(e) = s.read_exact(&mut buf) {
            eprintln!("failed to read buffer of len {}: {}", len, e);
            break;
        }

        let msg: ClientMsg = match postcard::from_bytes(&buf) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("failed to deserialize buffer: {}", e);
                continue;
            }
        };

        if tx.send(AppMsg::ClientMsg(id, msg)).is_err() {
            break;
        }
    }
}
