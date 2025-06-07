use std::sync::Arc;

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
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
            .title("1")
            .title_alignment(Alignment::Left)
            .style(block_style);

        let info_style = Style::default().fg(Color::White);
        let info = Paragraph::new(format!("{}", "Homebrew"))
            .left_aligned()
            .block(block)
            .style(info_style);
        info.render(area, buf);
    }
}

impl AboutPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
