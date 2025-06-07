use std::sync::Arc;

use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::state::{Pane, State};

pub struct StatusBar {
    state: Arc<State>,
}

impl Widget for StatusBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let config = self.state.config.lock().unwrap();
        let search = self.state.search.lock().unwrap();

        let left_text = match self.state.current_pane() {
            Pane::SearchInput | Pane::SearchResults(_) => {
                format!(" {} | {} ", self.state.input_mode(), search.source)
            }
            _ => format!(" {} ", self.state.input_mode()),
        };
        let status_bar_layout =
            Layout::horizontal(vec![Constraint::Percentage(70), Constraint::Fill(1)]).split(area);
        let status_bar_left = Span::styled(
            left_text,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
        let status_bar_right =
            Paragraph::new(format!(" {} {}", config.app_name, config.app_version,))
                .right_aligned()
                .fg(Color::Green);
        status_bar_left.render(status_bar_layout[0], buf);
        status_bar_right.render(status_bar_layout[1], buf);
    }
}

impl StatusBar {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
