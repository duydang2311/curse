use crate::{
    elm,
    ui::{focus_screen::FocusScreen, main_screen::MainScreen},
};

pub mod file_tree;
pub mod focus_screen;
pub mod main_screen;
pub mod now_playing;

#[derive(Debug)]
pub enum Screen {
    Main(MainScreen),
    Focus(FocusScreen),
}

impl elm::View for Screen {
    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match self {
            Screen::Main(screen) => screen.update(msg),
            Screen::Focus(screen) => screen.update(msg),
        }
    }

    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        match self {
            Screen::Main(screen) => screen.view(area, buffer),
            Screen::Focus(screen) => screen.view(area, buffer),
        }
    }
}
