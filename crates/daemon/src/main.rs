use clap::Command;
use common::get_socket_name;
use eyre::{Context, bail, eyre};
use interprocess::local_socket::{ListenerOptions, Stream, prelude::*};
use std::io::{BufRead, BufReader, Write};

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
    stream.write_all(b"stop\n")?;
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
    let mut buffer = String::with_capacity(128);
    for mut conn in listener
        .incoming()
        .filter_map(|conn| {
            conn.map_err(|e| eyre!("incoming connection failed: {e}"))
                .ok()
        })
        .map(BufReader::new)
    {
        if let Err(e) = conn.read_line(&mut buffer) {
            eprintln!("failed to read client buffer: {}", e);
            continue;
        }

        reply(
            &mut conn,
            match buffer.trim() {
                "stop" => break,
                "ping" => b"pong\n",
                _ => b"unknown command\n",
            },
        );
        buffer.clear();
    }
    Ok(())
}

fn reply(conn: &mut BufReader<Stream>, buf: &[u8]) {
    if let Err(e) = conn.get_mut().write_all(buf) {
        eprintln!("failed to reply to client: {}", e);
    }
}
