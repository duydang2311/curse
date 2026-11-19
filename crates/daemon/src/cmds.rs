use crate::{
    app::{App, AppMsg},
    audio::{
        AudioMsg::{self},
        spawn_audio_thread,
    },
    client::spawn_client_thread,
};
use common::{get_socket_name, msg::ClientMsg, spawn::spawn_daemon_detached};
use eyre::{Context, bail, eyre};
use interprocess::local_socket::{ListenerOptions, Stream, prelude::*};
use std::{
    io::{ErrorKind, Write},
    sync::mpsc,
    time::Duration,
};

const SOCKET_NAME: &str = "curse-daemon.sock";

pub fn start() -> eyre::Result<()> {
    spawn_daemon_detached()
}

pub fn stop() -> eyre::Result<()> {
    let name = get_socket_name()?;
    let mut stream =
        Stream::connect(name).wrap_err_with(|| format!("failed to connect to daemon"))?;
    write_all(&mut stream, &postcard::to_allocvec(&ClientMsg::Stop)?)?;
    println!("daemon stopped successfully");
    Ok(())
}

pub fn run() -> eyre::Result<()> {
    let name = get_socket_name()?;
    let listener = match ListenerOptions::new().name(name.clone()).create_sync() {
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            if Stream::connect(name.clone()).is_ok() {
                bail!("daemon is already running");
            }
            ListenerOptions::new()
                .name(name.clone())
                .try_overwrite(true)
                .max_spin_time(Duration::from_millis(500))
                .create_sync()
                .wrap_err("failed to start daemon")?
        }
        Err(e) => return Err(e).wrap_err("failed to start daemon"),
        Ok(l) => l,
    };

    println!("daemon running at {SOCKET_NAME}");
    let (app_tx, app_rx) = mpsc::channel::<AppMsg>();
    let (audio_tx, audio_rx) = mpsc::channel::<AudioMsg>();
    let app = App::new(app_tx, audio_tx.clone());

    let client_handle = spawn_client_thread(listener, app.clone());
    let audio_handle = spawn_audio_thread(app.clone(), audio_rx);

    app_rx.recv()?;

    let _ = audio_tx.send(AudioMsg::Shutdown);
    audio_handle
        .join()
        .map_err(|_| eyre!("failed to join audio thread"))?;

    Ok(())
}

fn write_all(stream: &mut Stream, bytes: &[u8]) -> eyre::Result<()> {
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}
