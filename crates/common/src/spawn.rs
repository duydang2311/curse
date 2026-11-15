use eyre::eyre;
use interprocess::local_socket::{Stream, prelude::*};
use std::{env, process};

use crate::get_socket_name;

pub fn spawn_daemon_detached() -> eyre::Result<()> {
    let name = get_socket_name()?;
    if let Ok(_) = Stream::connect(name) {
        println!("daemon is already running");
        return Ok(());
    }
    let exe = env::current_exe()?
        .canonicalize()?
        .parent()
        .ok_or_else(|| eyre!("failed to get exe parent directory"))?
        .join(format!("daemon{}", env::consts::EXE_SUFFIX));
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
