use crate::state::{Pane, State};
use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
        let block_style = match self.state.current_pane() {
            Pane::SearchResults(_) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let search = self.state.search.lock().unwrap();
        let updatable_count = search.results.iter().filter(|r| r.update_available).count();
        let title = if search.updatable_only {
            format!("[2] Package (updates only)")
        } else if updatable_count > 0 {
            format!("[2] Package ({updatable_count} updates)")
        } else {
            "[2] Package".to_string()
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .title_alignment(HorizontalAlignment::Left)
            .style(block_style);
        let installed_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let not_installed_style = Style::default().fg(Color::DarkGray);
        let update_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let search_results = search
            .results
            .iter()
            .map(|result| {
                let (prefix, style) = if result.is_installed {
                    (" ●", installed_style)
                } else {
                    (" ○", not_installed_style)
                };
                let mut spans = vec![
                    Span::styled(prefix, style),
                    Span::raw(format!(" {}", result.name)),
                ];
                if result.update_available {
                    spans.push(Span::styled(" ↑", update_style));
                }
                ListItem::new(Line::from(spans)).style(style)
            })
            .collect::<Vec<ListItem>>();
        let is_focused = matches!(self.state.current_pane(), Pane::SearchResults(_));
        let selected_style = if is_focused {
            Style::default()
                .bg(Color::Rgb(200, 225, 255))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };
        let search_results = List::new(search_results)
            .block(block)
            .style(not_installed_style)
            .highlight_style(selected_style)
            .highlight_symbol("")
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
