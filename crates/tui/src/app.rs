use crate::ui::Screen;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    screen: Screen,
}

#[derive(Debug)]
pub enum AppMsg {
    Key(KeyEvent),
    DaemonConnected,
    DaemonDisconnected,
    TrackStarted(PathBuf),
    Error(String),
}

#[derive(Debug)]
#[must_use]
pub enum Cmd {
    None,
    Play(PathBuf),
}

impl App {
    pub fn new(screen: Screen) -> App {
        App {
            running: true,
            screen,
        }
    }

    pub fn update(&mut self, msg: AppMsg) -> Cmd {
        if let AppMsg::Key(k) = &msg
            && is_quit(k)
        {
            self.running = false;
            return Cmd::None;
        }
        self.screen.update(msg)
    }

    pub fn view(&self, frame: &mut ratatui::Frame) {
        self.screen.view(frame);
    }
}

fn is_quit(k: &KeyEvent) -> bool {
    match k.code {
        KeyCode::Char('q') => true,
        KeyCode::Char('c') => k.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}
