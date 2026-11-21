use crate::elm;
use ratatui::{
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::path::PathBuf;

const TITLE: &str = "Now playing";

#[derive(Debug, Default)]
pub struct NowPlaying {
    track: Option<PathBuf>,
}

impl elm::View for NowPlaying {
    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
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
        Paragraph::new(line).block(block).render(area, buffer);
    }

    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match msg {
            elm::Message::TrackStarted(track) => self.track = Some(track.clone()),
            elm::Message::DaemonDisconnected => self.track = None,
            elm::Message::Handshake(snapshot) => {
                self.track = snapshot.current_track.clone();
            }
            _ => {}
        }
        Ok(elm::Command::None)
    }
}
