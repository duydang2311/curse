use common::playback::Track;

#[derive(Debug, Default)]
pub struct PlaybackState {
    tracks: Vec<Track>,
    current: usize,
}

impl PlaybackState {
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.tracks.get(self.current)
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn set_current(&mut self, current: usize) {
        self.current = current;
    }
}
