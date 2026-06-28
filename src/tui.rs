use std::sync::Arc;

use ratatui::layout::{Constraint, Layout};

use crate::state::{InputMode, Pane, State};
use crate::widget::{
    about_pane::AboutPane, context_pane::ContextPane, search_input_pane::SearchInputPane,
    search_results_pane::SearchResultsPane, status_bar::StatusBar, toast::ToastWidget,
};

use ratatui::{
    Terminal,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io;

pub fn init() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(std::io::stdout()))
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = ratatui::restore();
        hook(panic_info);
    }));
}

pub fn restore() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn draw(state: &Arc<State>, frame: &mut ratatui::Frame) {
    state.clean_expired_toasts();

    let layout =
        Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());

    let main_layout =
        Layout::horizontal(vec![Constraint::Percentage(40), Constraint::Fill(1)]).split(layout[0]);

    let sidebar_layout = Layout::vertical(vec![
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(main_layout[0]);

    let about_pane = AboutPane::new(state.clone());
    frame.render_widget(about_pane, sidebar_layout[0]);

    let search_input_pane = SearchInputPane::new(state.clone());
    frame.render_widget(search_input_pane, sidebar_layout[1]);

    let search_results_pane = SearchResultsPane::new(state.clone());
    let mut list_state = state.search.lock().unwrap().list_state.clone();
    frame.render_stateful_widget(search_results_pane, sidebar_layout[2], &mut list_state);
    state.search.lock().unwrap().list_state = list_state;

    let info_pane = ContextPane::new(state.clone());
    frame.render_widget(info_pane, main_layout[1]);

    if let Some(toast) = state.current_toast() {
        frame.render_widget(ToastWidget::new(toast), layout[0]);
    }

    if matches!(state.current_pane(), Pane::SearchInput) && matches!(state.input_mode(), InputMode::Insert) {
        let search = state.search.lock().unwrap();
        let x = sidebar_layout[1].x + 1 + search.query.len() as u16;
        let y = sidebar_layout[1].y + 1;
        drop(search);
        let max_x = sidebar_layout[1].right().saturating_sub(1);
        frame.set_cursor_position((x.min(max_x), y));
    }

    let status_bar = StatusBar::new(state.clone());
    frame.render_widget(status_bar, layout[1]);
}
