use std::sync::Arc;

use ratatui::{
    layout::Alignment::Left,
    style::{Modifier, Style},
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
        let block_style = match *self.state.current_pane() {
            Pane::SearchInput => Style::default()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };
        let block = Block::bordered()
            .border_type(Rounded)
            .title("1")
            .title_alignment(Left)
            .style(block_style);

        let query_style = Style::default();
        let search = self.state.search.lock().unwrap();
        let query = Paragraph::new(search.query.clone())
            .left_aligned()
            .block(block)
            .style(query_style);
        query.render(area, buf);
    }
}

impl SearchInputPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
