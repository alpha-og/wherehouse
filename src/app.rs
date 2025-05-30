use std::{
    fmt::Display,
    process::Command,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListItem, Paragraph},
};

use crate::{
    state::{self, Pane, State, StateEvent, StateItemType, StateResponse},
    tui,
};

pub struct Tui {
    tx_state: Sender<StateEvent>,
}

impl Tui {
    pub fn new(tx_state: Sender<StateEvent>) -> Self {
        Self { tx_state }
    }
    pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::Result<()> {
        loop {
            if State::sync_tui(&self.tx_state) {
                terminal.draw(|frame| self.draw(frame))?;
            };
            if let Some(StateResponse::ShouldQuit(should_quit)) =
                State::get(&self.tx_state, state::StateItemType::ShouldQuit)
            {
                if should_quit {
                    break;
                }
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

        let active_style = Style::new().bold().fg(Color::Red);
        let mut search_bar_style = Style::default();
        let mut list_style = Style::default();

        if let Some(StateResponse::CurrentPane(current_pane)) =
            State::get(&self.tx_state, StateItemType::CurrentPane)
        {
            match current_pane {
                Pane::SearchInput => search_bar_style = active_style,
                Pane::SearchResults => list_style = active_style,
                _ => {}
            }
        }

        let search_bar = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Search".bold())
            .title_alignment(ratatui::layout::Alignment::Left);
        if let Some(StateResponse::SearchQuery(search_query)) =
            State::get(&self.tx_state, StateItemType::SearchQuery)
        {
            let search_input = Paragraph::new(search_query)
                .left_aligned()
                .block(search_bar)
                .style(search_bar_style);
            frame.render_widget(search_input, splits[0]);
        }

        let active_list_item = Style::new().bold().bg(Color::White).fg(Color::Black);

        if let Some(StateResponse::SearchResults(Some(search_results))) =
            State::get(&self.tx_state, StateItemType::SearchResults)
        {
            if let Some(StateResponse::SelectedSearchResult(selected_search_result)) =
                State::get(&self.tx_state, StateItemType::SearchSelectedResult)
            {
                let list = List::new(search_results.iter().enumerate().map(|(i, item)| {
                    if i == selected_search_result {
                        ListItem::new(item.display_text.clone()).style(active_list_item)
                    } else {
                        ListItem::new(item.display_text.clone()).style(Style::default())
                    }
                }))
                .block(Block::bordered().border_type(BorderType::Rounded))
                .style(list_style);
                frame.render_widget(list, splits[1]);
            }
        } else {
            let list = List::new(vec![""])
                .block(Block::bordered().border_type(BorderType::Rounded))
                .style(list_style);
            frame.render_widget(list, splits[1]);
        }

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

        let status_bar_splits =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Fill(1)]).split(splits[2]);
        if let Some(StateResponse::InputMode(input_mode)) =
            State::get(&self.tx_state, StateItemType::InputMode)
        {
            let status_bar_left = Span::styled(
                format!(" {} ", input_mode),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(status_bar_left, status_bar_splits[0]);
        }
        let status_bar_right = Paragraph::new(format!("WhereHouse 0.1.0 | {homebrew_version}"))
            .right_aligned()
            .fg(Color::Green);
        frame.render_widget(status_bar_right, status_bar_splits[1]);
    }
}
