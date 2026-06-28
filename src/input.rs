use std::sync::mpsc::Sender;
use std::{sync::Arc, time::Duration};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use wherehouse::package_manager::Command;

use crate::state::{Event, InputMode, Pane, State};

pub struct InputHandler {
    tx: Sender<Event>,
    state: Arc<State>,
    update: bool,
}

impl InputHandler {
    pub fn new(state: Arc<State>, tx: Sender<Event>) -> Self {
        Self {
            tx,
            state,
            update: false,
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
        if event::poll(Duration::from_millis(300))? {
            self.update = true;
            match event::read().unwrap() {
                event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_press(key_event)?;
                }
                _ => {}
            }
        } else {
            self.update_search()?;
        }
        Ok(())
    }
    fn handle_key_press(&mut self, key_event: event::KeyEvent) -> color_eyre::Result<()> {
        let state = self.state.clone();
        let input_mode = state.input_mode();
        if let InputMode::Normal = input_mode {
            match key_event.code {
                KeyCode::Char('1') => {
                    self.tx.send(Event::CommandIssued(Command::Config))?;
                }
                KeyCode::Char('2') => {
                    self.tx.send(Event::PaneFocused(Pane::SearchInput))?;
                }
                KeyCode::Char('3') => {
                    let info = state.search().selected_result_info.clone();
                    self.tx
                        .send(Event::PaneFocused(Pane::SearchResults(info)))?;
                }
                KeyCode::Char('q') => self.quit()?,
                _ => {}
            }
        }

        match state.current_pane() {
            Pane::About(_) => {
                if let InputMode::Normal = input_mode {
                    match key_event.code {
                        KeyCode::Char('C') => {
                            self.tx.send(Event::CommandIssued(Command::CheckHealth))?;
                        }
                        _ => {}
                    }
                }
            }
            Pane::SearchInput => match input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('i') => {
                        self.tx.send(Event::InputModeChanged(InputMode::Insert))?;
                    }
                    _ => {}
                },
                InputMode::Insert => match key_event.code {
                    KeyCode::Char(ch) => self.append_search_query(ch)?,
                    KeyCode::Backspace => {
                        self.pop_search_query()?;
                    }
                    KeyCode::Esc => {
                        self.tx.send(Event::InputModeChanged(InputMode::Normal))?;
                    }
                    _ => {}
                },
            },
            Pane::SearchResults(_) => match input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit()?,
                    KeyCode::Char('k') => self.select_previous_search_result()?,
                    KeyCode::Char('j') => self.select_next_search_result()?,
                    KeyCode::Char('I') => {
                        self.tx.send(Event::CommandIssued(Command::InstallPackage))?;
                    }
                    KeyCode::Char('X') => {
                        self.tx
                            .send(Event::CommandIssued(Command::UninstallPackage))?;
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn quit(&self) -> color_eyre::Result<()> {
        self.tx.send(Event::Quit)?;
        Ok(())
    }

    fn append_search_query(&mut self, ch: char) -> color_eyre::Result<()> {
        self.tx.send(Event::InsertChar(ch))?;
        Ok(())
    }

    fn pop_search_query(&mut self) -> color_eyre::Result<()> {
        self.tx.send(Event::DeleteChar)?;
        Ok(())
    }

    fn reset_selected_search_result(&self) -> color_eyre::Result<()> {
        let mut search = self.state.search.lock().unwrap();
        search.list_state.select(None);
        Ok(())
    }

    fn select_previous_search_result(&self) -> color_eyre::Result<()> {
        self.tx.send(Event::SelectionMoved(-1))?;
        self.tx.send(Event::CommandIssued(Command::PackageInfo))?;
        Ok(())
    }

    fn select_next_search_result(&self) -> color_eyre::Result<()> {
        self.tx.send(Event::SelectionMoved(1))?;
        self.tx.send(Event::CommandIssued(Command::PackageInfo))?;
        Ok(())
    }

    fn update_search(&mut self) -> color_eyre::Result<()> {
        if self.update == false {
            return Ok(());
        };
        self.update = false;
        if let Pane::SearchInput = self.state.current_pane() {
            if self.state.search().query.is_empty() {
                return Ok(());
            };
            self.tx.send(Event::CommandIssued(Command::FilterPackages))?;
        }
        Ok(())
    }
}
