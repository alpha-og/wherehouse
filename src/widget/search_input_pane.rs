use ratatui::{
    layout::Alignment::Left,
    style::{Modifier, Style},
    widgets::{Block, BorderType::Rounded, Paragraph, Widget},
};

pub struct SearchInputPane {
    query: String,
    active: bool,
}

impl Widget for SearchInputPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = if self.active {
            Style::default()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let block = Block::bordered()
            .border_type(Rounded)
            .title("1")
            .title_alignment(Left)
            .style(block_style);

        let query_style = Style::default();
        let query = Paragraph::new(self.query)
            .left_aligned()
            .block(block)
            .style(query_style);
        query.render(area, buf);
    }
}

impl Default for SearchInputPane {
    fn default() -> Self {
        Self {
            query: String::default(),
            active: false,
        }
    }
}

impl SearchInputPane {
    pub fn query(&mut self, query: String) {
        self.query = query;
    }
    pub fn active(&mut self) {
        self.active = true;
    }
}
