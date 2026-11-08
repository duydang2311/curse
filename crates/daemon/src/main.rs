use clap::Command;
use common::{get_socket_name, msg::ClientMsg};
use eyre::{Context, bail, eyre};
use interprocess::local_socket::{ListenerOptions, Stream, prelude::*};
use std::{
    io::{BufReader, Read, Write},
    sync::mpsc::{self, Sender},
};

const SOCKET_NAME: &str = "curse-daemon.sock";

fn cli() -> Command {
    Command::new("curse-daemon")
        .about("curse music player daemon")
        .version("v0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("start").about("Start the daemon (detached)"))
        .subcommand(Command::new("stop").about("Stop the daemon"))
        .subcommand(Command::new("run").about("Start and run the daemon in foreground"))
}

fn main() -> eyre::Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", _)) => start()?,
        Some(("stop", _)) => stop()?,
        Some(("run", _)) => run()?,
        _ => unreachable!(),
    }
    Ok(())
}

fn start() -> eyre::Result<()> {
    let name = get_socket_name()?;
    if let Ok(_) = Stream::connect(name) {
        bail!("daemon is already running")
    }
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("run")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
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

fn stop() -> eyre::Result<()> {
    let name = get_socket_name()?;
    let mut stream =
        Stream::connect(name).wrap_err_with(|| format!("failed to connect to daemon"))?;
    write_all(&mut stream, &postcard::to_allocvec(&ClientMsg::Stop)?)?;
    println!("daemon stopped successfully");
    Ok(())
}

fn run() -> eyre::Result<()> {
    let name = get_socket_name()?;
    let listener = ListenerOptions::new()
        .name(name)
        .create_sync()
        .wrap_err_with(|| format!("failed to start daemon"))?;
    println!("daemon running at {SOCKET_NAME}");

    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        for conn in listener.incoming().filter_map(|conn| {
            conn.map_err(|e| eyre!("incoming connection failed: {e}"))
                .ok()
        }) {
            let shutdown = shutdown_tx.clone();
            let reader = BufReader::new(conn);
            std::thread::spawn(move || handle_client(reader, shutdown));
        }
    });
    shutdown_rx.recv()?;
    Ok(())
}

fn handle_client(mut conn: BufReader<Stream>, shutdown_tx: Sender<()>) {
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(e) = conn.read_exact(&mut len_buf) {
            eprintln!("failed to read len buffer: {}", e);
            continue;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if let Err(e) = conn.read_exact(&mut buf) {
            eprintln!("failed to read buffer of len {}: {}", len, e);
            continue;
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
                if let Err(e) = shutdown_tx.send(()) {
                    eprintln!("failed to send shutdown signal: {}", e);
                }
                break;
            }
            ClientMsg::Play(path_buf) => {
                todo!()
            }
        };
    }
}

fn write_all(stream: &mut Stream, bytes: &[u8]) -> eyre::Result<()> {
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}
