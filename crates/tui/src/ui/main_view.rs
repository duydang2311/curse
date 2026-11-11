use crate::{app::Context, file_tree::FileTree};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    symbols::border,
    text::Span,
    widgets::{Block, Widget},
};

#[derive(Debug)]
pub struct MainView {
    pub file_tree: FileTree,
}

impl MainView {
    pub fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        context: &mut Context,
    ) -> eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => context.exit(),
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                context.exit()
            }
            KeyCode::Enter => {
                let Some(current) = self.file_tree.get_current() else {
                    return Ok(());
                };

                if current.path.is_dir() {
                    let _ = self.file_tree.set_cwd(current.path.clone());
                } else if current.path.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("mp3")
                        || ext.eq_ignore_ascii_case("ogg")
                        || ext.eq_ignore_ascii_case("wav")
                }) {
                    context
                        .daemon
                        .play(current.path.clone())
                        .expect("cant play");
                }
                // let mut path_buf = std::env::home_dir().unwrap();
                // path_buf.push("Downloads");
                // path_buf.push("7⧸16 Irregular Time Signature [858423955].mp3");
                // context.daemon.play(path_buf).expect("cant play");
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.file_tree.get_cwd().parent() {
                    self.file_tree
                        .set_cwd(parent.to_path_buf())
                        .expect("failed to go back");
                }
            }
            KeyCode::Up => self.file_tree.move_up(),
            KeyCode::Down => self.file_tree.move_down(),
            _ => {}
        }
        Ok(())
    }

    pub fn draw(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> eyre::Result<()> {
        let [top, bottom] =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [top_left, top_right] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(2)]).areas(top);
        self.file_tree.render(top_left, buf);
        render_now_playing_block(top_right, buf);
        render_status_bar(bottom, buf);
        Ok(())
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
