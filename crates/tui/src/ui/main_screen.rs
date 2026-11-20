use crate::{
    app::{AppMsg, Cmd},
    ui::{file_tree::FileTree, now_playing::NowPlaying},
    utils::has_audio_extension,
};
use crossterm::event::{KeyCode, KeyEvent};
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

    pub fn update(&mut self, msg: AppMsg) -> Cmd {
        self.now_playing.update(&msg);
        match msg {
            AppMsg::Key(k) => {
                self.error = None;
                match self.handle_key(k) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        self.error = Some(e.to_string());
                        Cmd::None
                    }
                }
            }
            AppMsg::Error(e) => {
                self.error = Some(e);
                Cmd::None
            }
            _ => Cmd::None,
        }
    }

    fn handle_key(&mut self, k: KeyEvent) -> eyre::Result<Cmd> {
        match k.code {
            KeyCode::Enter => {
                let Some(current) = self.file_tree.get_current() else {
                    return Ok(Cmd::None);
                };
                let path = current.path.clone();
                if path.is_dir() {
                    self.file_tree.set_cwd(path)?;
                } else if has_audio_extension(&path) {
                    return Ok(Cmd::Play(path));
                }
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.file_tree.get_cwd().parent() {
                    self.file_tree.set_cwd(parent.to_path_buf())?;
                }
            }
            KeyCode::Up => self.file_tree.move_up(),
            KeyCode::Down => self.file_tree.move_down(),
            _ => {}
        }
        Ok(Cmd::None)
    }

    pub fn view(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let [top, bottom] =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [top_left, top_right] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(2)]).areas(top);
        self.file_tree.render(top_left, buf);
        self.now_playing.render(top_right, buf);
        render_status_bar(bottom, buf, self.error.as_deref());
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
