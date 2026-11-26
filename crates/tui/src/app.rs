use crate::{elm, playback::PlaybackState, ui::Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct App {
    screen: Screen,
    playback_state: PlaybackState,
}

impl App {
    pub fn new(screen: Screen) -> App {
        App {
            screen,
            playback_state: PlaybackState::default(),
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }
}

impl elm::View for App {
    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match msg {
            elm::Message::Key(k) if is_exit_key(k) => {
                return Ok(elm::Command::Exit);
            }
            elm::Message::Handshake(snapshot) => {
                self.playback_state.set_current(snapshot.current_track);
                todo!()
            }
            _ => {}
        }
        self.screen.update(msg)
    }

    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        self.screen.view(area, buffer)
    }
}

fn is_exit_key(k: &KeyEvent) -> bool {
    match k.code {
        KeyCode::Char('q') => true,
        KeyCode::Char('c') => k.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}
