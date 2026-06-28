use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::state::{Toast, ToastType};

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub struct ToastWidget {
    toast: Toast,
}

impl ToastWidget {
    pub fn new(toast: Toast) -> Self {
        Self { toast }
    }
}

impl Widget for ToastWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let (prefix, color) = match self.toast.toast_type {
            ToastType::Success => (" ✓ ".to_string(), Color::Green),
            ToastType::Error => (" ✗ ".to_string(), Color::LightRed),
            ToastType::Info => (" i ".to_string(), Color::White),
            ToastType::Progress => {
                let frame = SPINNER[(self.toast.created_at.elapsed().as_millis() as usize / 120) % SPINNER.len()];
                (format!(" {} ", frame), Color::Cyan)
            }
        };

        let message = format!("{}{}", prefix, self.toast.message);

        let width = (message.len() as u16 + 4)
            .min(area.width.saturating_sub(2))
            .max(20);
        let height = 3;
        let x = area.right().saturating_sub(width + 1);
        let toast_area = Rect::new(x, area.y + 1, width, height);

        if toast_area.right() > area.right() || toast_area.bottom() > area.bottom() {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(color));

        Clear.render(toast_area, buf);
        Paragraph::new(message).block(block).render(toast_area, buf);
    }
}
