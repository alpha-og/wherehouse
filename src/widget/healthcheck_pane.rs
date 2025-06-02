use std::sync::Arc;

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::state::{Pane, State};

pub struct HealthcheckPane {
    state: Arc<State>,
}

impl Widget for HealthcheckPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = match *self.state.current_pane() {
            Pane::Info => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("3")
            .title_alignment(Alignment::Left)
            .style(block_style);
        let healthcheck_results = self.state.healthcheck_results.lock().unwrap();
        let healtcheck_style = Style::default().fg(Color::White);
        let healthcheck_results = Paragraph::new(healthcheck_results.clone())
            .left_aligned()
            .block(block)
            .style(healtcheck_style);
        healthcheck_results.render(area, buf);
    }
}

impl HealthcheckPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
