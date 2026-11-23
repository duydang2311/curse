use common::msg::Snapshot;
use crossterm::event::KeyEvent;
use std::path::PathBuf;

use crate::elm;

#[derive(Debug)]
pub enum Message {
    Resize,
    Key(KeyEvent),
    DaemonDisconnected,
    Handshake(Snapshot),
    TrackStarted(PathBuf),
    Error(String),
}

#[derive(Debug)]
pub enum Command {
    None,
    Exit,
    Batch(Vec<Command>),
    Play(PathBuf),
    ShowMainScreen,
    ShowFocusScreen,
}

pub trait View {
    fn view(&self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer);
    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command>;
}

impl Command {
    pub fn try_batch<E>(
        results: impl IntoIterator<Item = Result<Command, E>>,
    ) -> Result<Command, E> {
        results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map(Command::Batch)
    }
}
