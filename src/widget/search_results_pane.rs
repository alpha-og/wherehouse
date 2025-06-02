use std::sync::Arc;

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListItem, Widget},
};

use crate::state::{Pane, State};

pub struct SearchResultsPane {
    state: Arc<State>,
}

impl Widget for SearchResultsPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match *self.state.current_pane() {
            Pane::SearchResults => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("2")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let search = self.state.search.lock().unwrap();
        let search_results = search.results.iter().enumerate().map(|(i, item)| {
            if i == search.selected_result {
                ListItem::new(item.display_text.clone()).style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(item.display_text.clone()).style(Style::default().fg(Color::White))
            }
        });
        let search_results_style = Style::default().fg(Color::White);
        let search_results = List::new(search_results)
            .block(block)
            .style(search_results_style);
        search_results.render(area, buf);
    }
}

impl SearchResultsPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
