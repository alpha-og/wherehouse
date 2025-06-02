use std::{process::Command, sync::Arc};

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::state::{Pane, State};

pub struct InfoPane {
    state: Arc<State>,
}

impl Widget for InfoPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match *self.state.current_pane() {
            Pane::Info => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            _ => Style::default(),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            // .title("4")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let search = self.state.search.lock().unwrap();
        let info_style = Style::default();
        let info = Paragraph::new(search.selected_result_info.clone())
            .left_aligned()
            .block(block)
            .style(info_style);
        info.render(area, buf);
    }
}

impl InfoPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
