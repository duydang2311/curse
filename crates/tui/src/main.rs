use crate::{
    app::App,
    daemon::{Daemon, spawn_daemon_thread},
    elm::View,
    ui::{focus_screen::FocusScreen, main_screen::MainScreen},
    utils::ResultExt,
};
use common::spawn::spawn_daemon_detached;
use crossterm::event::{self, Event, KeyEventKind};
use eyre::{Context, eyre};
use std::{
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

    let mut app = App::new(MainScreen::from_current_dir()?.into());
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
            let msg = rx.recv()?;
            if matches!(msg, elm::Message::Resize) {
                continue;
            }
            match app.update(&msg).ok_or_report(&tx) {
                Some(elm::Command::Exit) => {
                    break;
                }
                Some(cmd) => run_cmd(cmd, &mut app, &daemon, &tx),
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

fn run_cmd(cmd: elm::Command, app: &mut App, daemon: &Daemon, tx: &Sender<elm::Message>) {
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
            elm::Command::ShowMainScreen => {
                match MainScreen::from_current_dir().ok_or_report(&tx) {
                    Some(screen) => app.set_screen(screen.into()),
                    None => {}
                }
            }
            elm::Command::ShowFocusScreen => {
                app.set_screen(FocusScreen.into());
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
            if !event::poll(Duration::from_millis(200))? {
                continue;
            }
            let msg = match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => elm::Message::Key(k),
                Event::Resize(_, _) => elm::Message::Resize,
                _ => continue,
            };
            if tx.send(msg).is_err() {
                break;
            }
        }
        Ok(())
    })
}
