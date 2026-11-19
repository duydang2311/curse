use crate::{
    app::{App, AppEvent},
    daemon::{Daemon, spawn_daemon_thread},
    ui::{file_tree::FileTree, main_view::MainView},
};
use common::spawn::spawn_daemon_detached;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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

    let (app_tx, app_rx) = mpsc::channel();
    let input_exited = Arc::new(AtomicBool::new(false));
    let input_handler = spawn_input_thread(app_tx.clone(), input_exited.clone());

    let daemon = Daemon::new();
    let daemon_handle = spawn_daemon_thread(app_tx.clone());

    let result = ratatui::run(|term| -> eyre::Result<()> {
        let mut app = App::new(
            app_tx,
            app_rx,
            MainView {
                file_tree: FileTree::new(match env::current_dir() {
                    Ok(path) => path,
                    Err(e) => env::home_dir().ok_or(e)?,
                })?,
            }
            .into(),
            daemon,
        );

        while !app.context.exited {
            let mut last_draw_result = Ok(());
            term.draw(|frame| {
                last_draw_result = app.draw(frame);
            })?;
            last_draw_result?;
            match app.context.rx.recv()? {
                AppEvent::Key(k) => app.view.handle_key_event(k, &mut app.context)?,
                AppEvent::DaemonConnected(s) => {
                    app.context.daemon.set_stream(Some(s));
                }
                AppEvent::DaemonDisconnected(s) => {
                    app.context.daemon.set_stream(None);
                }
                _ => {}
            };
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

fn spawn_input_thread(
    tx: Sender<AppEvent>,
    exited: Arc<AtomicBool>,
) -> JoinHandle<eyre::Result<()>> {
    thread::spawn(move || -> eyre::Result<()> {
        while !exited.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(200))?
                && let Ok(Event::Key(k)) = event::read()
                && k.kind == KeyEventKind::Press
                && tx.send(AppEvent::Key(k)).is_err()
            {
                break;
            }
        }
        Ok(())
    })
}
