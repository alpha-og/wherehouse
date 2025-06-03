use crate::state::{Pane, State};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, HighlightSpacing, List, ListItem, ListState, StatefulWidget},
};
use std::sync::Arc;

pub struct SearchResultsPane {
    state: Arc<State>,
}

impl StatefulWidget for SearchResultsPane {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block_style = match *self.state.current_pane() {
            Pane::SearchResults => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("3")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let search = self.state.search.lock().unwrap();
        let search_results_style = Style::default().fg(Color::White);
        let search_results = search
            .results
            .iter()
            .map(|item| ListItem::new(item.display_text.clone()).style(search_results_style))
            .collect::<Vec<ListItem>>();
        let selected_style = Style::default()
            .bg(Color::White)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);
        let search_results = List::new(search_results)
            .block(block)
            .style(search_results_style)
            .highlight_style(selected_style)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        search_results.render(area, buf, state);
    }

    type State = ListState;
}

impl SearchResultsPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
