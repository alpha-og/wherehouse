use color_eyre::eyre::Context;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, Paragraph},
};

use crate::tui;

enum Pane {
    Search,
    List,
}

enum InputMode {
    Normal,
    Insert,
}

pub struct App {
    current_pane: Pane,
    search_query: String,
    input_mode: InputMode,
    quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_pane: Pane::Search,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            quit: false,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut ratatui::Frame) {
        let splits = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

        let search_bar = Block::bordered()
            .title("Search".bold())
            .title_alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(search_bar, splits[0]);

        let list = List::new(vec!["hello", "world", "ratatui"]).block(Block::bordered());
        frame.render_widget(list, splits[1]);

        let status_bar = Paragraph::new("WhereHouse 0.1.0")
            .alignment(ratatui::layout::Alignment::Right)
            .block(Block::bordered());
        frame.render_widget(status_bar, splits[2]);
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.quit(),

            _ => {}
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.quit = true;
    }
}
