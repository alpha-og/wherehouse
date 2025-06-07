use crate::state::{Pane, State};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::sync::Arc;

pub struct ContextPane {
    state: Arc<State>,
}

impl Widget for ContextPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match self.state.current_pane() {
            Pane::Context => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            // .title("4")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let context = match self.state.current_pane() {
            Pane::About(context) => context,
            Pane::SearchResults(context) => context,
            _ => String::default(),
        };
        let context_style = Style::default().fg(Color::White);
        let context = Paragraph::new(context)
            .left_aligned()
            .block(block)
            .style(context_style);
        context.render(area, buf);
    }
}

impl ContextPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
