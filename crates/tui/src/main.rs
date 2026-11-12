use crate::{
    app::App, app::Context, daemon::DaemonClient, ui::file_tree::FileTree, ui::main_view::MainView,
};
use std::env;

mod app;
mod daemon;
mod ui;
mod utils;

fn main() -> eyre::Result<()> {
    ratatui::run(|term| {
        App {
            context: Context {
                exit: false,
                daemon: DaemonClient::new(),
            },
            view: MainView {
                file_tree: FileTree::new(match env::current_dir() {
                    Ok(path) => path,
                    Err(e) => env::home_dir().ok_or(e)?,
                })?,
            }
            .into(),
        }
        .run(term)
    })
}
