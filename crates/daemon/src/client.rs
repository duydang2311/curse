use crate::{
    app::{App, AppMsg},
    audio::AudioMsg,
};
use common::msg::{ClientMsg, ServerMsg, write_frame};
use eyre::eyre;
use interprocess::local_socket::{Listener, Stream, prelude::*};
use std::{
    io::{BufReader, Read},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug, Default, Clone)]
pub struct Clients(Arc<Mutex<Vec<Arc<Stream>>>>);

impl Clients {
    pub fn add(&self, stream: Arc<Stream>) {
        self.0.lock().unwrap().push(stream);
        println!("add count: {}", self.0.lock().unwrap().len());
    }

    pub fn remove(&self, stream: &Arc<Stream>) {
        self.0.lock().unwrap().retain(|s| !Arc::ptr_eq(s, stream));
        println!("remove count: {}", self.0.lock().unwrap().len());
    }

    pub fn broadcast(&self, msg: &ServerMsg) {
        self.0.lock().unwrap().retain(|s| {
            let res = write_frame(&**s, msg);
            eprintln!("write_frame: {:?}", res);
            return res.is_ok();
        });
    }
}

pub fn spawn_client_thread(listener: Listener, app: App) -> JoinHandle<()> {
    thread::spawn(move || {
        for conn in listener.incoming().filter_map(|conn| {
            conn.map_err(|e| eyre!("incoming connection failed: {e}"))
                .ok()
        }) {
            let conn = Arc::new(conn);
            app.clients.add(conn.clone());
            thread::spawn({
                let app = app.clone();
                move || {
                    handle_client(conn.clone(), app.clone());
                    app.clients.remove(&conn);
                }
            });
        }
    })
}

fn handle_client(stream: Arc<Stream>, app: App) {
    let mut s = BufReader::new(&*stream);
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

        println!("received message len {}: {:?}", len, msg);
        match msg {
            ClientMsg::Stop => {
                if let Err(e) = app.tx.send(AppMsg::Shutdown) {
                    eprintln!("failed to send shutdown signal: {}", e);
                }
                break;
            }
            ClientMsg::Play(path_buf) => {
                if let Err(e) = app.audio_tx.send(AudioMsg::Play(path_buf)) {
                    eprintln!("failed to send play audio msg: {}", e);
                }
            }
        };
    }
}
