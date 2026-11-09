use crate::{
    app::{App, AppMsg},
    audio::{
        AudioMsg::{self},
        run_audio_thread,
    },
    listener::run_listener_thread,
};
use common::{get_socket_name, msg::ClientMsg};
use eyre::{Context, bail, eyre};
use interprocess::local_socket::{ListenerOptions, Stream, prelude::*};
use std::{env, io::Write, process, sync::mpsc, thread};

const SOCKET_NAME: &str = "curse-daemon.sock";

pub fn start() -> eyre::Result<()> {
    let name = get_socket_name()?;
    if let Ok(_) = Stream::connect(name) {
        bail!("daemon is already running")
    }
    let exe = env::current_exe()?;
    let mut cmd = process::Command::new(exe);
    cmd.arg("run")
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACH_PROCESS_AND_CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0008 | 0x0000_0200;
        cmd.creation_flags(DETACH_PROCESS_AND_CREATE_NEW_PROCESS_GROUP);
    }
    let child = cmd.spawn()?;
    println!("daemon started in background (pid: {})", child.id());
    Ok(())
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
    let listener = ListenerOptions::new()
        .name(name)
        .create_sync()
        .wrap_err_with(|| format!("failed to start daemon"))?;
    println!("daemon running at {SOCKET_NAME}");

    let (app_tx, app_rx) = mpsc::channel::<AppMsg>();
    let (audio_tx, audio_rx) = mpsc::channel::<AudioMsg>();
    let app = App::new(app_tx, audio_tx.clone());

    thread::spawn(move || {
        let app = app.clone();
        run_listener_thread(listener, app);
    });

    let audio_handle = thread::spawn(|| {
        run_audio_thread(audio_rx);
    });

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
