use crate::{daemon::DaemonClient, ui::View};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

#[derive(Debug)]
pub struct App {
    pub context: Context,
    pub view: View,
}

#[derive(Debug)]
pub struct Context {
    pub exit: bool,
    pub daemon: DaemonClient,
}

impl App {
    pub fn run(&mut self, term: &mut ratatui::DefaultTerminal) -> eyre::Result<()> {
        while !self.context.exit {
            let mut last_draw_result = Ok(());
            term.draw(|frame| {
                last_draw_result = self.draw(frame);
            })?;
            last_draw_result?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) -> eyre::Result<()> {
        self.view.draw(frame)
    }

    fn handle_events(&mut self) -> eyre::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> eyre::Result<()> {
        self.view.handle_key_event(key_event, &mut self.context)
    }
}

impl Context {
    pub fn exit(&mut self) {
        self.exit = true;
    }
}
