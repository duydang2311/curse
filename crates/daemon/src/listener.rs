use crate::{
    app::{App, AppMsg},
    audio::AudioMsg,
};
use common::msg::ClientMsg;
use eyre::eyre;
use interprocess::local_socket::{Listener, Stream, prelude::*};
use std::{
    io::{BufReader, Read},
    thread,
};

pub fn run_listener_thread(listener: Listener, app: App) {
    for conn in listener.incoming().filter_map(|conn| {
        conn.map_err(|e| eyre!("incoming connection failed: {e}"))
            .ok()
    }) {
        let app = app.clone();
        let reader = BufReader::new(conn);
        thread::spawn(move || handle_client(reader, app));
    }
}

fn handle_client(mut conn: BufReader<Stream>, app: App) {
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(e) = conn.read_exact(&mut len_buf) {
            eprintln!("failed to read len buffer: {}", e);
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if let Err(e) = conn.read_exact(&mut buf) {
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
