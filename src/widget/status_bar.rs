use std::sync::Arc;

use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::state::{InputMode, PackageManager, Pane, State};

pub struct StatusBar {
    state: Arc<State>,
}

impl Widget for StatusBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let input_mode = self.state.input_mode.lock().unwrap();
        let config = self.state.config.lock().unwrap();
        let current_pane = self.state.current_pane.lock().unwrap();
        let search_source = self.state.search.source.lock().unwrap();

        let left_text = match *current_pane {
            Pane::SearchInput | Pane::SearchResults => {
                format!(" {} | {} ", *input_mode, *search_source)
            }
        };
        let status_bar_layout =
            Layout::horizontal(vec![Constraint::Percentage(70), Constraint::Fill(1)]).split(area);
        let status_bar_left = Span::styled(
            left_text,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
        let status_bar_right = Paragraph::new(format!(
            " {} {} | {} {} ",
            config.app_name,
            config.app_version,
            config.package_manager,
            config.package_manager_version,
        ))
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
