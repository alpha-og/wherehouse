use crate::package_manager::Backend;
use crate::package_manager::manager::PackageManager;
use crate::state::{Pane, State};
use ratatui::{
    layout::{Constraint, HorizontalAlignment, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};
use std::sync::Arc;

fn pane_name(pane: &Pane) -> &'static str {
    match pane {
        Pane::About(_) => "About",
        Pane::SearchResults(_) => "Package",
        Pane::Context => "Info",
    }
}

fn wrap_lines(lines: Vec<Line<'static>>, max_width: usize) -> Vec<Line<'static>> {
    if max_width < 4 {
        return lines;
    }
    let mut result = Vec::new();
    for line in lines {
        let w = line.width() as usize;
        if w <= max_width {
            result.push(line);
            continue;
        }
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        let indent_len = text.len() - text.trim_start().len();
        let indent = &text[..indent_len];
        let body = &text[indent_len..];
        let base_style = line.spans.first().map(|s| s.style).unwrap_or_default();
        let mut remaining = body;
        let mut first = true;
        while !remaining.is_empty() {
            let avail = max_width.saturating_sub(indent_len);
            if remaining.len() <= avail {
                result.push(if first {
                    line
                } else {
                    Line::from(Span::styled(format!("{indent}{remaining}"), base_style))
                });
                break;
            }
            let break_at = remaining[..avail].rfind(' ').unwrap_or(avail);
            let chunk = &remaining[..break_at];
            result.push(Line::from(Span::styled(
                format!("{indent}{chunk}"),
                base_style,
            )));
            remaining = remaining[break_at..].trim_start();
            first = false;
        }
    }
    result
}

pub struct ContextPane {
    state: Arc<State>,
}

impl Widget for ContextPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let current_pane = self.state.current_pane();
        let block_style = match &current_pane {
            Pane::Context => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let display_pane = match &current_pane {
            Pane::Context => self.state.previous_pane.lock().unwrap().clone(),
            other => other.clone(),
        };
        let title = format!("[0] {}", pane_name(&display_pane));
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .title_alignment(HorizontalAlignment::Left)
            .style(block_style);
        let inner = block.inner(area);
        let content = match &display_pane {
            Pane::About(_) => {
                let cfg = self.state.config();
                let raw = cfg.system_config.clone();
                let name = cfg.app_name.clone();
                let ver = cfg.app_version.clone();
                let pm = Backend::package_manager_from_backend(cfg.backend.clone());
                drop(cfg);
                wrap_lines(pm.render_config(&raw, &name, &ver), inner.width as usize)
            }
            Pane::SearchResults(_) => {
                let cfg = self.state.config();
                let pm = Backend::package_manager_from_backend(cfg.backend.clone());
                drop(cfg);
                let raw = self.state.search().selected_result_info.clone();
                wrap_lines(pm.render_info(&raw), inner.width as usize)
            }
            _ => Vec::new(),
        };
        let total_lines = content.len();
        let mut scroll_lock = self.state.context_scroll.lock().unwrap();
        let visible = inner.height as usize;
        let max_scroll = total_lines.saturating_sub(visible);
        *scroll_lock = scroll_lock.min(max_scroll);
        let scroll = *scroll_lock;
        drop(scroll_lock);
        block.render(area, buf);
        if total_lines <= visible {
            let context = Paragraph::new(Text::from(content))
                .left_aligned()
                .style(Style::default().fg(Color::White));
            context.render(inner, buf);
            return;
        }
        let vert = Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).split(inner);
        let context = Paragraph::new(Text::from(content))
            .left_aligned()
            .scroll((scroll as u16, 0))
            .style(Style::default().fg(Color::White));
        context.render(vert[0], buf);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight).thumb_symbol("█");
        let mut scrollbar_state = ScrollbarState::new(max_scroll)
            .position(scroll)
            .viewport_content_length(visible);
        scrollbar.render(vert[1], buf, &mut scrollbar_state);
    }
}

impl ContextPane {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}
