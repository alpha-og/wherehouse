use ratatui::{
    layout::Alignment::Left,
    style::Style,
    text::Span,
    widgets::{Block, BorderType::Rounded, Paragraph, Widget},
};

pub struct SearchPane {
    id: u8,
    query: String,
    active: bool,
}

impl Widget for SearchPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block_style = if self.active {
            Style::default().fg(ratatui::style::Color::Magenta)
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
