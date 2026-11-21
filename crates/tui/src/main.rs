use crate::{
    app::App,
    daemon::{Daemon, spawn_daemon_thread},
    elm::View,
    ui::{file_tree::FileTree, main_screen::MainScreen},
    utils::ResultExt,
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
mod elm;
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
        loop {
            term.draw(|frame| {
                let area = frame.area();
                let buffer = frame.buffer_mut();
                app.view(area, buffer);
            })?;
            match app.update(&rx.recv()?).ok_or_report(&tx) {
                Some(elm::Command::Exit) => {
                    break;
                }
                Some(cmd) => run_cmd(cmd, &daemon, &tx),
                None => {}
            }
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

fn run_cmd(cmd: elm::Command, daemon: &Daemon, tx: &Sender<elm::Message>) {
    let mut stack: Vec<elm::Command> = vec![cmd];
    while let Some(cmd) = stack.pop() {
        match cmd {
            elm::Command::None => {}
            elm::Command::Exit => {
                unreachable!()
            }
            elm::Command::Batch(cmds) => {
                stack.extend(cmds.into_iter().rev());
            }
            elm::Command::Play(path) => {
                daemon.play(path).ok_or_report(tx);
            }
        }
    }
}

fn spawn_input_thread(
    tx: Sender<elm::Message>,
    exited: Arc<AtomicBool>,
) -> JoinHandle<eyre::Result<()>> {
    thread::spawn(move || -> eyre::Result<()> {
        while !exited.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(200))?
                && let Ok(Event::Key(k)) = event::read()
                && k.kind == KeyEventKind::Press
                && tx.send(elm::Message::Key(k)).is_err()
            {
                break;
            }
        }
        Ok(())
    })
}
