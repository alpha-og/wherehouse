use std::sync::Arc;

use ratatui::{
    layout::HorizontalAlignment::Left,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType::Rounded, Paragraph, Widget},
};

use crate::state::{Pane, State};

pub struct SearchInputPane {
    state: Arc<State>,
}

impl Widget for SearchInputPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match self.state.current_pane() {
            Pane::SearchInput => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(Rounded)
            .title("2")
            .title_alignment(Left)
            .style(block_style);

        let query_style = Style::default().fg(Color::White);
        let search = self.state.search.lock().unwrap();
        let display = if search.query.is_empty() {
            Paragraph::new("Search packages...")
                .left_aligned()
                .block(block)
                .style(Style::default().fg(Color::DarkGray))
        } else {
            Paragraph::new(search.query.clone())
                .left_aligned()
                .block(block)
                .style(query_style)
        };
        display.render(area, buf);
    }
}

impl SearchInputPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
