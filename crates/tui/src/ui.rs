use crate::{
    app::{AppMsg, Cmd},
    ui::main_screen::MainScreen,
};

pub mod file_tree;
pub mod main_screen;
pub mod now_playing;

#[derive(Debug)]
pub enum Screen {
    Main(MainScreen),
}

impl Screen {
    pub fn update(&mut self, msg: AppMsg) -> Cmd {
        match self {
            Screen::Main(screen) => screen.update(msg),
        }
    }

    pub fn view(&self, frame: &mut ratatui::Frame) {
        match self {
            Screen::Main(screen) => screen.view(frame),
        }
    }
}

impl From<MainScreen> for Screen {
    fn from(screen: MainScreen) -> Screen {
        Screen::Main(screen)
    }
}
