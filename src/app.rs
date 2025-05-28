use std::{process::Command, sync::mpsc, thread, time::Duration};

use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::{
    input, tui,
    worker::{self, WorkerEvent},
};

enum Pane {
    Search,
    List,
}

enum InputMode {
    Normal,
    Insert,
}

pub enum TuiEvent {
    KeyInput(KeyEvent),
    SearchResult(SearchResults),
}

pub struct App {
    current_pane: Pane,
    search_query: String,
    search_results: SearchResults,
    input_mode: InputMode,
    quit: bool,
}

pub struct SearchResults {
    pub selected_index: usize,
    pub results: Vec<String>,
}

impl Default for SearchResults {
    fn default() -> Self {
        Self {
            selected_index: 0,
            results: Vec::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_pane: Pane::Search,
            search_query: String::new(),
            search_results: SearchResults::default(),
            input_mode: InputMode::Normal,
            quit: false,
        }
    }
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut tui::Tui,
        rx: mpsc::Receiver<TuiEvent>,
        tx_worker: mpsc::Sender<worker::WorkerEvent>,
        _tx_input: mpsc::Sender<input::InputEvent>,
    ) -> color_eyre::Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;
            match rx.recv()? {
                TuiEvent::KeyInput(key_input) => self
                    .handle_key_event(key_input, tx_worker.clone())
                    .wrap_err_with(|| format!("handling key event failed:\n{key_input:#?}"))?,
                TuiEvent::SearchResult(search_results) => self.search_results = search_results,
            }
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut ratatui::Frame) {
        let splits = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let mut active_style = Style::new().bold().fg(Color::Red);
        let mut search_bar_style = Style::default();
        let mut list_style = Style::default();

        match self.current_pane {
            Pane::Search => search_bar_style = active_style,
            Pane::List => list_style = active_style,
            _ => {}
        }

        let search_bar = Block::bordered()
            .title("Search".bold())
            .title_alignment(ratatui::layout::Alignment::Left);
        let search_input = Paragraph::new(self.search_query.clone())
            .left_aligned()
            .block(search_bar)
            .style(search_bar_style);
        frame.render_widget(search_input, splits[0]);

        let active_list_item = Style::new().bold().fg(Color::Magenta);

        let list = List::new(
            self.search_results
                .results
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    if i == self.search_results.selected_index {
                        ListItem::new(item.clone()).style(active_list_item)
                    } else {
                        ListItem::new(item.clone()).style(Style::default())
                    }
                }),
        )
        .block(Block::bordered())
        .style(list_style);
        frame.render_widget(list, splits[1]);

        let homebrew_version = String::from_utf8(
            Command::new("brew")
                .arg("--version")
                .output()
                .expect("failed to execute shell command brew --version")
                .stdout,
        )
        .expect("failed to parse Vec<u8> as String");
        let homebrew_version = homebrew_version
            .strip_suffix("\n")
            .expect("failed to strip newline");

        let status_bar =
            Paragraph::new(format!("WhereHouse 0.1.0 | {homebrew_version}")).right_aligned();
        frame.render_widget(status_bar, splits[2]);
    }

    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        tx_worker: mpsc::Sender<WorkerEvent>,
    ) -> color_eyre::Result<()> {
        match self.current_pane {
            Pane::Search => match self.input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('i') => self.input_mode = InputMode::Insert,
                    KeyCode::Tab => self.current_pane = Pane::List,
                    _ => {}
                },
                InputMode::Insert => match key_event.code {
                    KeyCode::Char(ch) => {
                        self.search_query.push(ch);
                    }
                    KeyCode::Backspace => {
                        self.search_query.pop();
                    }
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    KeyCode::Enter => {
                        tx_worker
                            .send(WorkerEvent::Search(self.search_query.clone()))
                            .unwrap();
                    }

                    _ => {}
                },
            },
            Pane::List => match self.input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('k') => self.select_previous_search_result(),
                    KeyCode::Char('j') => self.select_next_search_result(),
                    KeyCode::Tab => self.current_pane = Pane::Search,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn select_previous_search_result(&mut self) {
        self.search_results.selected_index = self.search_results.selected_index.saturating_sub(1);
    }
    fn select_next_search_result(&mut self) {
        self.search_results.selected_index = self.search_results.selected_index.saturating_add(1);
    }

    fn quit(&mut self) {
        self.quit = true;
    }
}
