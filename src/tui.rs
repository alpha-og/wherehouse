use std::sync::Arc;

use ratatui::layout::{Constraint, Layout};

use crate::{
    state::State,
    widget::{
        info_pane::InfoPane, search_input_pane::SearchInputPane,
        search_results_pane::SearchResultsPane, status_bar::StatusBar,
    },
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

pub struct Tui {
    state: Arc<State>,
}

impl Tui {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut ratatui::Frame) {
        let layout =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());

        let main_layout = Layout::horizontal(vec![Constraint::Percentage(40), Constraint::Fill(1)])
            .split(layout[0]);

        let search_layout = Layout::vertical(vec![Constraint::Length(3), Constraint::Fill(1)])
            .split(main_layout[0]);

        let search_input_pane = SearchInputPane::new(self.state.clone());
        frame.render_widget(search_input_pane, search_layout[0]);

        let search_results_pane = SearchResultsPane::new(self.state.clone());
        frame.render_widget(search_results_pane, search_layout[1]);

        let info_pane = InfoPane::new(self.state.clone());
        frame.render_widget(info_pane, main_layout[1]);

        let status_bar = StatusBar::new(self.state.clone());
        frame.render_widget(status_bar, layout[1]);
    }
}
