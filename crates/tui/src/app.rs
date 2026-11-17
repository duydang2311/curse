use crate::{daemon::DaemonClient, ui::View};
use crossterm::event::KeyEvent;
use std::sync::mpsc::{Receiver, Sender};

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
    pub daemon: DaemonClient,
}

#[derive(Debug)]
pub enum AppEvent {
    #[allow(dead_code)]
    Tick,
    Key(KeyEvent),
}

impl App {
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>, view: View) -> App {
        App {
            context: AppContext {
                tx,
                rx,
                exited: false,
                daemon: DaemonClient::new(),
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
