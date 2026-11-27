use crate::{elm, utils::has_audio_extension};
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::{
    fs,
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileTree {
    root: usize,
    nodes: Vec<Node>,
    visible: Vec<(usize, u16)>,
    cursor: usize,
    offset: usize,
}

#[derive(Debug)]
pub struct Node {
    path: PathBuf,
    name: String,
    kind: NodeKind,
    parent: Option<usize>,
}

#[derive(Debug)]
pub enum NodeKind {
    Directory(Dir),
    File,
}

#[derive(Debug, Default)]
pub struct Dir {
    children: Option<Range<usize>>,
    expanded: bool,
}

impl FileTree {
    pub fn new(cwd: PathBuf) -> eyre::Result<FileTree> {
        let mut tree = FileTree {
            root: 0,
            nodes: vec![],
            visible: vec![],
            cursor: 0,
            offset: 0,
        };
        tree.set_cwd(cwd)?;
        Ok(tree)
    }

    fn dir(&self, id: usize) -> Option<&Dir> {
        match &self.nodes[id].kind {
            NodeKind::Directory(dir) => Some(dir),
            NodeKind::File => None,
        }
    }

    fn dir_mut(&mut self, id: usize) -> Option<&mut Dir> {
        match &mut self.nodes[id].kind {
            NodeKind::Directory(dir) => Some(dir),
            NodeKind::File => None,
        }
    }

    pub fn get_cwd(&self) -> &Path {
        &self.nodes[self.root].path
    }

    pub fn set_cwd(&mut self, cwd: PathBuf) -> eyre::Result<()> {
        let mut nodes = vec![Node {
            name: lossy_name(cwd.as_os_str()),
            path: cwd,
            kind: NodeKind::Directory(Dir::default()),
            parent: None,
        }];
        nodes.extend(read_dir_nodes(&nodes[0].path, 0)?);
        let len = nodes.len();
        if let NodeKind::Directory(dir) = &mut nodes[0].kind {
            dir.children = Some(1..len);
            dir.expanded = true;
        }
        self.nodes = nodes;
        self.root = 0;
        self.cursor = 0;
        self.offset = 0;
        self.rebuild_visible();
        Ok(())
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.visible.len() {
            self.cursor += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn current_id(&self) -> Option<usize> {
        self.visible.get(self.cursor).map(|&(id, _)| id)
    }

    pub fn get_current(&self) -> Option<&Node> {
        self.current_id().map(|id| &self.nodes[id])
    }

    pub fn expand(&mut self, id: usize) -> eyre::Result<()> {
        self.ensure_loaded(id)?;
        self.set_expanded(id, true);
        Ok(())
    }

    pub fn collapse(&mut self, id: usize) {
        self.set_expanded(id, false);
    }

    pub fn toggle(&mut self, id: usize) -> eyre::Result<()> {
        match self.dir(id) {
            Some(dir) if dir.expanded => Ok(self.collapse(id)),
            Some(_) => self.expand(id),
            None => Ok(()),
        }
    }

    fn set_expanded(&mut self, id: usize, value: bool) {
        if let Some(dir) = self.dir_mut(id) {
            dir.expanded = value;
            self.rebuild_visible();
        }
    }

    fn ensure_loaded(&mut self, id: usize) -> eyre::Result<()> {
        let Some(Dir { children: None, .. }) = self.dir(id) else {
            return Ok(());
        };
        let new = read_dir_nodes(&self.nodes[id].path, id)?;
        let start = self.nodes.len();
        self.nodes.extend(new);
        let end = self.nodes.len();
        if let Some(dir) = self.dir_mut(id) {
            dir.children = Some(start..end);
        }
        Ok(())
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        let mut stack = vec![(self.root, 0u16)];
        while let Some((id, depth)) = stack.pop() {
            self.visible.push((id, depth));
            if let Some(Dir {
                children: Some(ch),
                expanded: true,
            }) = self.dir(id)
            {
                stack.extend(ch.clone().rev().map(|c| (c, depth + 1)));
            }
        }
        self.cursor = self.cursor.min(self.visible.len().saturating_sub(1));
    }
}

fn read_dir_nodes(path: &Path, parent: usize) -> eyre::Result<Vec<Node>> {
    let mut nodes = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let kind = if path.is_dir() {
            NodeKind::Directory(Dir::default())
        } else if has_audio_extension(&path) {
            NodeKind::File
        } else {
            continue;
        };
        nodes.push(Node {
            name: lossy_name(&entry.file_name()),
            path,
            kind,
            parent: Some(parent),
        });
    }
    Ok(nodes)
}

fn lossy_name(name: &std::ffi::OsStr) -> String {
    name.to_string_lossy().into_owned()
}

const TITLE: &str = "File system";

impl elm::View for FileTree {
    fn view(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered().border_set(border::ROUNDED).title(TITLE);
        let height = block.inner(area).height as usize;
        let lines: Vec<Line> = self
            .visible
            .iter()
            .enumerate()
            .skip(self.cursor.saturating_sub(height.saturating_sub(1)))
            .take(height)
            .map(|(idx, &(id, depth))| {
                let marker = match self.dir(id) {
                    Some(Dir { expanded: true, .. }) => "▾ ",
                    Some(_) => "▸ ",
                    None => "  ",
                };
                let indent = "  ".repeat(depth as usize);
                let line = Line::raw(format!("{indent}{marker}{}", self.nodes[id].name));
                if self.cursor == idx {
                    line.style(Style::new().bg(Color::Blue))
                } else {
                    line
                }
            })
            .collect();
        Paragraph::new(lines).block(block).render(area, buf);
    }

    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        let elm::Message::Key(k) = msg else {
            return Ok(elm::Command::None);
        };
        match k.code {
            KeyCode::Enter => {
                let Some(id) = self.current_id() else {
                    return Ok(elm::Command::None);
                };
                let path = self.nodes[id].path.clone();
                if self.dir(id).is_some() {
                    self.set_cwd(path)?;
                } else {
                    return Ok(elm::Command::AddTrack(path));
                }
            }
            KeyCode::Char(' ') => {
                if let Some(id) = self.current_id() {
                    self.toggle(id)?;
                }
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.get_cwd().parent() {
                    self.set_cwd(parent.to_path_buf())?;
                }
            }
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            _ => {}
        }
        Ok(elm::Command::None)
    }
}
