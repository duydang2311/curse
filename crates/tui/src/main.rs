use crate::{app::App, file_tree::FileTree};
use std::env;

mod app;
mod file_tree;

fn main() -> eyre::Result<()> {
    ratatui::run(|term| {
        App {
            exit: false,
            file_tree: FileTree::new(match env::current_dir() {
                Ok(path) => path,
                Err(e) => env::home_dir().ok_or(e)?,
            })?,
        }
        .run(term)
    })
}
