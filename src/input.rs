use std::sync::mpsc::Sender;
use std::time::Instant;
use std::{sync::Arc, time::Duration};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use wherehouse::package_manager::Command;

use crate::state::{Event, Pane, State};

const PACKAGE_INFO_DEBOUNCE: Duration = Duration::from_millis(150);

pub struct InputHandler {
    tx: Sender<Event>,
    state: Arc<State>,
    last_package_request: Option<Instant>,
}

impl InputHandler {
    pub fn new(state: Arc<State>, tx: Sender<Event>) -> Self {
        Self {
            tx,
            state,
            last_package_request: None,
        }
    }
    pub fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            self.capture_input()?;
            if self.state.exit() {
                break Ok(());
            }
        }
    }
    fn capture_input(&mut self) -> color_eyre::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read().unwrap() {
                event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_press(key_event)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn handle_key_press(&mut self, key_event: event::KeyEvent) -> color_eyre::Result<()> {
        let current_pane = self.state.current_pane();
        let search_active = self.state.search.lock().unwrap().search_active;

        match key_event.code {
            KeyCode::Char('/') => {
                let mut search = self.state.search.lock().unwrap();
                search.search_active = true;
                search.query.clear();
                search.query_last_changed = std::time::Instant::now();
            }
            KeyCode::Enter if search_active => {
                let mut search = self.state.search.lock().unwrap();
                search.search_active = false;
            }
            KeyCode::Esc => {
                let mut search = self.state.search.lock().unwrap();
                if search.search_active {
                    search.search_active = false;
                } else if matches!(current_pane, Pane::SearchResults(_)) && !search.query.is_empty() {
                    search.query.clear();
                    search.query_last_changed = std::time::Instant::now();
                }
            }
            KeyCode::Char(ch) if search_active => {
                self.tx.send(Event::InsertChar(ch))?;
            }
            KeyCode::Backspace if search_active => {
                self.tx.send(Event::DeleteChar)?;
            }
            KeyCode::Char('1') => {
                self.tx.send(Event::CommandIssued(Command::Config))?;
            }
            KeyCode::Char('0') => {
                self.tx.send(Event::PaneFocused(Pane::Context))?;
            }
            KeyCode::Char('2') => {
                let info = self.state.search().selected_result_info.clone();
                self.tx.send(Event::PaneFocused(Pane::SearchResults(info)))?;
            }
            KeyCode::Char('q') => self.quit()?,
            KeyCode::Char('C') if matches!(current_pane, Pane::About(_)) => {
                self.tx.send(Event::CommandIssued(Command::CheckHealth))?;
            }
            KeyCode::Char('[') if matches!(current_pane, Pane::About(_)) => {
                self.tx.send(Event::SwitchBackend(-1))?;
            }
            KeyCode::Char(']') if matches!(current_pane, Pane::About(_)) => {
                self.tx.send(Event::SwitchBackend(1))?;
            }
            KeyCode::Char('k') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.select_previous_search_result()?;
            }
            KeyCode::Char('j') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.select_next_search_result()?;
            }
            KeyCode::Char('I') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.tx.send(Event::CommandIssued(Command::InstallPackage))?;
            }
            KeyCode::Char('X') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.tx.send(Event::CommandIssued(Command::UninstallPackage))?;
            }
            KeyCode::Char('u') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.tx.send(Event::CommandIssued(Command::UpdatePackage))?;
            }
            KeyCode::Char('U') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.tx.send(Event::CommandIssued(Command::UpdateAll))?;
            }
            KeyCode::Char('f') if matches!(current_pane, Pane::SearchResults(_)) => {
                self.tx.send(Event::ToggleUpdatableFilter)?;
            }
            _ if key_event.modifiers == KeyModifiers::CONTROL => match key_event.code {
                KeyCode::Char('d') => self.tx.send(Event::ContextScroll(8))?,
                KeyCode::Char('u') => self.tx.send(Event::ContextScroll(-8))?,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn quit(&self) -> color_eyre::Result<()> {
        self.tx.send(Event::Quit)?;
        Ok(())
    }

    fn request_package_info(&mut self) -> color_eyre::Result<()> {
        if self.last_package_request.is_some_and(|t| t.elapsed() < PACKAGE_INFO_DEBOUNCE) {
            return Ok(());
        }
        self.last_package_request = Some(Instant::now());
        self.tx.send(Event::CommandIssued(Command::PackageInfo))?;
        Ok(())
    }

    fn select_previous_search_result(&mut self) -> color_eyre::Result<()> {
        let search = self.state.search.lock().unwrap();
        let can_move = search.list_state.selected().map_or(false, |s| s > 0);
        let len = search.results.len();
        drop(search);
        self.tx.send(Event::SelectionMoved(-1))?;
        if can_move && len > 0 {
            self.request_package_info()?;
        }
        Ok(())
    }

    fn select_next_search_result(&mut self) -> color_eyre::Result<()> {
        let search = self.state.search.lock().unwrap();
        let can_move = match search.list_state.selected() {
            Some(s) => s + 1 < search.results.len(),
            None => false,
        };
        let len = search.results.len();
        drop(search);
        self.tx.send(Event::SelectionMoved(1))?;
        if can_move && len > 0 {
            self.request_package_info()?;
        }
        Ok(())
    }
}
