use crate::{elm, ui::Screen};
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::Widget,
};
use std::borrow::Cow;

const TABS: [&str; 3] = ["1 Library", "2 Now Playing", "3 Queue"];
const ACTIVE_TAB: usize = 1;
const TITLE: &str = "Low Tide Signal";
const ARTIST: &str = "Glass Harbor";
const ALBUM: &str = "Paper Lanterns · 2024";
const ELAPSED_SECS: usize = 108;
const DURATION_SECS: usize = 222;
const VOLUME: u8 = 72;
const NEXT: &str = "02  Ferry Lights";

const COVER: [&str; 12] = [
    "░░░░░░░░░░░░░░░░░░░░░░░░░░",
    "░░░░░░░░░░░░░░░░░░░░░░░░░░",
    "░░░░░░░░░░▒▒▒▒▒▒░░░░░░░░░░",
    "░░░░░░░▒▒▒▓▓▓▓▓▓▒▒▒░░░░░░░",
    "░░░░░░▒▒▓▓▓████▓▓▓▒▒░░░░░░",
    "░░░░░░▒▒▓▓▓████▓▓▓▒▒░░░░░░",
    "░░░░░░░▒▒▒▓▓▓▓▓▓▒▒▒░░░░░░░",
    "▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒",
    "▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓",
    "▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓",
    "██████████▓▓▓▓▓▓██████████",
    "██████████████████████████",
];

const KEYS_FULL: [(&str, &str); 6] = [
    ("/", "find"),
    ("tab", "views"),
    ("space", "pause"),
    ("[ ]", "seek"),
    ("+ −", "volume"),
    ("q", "quit"),
];
const KEYS_SHORT: [(&str, &str); 3] = [("/", "find"), ("space", "pause"), ("q", "quit")];

const BAR_WIDTH: usize = 44;
const MIN_HEIGHT_FOR_COVER: u16 = 26;
const FOOTER_GAP: usize = 2;

#[derive(Debug, Default)]
pub struct FocusScreen;

impl elm::View for FocusScreen {
    fn view(&self, area: Rect, buffer: &mut Buffer) {
        let [top, stage, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .areas(area.inner(Margin::new(2, 0)));
        render_top_bar(top, buffer);
        render_stage(stage, buffer, area.height >= MIN_HEIGHT_FOR_COVER);
        render_footer(footer, buffer);
    }

    fn update(&mut self, msg: &elm::Message) -> eyre::Result<elm::Command> {
        match msg {
            elm::Message::Key(k) if k.code == KeyCode::Char('1') => {
                Ok(elm::Command::ShowMainScreen)
            }
            _ => Ok(elm::Command::None),
        }
    }
}

impl From<FocusScreen> for Screen {
    fn from(screen: FocusScreen) -> Screen {
        Screen::Focus(screen)
    }
}

fn render_top_bar(area: Rect, buf: &mut Buffer) {
    let mut tabs = Vec::new();
    for (i, &tab) in TABS.iter().enumerate() {
        if i > 0 {
            tabs.push(Span::raw("   "));
        }
        tabs.push(if i == ACTIVE_TAB {
            tab.bold().underlined()
        } else {
            tab.dim()
        });
    }
    Line::from(tabs).render(area, buf);
    Line::from(vec![
        "hum".bold(),
        Span::raw("  "),
        format!("·  vol {VOLUME}%").dim(),
    ])
    .right_aligned()
    .render(area, buf);
}

fn render_stage(area: Rect, buf: &mut Buffer, show_cover: bool) {
    let [info_area, progress_area] = if show_cover {
        let [cover, info, progress] = Layout::vertical([
            Constraint::Length(COVER.len() as u16),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .spacing(1)
        .flex(Flex::Center)
        .areas(area);
        Text::from(COVER.map(Line::raw).to_vec())
            .centered()
            .render(cover, buf);
        [info, progress]
    } else {
        Layout::vertical([Constraint::Length(3), Constraint::Length(1)])
            .spacing(1)
            .flex(Flex::Center)
            .areas(area)
    };

    let width = area.width as usize;
    Text::from(vec![
        Line::from(truncate(TITLE, width).bold()),
        Line::raw(truncate(ARTIST, width)),
        Line::from(truncate(ALBUM, width).dim()),
    ])
    .centered()
    .render(info_area, buf);
    progress_line().render(progress_area, buf);
}

fn progress_line() -> Line<'static> {
    let filled = (ELAPSED_SECS * (BAR_WIDTH - 1) + DURATION_SECS / 2) / DURATION_SECS;
    Line::from(vec![
        format_time(ELAPSED_SECS).dim(),
        Span::raw("  "),
        "━".repeat(filled).bold(),
        "●".bold(),
        "─".repeat(BAR_WIDTH - 1 - filled).dim(),
        Span::raw("  "),
        format_time(DURATION_SECS).dim(),
    ])
    .centered()
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    let width = area.width as usize;
    let next = Line::from(vec!["next".dim(), Span::raw("  "), Span::raw(NEXT)]);
    let full = key_hints(&KEYS_FULL);
    if next.width() + FOOTER_GAP + full.width() <= width {
        next.render(area, buf);
        full.right_aligned().render(area, buf);
    } else if full.width() <= width {
        full.right_aligned().render(area, buf);
    } else {
        key_hints(&KEYS_SHORT).right_aligned().render(area, buf);
    }
}

fn key_hints(keys: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, &(key, label)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.extend([key.bold(), Span::raw(" "), label.dim()]);
    }
    Line::from(spans)
}

fn format_time(secs: usize) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn truncate(s: &str, max: usize) -> Cow<'_, str> {
    let width = |s: &str| Span::raw(s).width();
    if width(s) <= max {
        return Cow::Borrowed(s);
    }
    let end = s
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|&i| width(&s[..i]) < max)
        .last()
        .unwrap_or(0);
    Cow::Owned(format!("{}…", &s[..end]))
}
