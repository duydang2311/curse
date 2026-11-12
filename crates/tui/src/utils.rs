use std::path::Path;

pub fn has_audio_extension(path: &Path) -> bool {
    path.extension().is_some_and(|ext| {
        ext.eq_ignore_ascii_case("mp3")
            || ext.eq_ignore_ascii_case("ogg")
            || ext.eq_ignore_ascii_case("wav")
    })
}
