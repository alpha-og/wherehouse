use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListItem, Widget},
};

use crate::state::SearchResults;

pub struct SearchResultsPane {
    results: SearchResults,
    selected: usize,
    active: bool,
}

impl Widget for SearchResultsPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = if self.active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("2")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let search_results = self.results.iter().enumerate().map(|(i, item)| {
            if i == self.selected {
                ListItem::new(item.display_text.clone()).style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(item.display_text.clone()).style(Style::default())
            }
        });
        let search_results_style = Style::default();
        let search_results = List::new(search_results)
            .block(block)
            .style(search_results_style);
        search_results.render(area, buf);
    }
}

impl Default for SearchResultsPane {
    fn default() -> Self {
        Self {
            results: SearchResults::default(),
            selected: usize::default(),
            active: bool::default(),
        }
    }
}

impl SearchResultsPane {
    pub fn results(&mut self, results: SearchResults) {
        self.results = results;
    }
    pub fn select(&mut self, selected_result: usize) {
        self.selected = selected_result;
    }
    pub fn active(&mut self) {
        self.active = true;
    }
}
