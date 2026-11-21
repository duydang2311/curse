use crate::{elm, ui::main_screen::MainScreen};

pub mod file_tree;
pub mod main_screen;
pub mod now_playing;

#[derive(Debug)]
pub enum Screen {
    Main(MainScreen),
}

impl elm::View for Screen {
    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match self {
            Screen::Main(screen) => screen.update(msg),
        }
    }

    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        match self {
            Screen::Main(screen) => screen.view(area, buffer),
        }
    }
}

impl From<MainScreen> for Screen {
    fn from(screen: MainScreen) -> Screen {
        Screen::Main(screen)
    }
}
