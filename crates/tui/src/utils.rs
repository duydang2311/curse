use crate::elm;
use std::{fmt::Display, path::Path, sync::mpsc::Sender};

pub trait ResultExt<T> {
    fn ok_or_report(self, tx: &Sender<elm::Message>) -> Option<T>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn ok_or_report(self, tx: &Sender<elm::Message>) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                let _ = tx.send(elm::Message::Error(e.to_string()));
                None
            }
        }
    }
}

pub fn has_audio_extension(path: &Path) -> bool {
    path.extension().is_some_and(|ext| {
        ext.eq_ignore_ascii_case("mp3")
            || ext.eq_ignore_ascii_case("ogg")
            || ext.eq_ignore_ascii_case("wav")
    })
}
