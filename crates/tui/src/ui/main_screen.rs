use std::env;

use crate::{
    elm,
    ui::{Screen, file_tree::FileTree, now_playing::NowPlaying},
};
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::Widget,
};

#[derive(Debug)]
pub struct MainScreen {
    file_tree: FileTree,
    now_playing: NowPlaying,
    error: Option<String>,
}

impl MainScreen {
    pub fn new(file_tree: FileTree) -> MainScreen {
        MainScreen {
            file_tree,
            now_playing: NowPlaying::default(),
            error: None,
        }
    }

    pub fn from_current_dir() -> eyre::Result<MainScreen> {
        let cwd = match env::current_dir() {
            Ok(path) => path,
            Err(e) => env::home_dir().ok_or(e)?,
        };
        Ok(MainScreen::new(FileTree::new(cwd)?))
    }
}

impl From<MainScreen> for Screen {
    fn from(screen: MainScreen) -> Screen {
        Screen::Main(screen)
    }
}

impl elm::View for MainScreen {
    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        let [top, bottom] =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [top_left, top_right] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(2)]).areas(top);
        self.file_tree.view(top_left, buffer);
        self.now_playing.view(top_right, buffer);
        render_status_bar(bottom, buffer, self.error.as_deref());
    }

    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match msg {
            elm::Message::Error(e) => {
                self.error = Some(e.to_string());
            }
            elm::Message::Key(k) if k.code == KeyCode::Char('2') => {
                return Ok(elm::Command::ShowFocusScreen);
            }
            _ => {}
        }
        elm::Command::try_batch([self.now_playing.update(msg), self.file_tree.update(msg)])
    }
}

fn render_status_bar(area: Rect, buf: &mut Buffer, error: Option<&str>) {
    let help = Span::raw("? help");
    let [left, right] = Layout::horizontal(vec![
        Constraint::Fill(1),
        Constraint::Length(help.width() as u16),
    ])
    .areas(area);
    match error {
        Some(e) => Span::styled(e, Style::new().fg(Color::Red)).render(left, buf),
        None => Span::raw("q quit").render(left, buf),
    }
    help.render(right, buf);
}
