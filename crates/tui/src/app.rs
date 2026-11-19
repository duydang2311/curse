use crate::{daemon::Daemon, ui::View};
use crossterm::event::KeyEvent;
use interprocess::local_socket::Stream;
use std::sync::{
    Arc,
    mpsc::{Receiver, Sender},
};

#[derive(Debug)]
pub struct App {
    pub context: AppContext,
    pub view: View,
}

#[derive(Debug)]
pub struct AppContext {
    pub tx: Sender<AppEvent>,
    pub rx: Receiver<AppEvent>,
    pub exited: bool,
    pub daemon: Daemon,
}

#[derive(Debug)]
pub enum AppEvent {
    #[allow(dead_code)]
    Tick,
    Key(KeyEvent),
    DaemonConnected(Arc<Stream>),
    DaemonDisconnected(Arc<Stream>),
}

impl App {
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>, view: View, daemon: Daemon) -> App {
        App {
            context: AppContext {
                tx,
                rx,
                exited: false,
                daemon,
            },
            view,
        }
    }

    pub fn draw(&mut self, frame: &mut ratatui::Frame) -> eyre::Result<()> {
        self.view.draw(frame)
    }
}

impl AppContext {
    pub fn exit(&mut self) {
        self.exited = true;
    }

    pub fn tick(&self) -> eyre::Result<()> {
        Ok(self.tx.send(AppEvent::Tick)?)
    }
}
