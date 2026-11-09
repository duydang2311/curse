use std::path::PathBuf;

use crate::{daemon::DaemonClient, file_tree::FileTree};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    symbols::border,
    text::Span,
    widgets::{Block, Widget},
};

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub file_tree: FileTree,
    pub daemon_client: DaemonClient,
}

impl App {
    pub fn run(&mut self, term: &mut ratatui::DefaultTerminal) -> eyre::Result<()> {
        while !self.exit {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> eyre::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Enter => {
                let mut path_buf = std::env::home_dir().unwrap();
                path_buf.push("Downloads");
                path_buf.push("7⧸16 Irregular Time Signature [858423955].mp3");
                self.daemon_client.play(path_buf).expect("cant play");
            }
            KeyCode::Up => self.file_tree.move_up(),
            KeyCode::Down => self.file_tree.move_down(),
            _ => {}
        }
    }
}

fn render_now_playing_block(area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    let block = Block::bordered().border_set(border::ROUNDED);
    block.render(area, buf);
}

fn render_status_bar(area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    let help = Span::raw("? help");
    let [left, right] = Layout::horizontal(vec![
        Constraint::Fill(1),
        Constraint::Length(help.width() as u16),
    ])
    .areas(area);
    Span::raw("q quit").render(left, buf);
    help.render(right, buf);
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [top, bottom] =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [top_left, top_right] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(2)]).areas(top);
        self.file_tree.render(top_left, buf);
        render_now_playing_block(top_right, buf);

        render_status_bar(bottom, buf);
    }
}
