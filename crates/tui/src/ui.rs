use crossterm::event::KeyEvent;

use crate::{app::Context, ui::main_view::MainView};

pub mod file_tree;
pub mod main_view;

#[derive(Debug)]
pub enum View {
    Main(MainView),
}

impl View {
    pub fn draw(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> eyre::Result<()> {
        match self {
            View::Main(main_view) => main_view.draw(area, buf),
        }
    }

    pub fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        context: &mut Context,
    ) -> eyre::Result<()> {
        match self {
            View::Main(main_view) => main_view.handle_key_event(key_event, context),
        }
    }
}

impl Into<View> for MainView {
    fn into(self) -> View {
        View::Main(self)
    }
}
