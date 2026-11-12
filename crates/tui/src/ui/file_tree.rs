use ratatui::{
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils::has_audio_extension;

#[derive(Debug)]
pub struct FileTree {
    cwd: PathBuf,
    entries: Vec<FileTreeEntry>,
    current_idx: usize,
}

#[derive(Debug)]
pub struct FileTreeEntry {
    pub path: PathBuf,
    pub name: String,
}

impl FileTree {
    pub fn new(cwd: PathBuf) -> eyre::Result<FileTree> {
        let entries = Self::read_entries(&cwd)?;
        let file_tree = FileTree {
            cwd,
            entries,
            current_idx: 0,
        };
        Ok(file_tree)
    }

    pub fn get_cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn set_cwd(&mut self, cwd: PathBuf) -> eyre::Result<()> {
        let entries = Self::read_entries(&cwd)?;
        self.cwd = cwd;
        self.entries = entries;
        self.current_idx = 0;
        Ok(())
    }

    pub fn move_down(&mut self) {
        if self.current_idx + 1 < self.entries.len() {
            self.current_idx = self.current_idx + 1;
        }
    }

    pub fn move_up(&mut self) {
        self.current_idx = self.current_idx.saturating_sub(1);
    }

    pub fn get_current(&self) -> Option<&FileTreeEntry> {
        self.entries.get(self.current_idx)
    }

    fn read_entries(cwd: &Path) -> eyre::Result<Vec<FileTreeEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(cwd)? {
            let entry = entry?;
            let path = entry.path();
            let is_audio = has_audio_extension(&entry.path());
            if !path.is_dir() && !is_audio {
                continue;
            }
            entries.push(FileTreeEntry {
                path: entry.path(),
                name: entry
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|os| os.to_string_lossy().to_string()),
            });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }
}

const TITLE: &str = "File system";

impl Widget for &FileTree {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().border_set(border::ROUNDED).title(TITLE);
        let height = block.inner(area).height as usize;
        let lines: Vec<Line> = self
            .entries
            .iter()
            .enumerate()
            .skip(self.current_idx.saturating_sub(height.saturating_sub(1)))
            .take(height)
            .map(|(idx, entry)| {
                let line = Line::raw(&entry.name);
                if self.current_idx == idx {
                    line.style(Style::new().bg(Color::Blue))
                } else {
                    line
                }
            })
            .collect();
        let paragraph = Paragraph::new(lines).block(block);
        paragraph.render(area, buf);
    }
}
