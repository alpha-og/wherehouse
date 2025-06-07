use std::{sync::Arc, time::Duration};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use wherehouse::package_manager::{Command, PackageLocality, PackageManager};

use crate::{
    state::{InputMode, Pane, State},
    task_manager::TaskManager, // trace_dbg,
};

pub struct InputHandler<T> {
    task_manager: TaskManager<T>,
    state: Arc<State>,
    update: bool,
}

impl<T: PackageManager + Send + Sync + 'static> InputHandler<T> {
    pub fn new(state: Arc<State>, task_manager: TaskManager<T>) -> Self {
        Self {
            task_manager,
            state,
            update: false,
        }
    }
    pub fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            self.capture_input()?;
            if self.state.exit() {
                break;
            }
        }
        Ok(())
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
                    self.task_manager.execute(Command::Config)?;
                    state.set_current_pane(Pane::About(state.about()));
                }
                KeyCode::Char('2') => {
                    state.set_current_pane(Pane::SearchInput);
                }
                KeyCode::Char('3') => {
                    state.set_current_pane(Pane::SearchResults(
                        (*state.search()).selected_result_info.clone(),
                    ));
                }
                KeyCode::Char('q') => self.quit()?,
                _ => {}
            }
        }

        match state.current_pane() {
            Pane::About(_) => {
                if let InputMode::Normal = input_mode {
                    match key_event.code {
                        // KeyCode::Char('I') => self.state.update_context(
                        //     self.state.config.lock().unwrap().system_config.clone(),
                        // ),
                        KeyCode::Char('C') => {
                            self.task_manager.execute(Command::CheckHealth)?;
                        }
                        _ => {}
                    }
                }
            }
            Pane::SearchInput => match input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('i') => state.set_input_mode(InputMode::Insert),
                    KeyCode::Char('l') => {
                        let mut search = state.search();
                        search.source = PackageLocality::Local;
                        self.task_manager.execute(Command::FilterPackages)?;
                    }
                    KeyCode::Char('r') => {
                        let mut search = state.search();
                        search.source = PackageLocality::Remote;
                        self.task_manager.execute(Command::FilterPackages)?;
                    }
                    _ => {}
                },
                InputMode::Insert => match key_event.code {
                    KeyCode::Char(ch) => self.append_search_query(ch)?,
                    KeyCode::Backspace => {
                        self.pop_search_query()?;
                    }
                    KeyCode::Esc => state.set_input_mode(InputMode::Normal),
                    _ => {}
                },
            },
            Pane::SearchResults(_) => match input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit()?,
                    KeyCode::Char('k') => self.select_previous_search_result()?,
                    KeyCode::Char('j') => self.select_next_search_result()?,
                    KeyCode::Char('I') => self.task_manager.execute(Command::InstallPackage)?,
                    KeyCode::Char('X') => self.task_manager.execute(Command::UninstallPackage)?,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn quit(&self) -> color_eyre::Result<()> {
        self.state.set_exit(true);
        Ok(())
    }

    fn append_search_query(&mut self, ch: char) -> color_eyre::Result<()> {
        self.reset_selected_search_result()?;
        if let Ok(mut search) = self.state.search.lock() {
            search.query.push(ch);
        }
        Ok(())
    }

    fn pop_search_query(&mut self) -> color_eyre::Result<()> {
        self.reset_selected_search_result()?;
        if let Ok(mut search) = self.state.search.lock() {
            search.query.pop();
        }
        Ok(())
    }

    fn reset_selected_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.list_state.select(None);
        }
        Ok(())
    }

    fn select_previous_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.list_state.select_previous();
        }
        self.task_manager.execute(Command::PackageInfo)?;
        Ok(())
    }

    fn select_next_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.list_state.select_next();
        }
        self.task_manager.execute(Command::PackageInfo)?;
        Ok(())
    }

    fn update_search(&mut self) -> color_eyre::Result<()> {
        if self.update == false {
            return Ok(());
        };
        self.update = false;
        if let Pane::SearchInput = self.state.current_pane() {
            if self.state.search().query.is_empty() {
                self.reset_selected_search_result()?;
                let mut search = self.state.search.lock().unwrap();
                search.results = Vec::default();
                return Ok(());
            };
            self.task_manager.execute(Command::FilterPackages)?;
        }
        Ok(())
    }
}
