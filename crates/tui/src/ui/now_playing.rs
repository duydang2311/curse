use std::path::PathBuf;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::app::AppMsg;

const TITLE: &str = "Now playing";

#[derive(Debug, Default)]
pub struct NowPlaying {
    track: Option<PathBuf>,
}

impl NowPlaying {
    pub fn update(&mut self, msg: &AppMsg) {
        match msg {
            AppMsg::TrackStarted(track) => self.track = Some(track.clone()),
            AppMsg::DaemonDisconnected => self.track = None,
            _ => {}
        }
    }
}

impl Widget for &NowPlaying {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::ROUNDED).title(TITLE);
        let line = match &self.track {
            None => Line::raw("Nothing playing"),
            Some(track) => Line::raw(
                track
                    .file_name()
                    .unwrap_or(track.as_os_str())
                    .to_string_lossy(),
            ),
        };
        Paragraph::new(line).block(block).render(area, buf);
    }
}
