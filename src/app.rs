use color_eyre::eyre::Context;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

use crate::tui;

pub struct App {
    quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self { quit: false }
    }
}

impl ratatui::widgets::Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" WhereHouse ".bold());
        Block::bordered()
            .title(title.centered())
            .border_set(border::THICK)
            .render(area, buf);
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
        frame.render_widget(self, frame.area());
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
