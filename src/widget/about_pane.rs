use std::sync::Arc;

use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::state::{Pane, State};

pub struct AboutPane {
    state: Arc<State>,
}

impl Widget for AboutPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match self.state.current_pane() {
            Pane::About(_) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[1] Package Manager")
            .title_alignment(HorizontalAlignment::Left)
            .style(block_style);

        let backend = self.state.current_backend();
        let count = self.state.available_backends.len();
        let idx = self.state.current_backend_index();
        let label = if count > 1 {
            format!("{}  [< {} of {} >]", backend.name(), idx + 1, count)
        } else {
            backend.name().to_string()
        };

        let info = Paragraph::new(Span::raw(label))
            .left_aligned()
            .block(block)
            .style(Style::default().fg(Color::White));
        info.render(area, buf);
    }
}

impl AboutPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
