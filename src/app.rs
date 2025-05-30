use std::{process::Command, sync::Arc, thread, time::Duration};

use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListItem, Paragraph},
};

use crate::{
    state::{Pane, State},
    tui,
};

pub struct Tui {
    state: Arc<State>,
}

impl Tui {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
    pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
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

        if let Ok(current_pane) = self.state.current_pane.lock() {
            match *current_pane {
                Pane::SearchInput => search_bar_style = active_style,
                Pane::SearchResults => list_style = active_style,
                _ => {}
            }
        }

        let search_bar = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Search".bold())
            .title_alignment(ratatui::layout::Alignment::Left);
        if let Ok(search_query) = self.state.search.query.lock() {
            let search_input = Paragraph::new(search_query.clone())
                .left_aligned()
                .block(search_bar)
                .style(search_bar_style);
            frame.render_widget(search_input, splits[0]);
        }
        if let Ok(search_results) = self.state.search.results.lock() {
            let active_list_item = Style::new().bold().bg(Color::White).fg(Color::Black);

            if let Ok(selected_search_result) = self.state.search.selected_result.lock() {
                let list = List::new(search_results.iter().enumerate().map(|(i, item)| {
                    if i == *selected_search_result {
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
        if let Ok(input_mode) = self.state.input_mode.lock() {
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
