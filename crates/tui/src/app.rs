use crate::{elm, ui::Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct App {
    screen: Screen,
}

impl App {
    pub fn new(screen: Screen) -> App {
        App { screen }
    }
}

impl elm::View for App {
    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        if let elm::Message::Key(k) = msg
            && is_exit_key(k)
        {
            return Ok(elm::Command::Exit);
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
