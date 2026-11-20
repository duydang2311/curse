use crate::{
    app::{App, AppMsg, Cmd},
    daemon::{Daemon, spawn_daemon_thread},
    ui::{file_tree::FileTree, main_screen::MainScreen},
};
use common::spawn::spawn_daemon_detached;
use crossterm::event::{self, Event, KeyEventKind};
use eyre::{Context, eyre};
use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

mod app;
mod daemon;
mod ui;
mod utils;

fn main() -> eyre::Result<()> {
    spawn_daemon_detached()?;

    let cwd = match env::current_dir() {
        Ok(path) => path,
        Err(e) => env::home_dir().ok_or(e)?,
    };
    let mut app = App::new(MainScreen::new(FileTree::new(cwd)?).into());

    let (tx, rx) = mpsc::channel();
    let input_exited = Arc::new(AtomicBool::new(false));
    let input_handler = spawn_input_thread(tx.clone(), input_exited.clone());

    let daemon = Daemon::default();
    spawn_daemon_thread(daemon.clone(), tx.clone());

    let result = ratatui::run(|term| -> eyre::Result<()> {
        while app.running {
            term.draw(|frame| app.view(frame))?;
            let cmd = app.update(rx.recv()?);
            run_cmd(cmd, &daemon, &tx);
        }
        Ok(())
    });

    input_exited.store(true, Ordering::Relaxed);
    input_handler
        .join()
        .map_err(|e| eyre!("failed to join input thread: {:?}", e))?
        .wrap_err("failed to handle input event")?;
    result
}

fn run_cmd(cmd: Cmd, daemon: &Daemon, tx: &Sender<AppMsg>) {
    let result = match cmd {
        Cmd::None => Ok(()),
        Cmd::Play(path) => daemon.play(path),
    };
    if let Err(e) = result {
        let _ = tx.send(AppMsg::Error(e.to_string()));
    }
}

fn spawn_input_thread(tx: Sender<AppMsg>, exited: Arc<AtomicBool>) -> JoinHandle<eyre::Result<()>> {
    thread::spawn(move || -> eyre::Result<()> {
        while !exited.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(200))?
                && let Ok(Event::Key(k)) = event::read()
                && k.kind == KeyEventKind::Press
                && tx.send(AppMsg::Key(k)).is_err()
            {
                break;
            }
        }
        Ok(())
    })
}
