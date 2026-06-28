use serde::Deserialize;
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

#[derive(Deserialize)]
struct BrewInfoOutput {
    formulae: Vec<FormulaInfo>,
    casks: Vec<CaskInfo>,
}

#[derive(Deserialize, Clone)]
struct FormulaInfo {
    name: String,
    desc: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    versions: Versions,
    installed: Vec<InstalledInfo>,
    linked_keg: Option<String>,
    outdated: Option<bool>,
    dependencies: Vec<String>,
    build_dependencies: Vec<String>,
    conflicts_with: Vec<String>,
    caveats: Option<String>,
    tap: Option<String>,
    deprecated: Option<bool>,
    deprecation_reason: Option<String>,
    disabled: Option<bool>,
    disable_reason: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Versions {
    stable: Option<String>,
}

#[derive(Deserialize, Clone)]
struct InstalledInfo {
    version: String,
}

#[derive(Deserialize, Clone)]
struct CaskInfo {
    token: String,
    name: Vec<String>,
    desc: Option<String>,
    homepage: Option<String>,
    version: Option<String>,
    installed: Option<String>,
    outdated: Option<bool>,
    url: Option<String>,
}

fn section_header(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {} ──", text),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn label_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {}: ", label),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}

fn dep_list(deps: &[String]) -> Line<'static> {
    if deps.is_empty() {
        return Line::from(Span::raw(""));
    }
    Line::from(Span::raw(format!("    {}", deps.join(", "))))
}

fn build_formula_lines(f: FormulaInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let mut title = format!("  {}", f.name);
    if let Some(ref ver) = f.versions.stable {
        title.push_str(&format!(" v{ver}"));
    }
    if !f.installed.is_empty() {
        title.push(' ');
        lines.push(Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[installed]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(ref desc) = f.desc {
        lines.push(Line::from(Span::styled(
            format!("  {desc}"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(ref hp) = f.homepage {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                hp.clone(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    lines.push(Line::from(""));

    let has_deps = !f.dependencies.is_empty() || !f.build_dependencies.is_empty();
    if has_deps {
        lines.push(section_header("Dependencies"));
        if !f.build_dependencies.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Build:",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(dep_list(&f.build_dependencies));
        }
        if !f.dependencies.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Runtime:",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(dep_list(&f.dependencies));
        }
        lines.push(Line::from(""));
    }

    lines.push(section_header("Details"));
    if let Some(ref lic) = f.license {
        lines.push(label_line("License", lic));
    }
    if let Some(ref tap) = f.tap {
        lines.push(label_line("Tap", tap));
    }
    if let Some(ref keg) = f.linked_keg {
        lines.push(label_line("Linked", keg));
    }
    if let Some(true) = f.outdated {
        lines.push(Line::from(Span::styled(
            "  Outdated",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(true) = f.deprecated {
        let reason = f.deprecation_reason.unwrap_or_else(|| "unknown".into());
        lines.push(Line::from(Span::styled(
            format!("  Deprecated: {reason}"),
            Style::default().fg(Color::Red),
        )));
    }
    if let Some(true) = f.disabled {
        let reason = f.disable_reason.unwrap_or_else(|| "unknown".into());
        lines.push(Line::from(Span::styled(
            format!("  Disabled: {reason}"),
            Style::default().fg(Color::Red),
        )));
    }
    if !f.conflicts_with.is_empty() {
        lines.push(label_line("Conflicts", &f.conflicts_with.join(", ")));
    }
    if let Some(ref msg) = f.caveats {
        lines.push(Line::from(""));
        lines.push(section_header("Caveats"));
        for line in msg.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    lines
}

fn build_cask_lines(c: CaskInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let display_name = c.name.first().map(|s| s.as_str()).unwrap_or(&c.token);

    let mut title = format!("  {display_name}");
    if let Some(ref ver) = c.version {
        title.push_str(&format!(" v{ver}"));
    }
    if c.installed.is_some() {
        title.push(' ');
        lines.push(Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[installed]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(ref desc) = c.desc {
        lines.push(Line::from(Span::styled(
            format!("  {desc}"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(ref hp) = c.homepage {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                hp.clone(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(section_header("Details"));
    if let Some(ref ver) = c.version {
        lines.push(label_line("Version", ver));
    }
    if let Some(ref installed) = c.installed {
        lines.push(label_line("Installed", installed));
    }
    if let Some(true) = c.outdated {
        lines.push(Line::from(Span::styled(
            "  Outdated",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(ref url) = c.url {
        lines.push(label_line("URL", url));
    }

    lines
}

fn render_config_info(raw: &str, backend: &str, app_name: &str, app_version: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        format!("  {app_name} v{app_version}"),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ─────────────────────",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    let kv: Vec<(String, String)> = raw
        .lines()
        .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .collect();

    lines.push(section_header("Backend"));
    lines.push(label_line("Name", backend));
    for (k, v) in &kv {
        match k.as_str() {
            "HOMEBREW_VERSION" => lines.push(label_line("Version", v)),
            "HOMEBREW_PREFIX" => lines.push(label_line("Prefix", v)),
            "HOMEBREW_CELLAR" => lines.push(label_line("Cellar", v)),
            "HOMEBREW_REPOSITORY" => lines.push(label_line("Repository", v)),
            _ => {}
        }
    }
    lines.push(Line::from(""));

    lines.push(section_header("System"));
    for (k, v) in &kv {
        match k.as_str() {
            "BUILD_ARCH" => lines.push(label_line("Architecture", v)),
            "HOMEBREW_SYSTEM" | "MACOS_VERSION" => {}
            "HOMEBREW_CC" => lines.push(label_line("C Compiler", v)),
            "HOMEBREW_CXX" => lines.push(label_line("C++ Compiler", v)),
            "HOMEBREW_SYSTEM_VERSION" => lines.push(label_line("OS Version", v)),
            "SHELL" => lines.push(label_line("Shell", v)),
            _ => {}
        }
    }
    let os_name = kv.iter().find(|(k, _)| k == "HOMEBREW_SYSTEM").map(|(_, v)| v.as_str()).unwrap_or("");
    let os_ver = kv.iter().find(|(k, _)| k == "HOMEBREW_SYSTEM_VERSION").map(|(_, v)| v.as_str()).unwrap_or("");
    if !os_name.is_empty() {
        lines.push(label_line("OS", &format!("{os_name} {os_ver}")));
    }
    lines.push(Line::from(""));

    lines.push(section_header("Settings"));
    for (k, v) in &kv {
        if k.starts_with("HOMEBREW_") {
            if matches!(k.as_str(), "HOMEBREW_VERSION" | "HOMEBREW_PREFIX" | "HOMEBREW_CELLAR"
                | "HOMEBREW_REPOSITORY" | "HOMEBREW_SHELLENV_PREFIX"
                | "HOMEBREW_CC" | "HOMEBREW_CXX" | "HOMEBREW_SYSTEM"
                | "HOMEBREW_SYSTEM_VERSION" | "HOMEBREW_BROWSER" | "HOMEBREW_EDITOR")
            {
                continue;
            }
            let display_key = k.strip_prefix("HOMEBREW_").unwrap_or(k).to_string();
            if v == "1" {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(display_key, Style::default().fg(Color::Cyan)),
                    Span::raw("  "),
                    Span::styled("✓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                lines.push(label_line(&display_key, v));
            }
        }
    }

    lines
}

fn render_info(info: &str) -> Vec<Line<'static>> {
    match serde_json::from_str::<BrewInfoOutput>(info) {
        Ok(output) => {
            if !output.formulae.is_empty() {
                build_formula_lines(output.formulae.into_iter().next().unwrap())
            } else if !output.casks.is_empty() {
                build_cask_lines(output.casks.into_iter().next().unwrap())
            } else {
                vec![Line::from(Span::raw(info.to_string()))]
            }
        }
        Err(_) => vec![Line::from(Span::raw(info.to_string()))],
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
        let base_style = line
            .spans
            .first()
            .map(|s| s.style)
            .unwrap_or_default();
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
            result.push(Line::from(Span::styled(format!("{indent}{chunk}"), base_style)));
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
        let block_style = match self.state.current_pane() {
            Pane::Context => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightBlue),
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(HorizontalAlignment::Left)
            .style(block_style);
        let inner = block.inner(area);
        let content = match self.state.current_pane() {
            Pane::About(_) => {
                let cfg = self.state.config();
                let raw = cfg.system_config.clone();
                let backend = cfg.backend.name();
                let name = cfg.app_name.clone();
                let ver = cfg.app_version.clone();
                drop(cfg);
                wrap_lines(render_config_info(&raw, backend, &name, &ver), inner.width as usize)
            }
            Pane::SearchResults(_) => {
                let raw = self.state.search().selected_result_info.clone();
                wrap_lines(render_info(&raw), inner.width as usize)
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
