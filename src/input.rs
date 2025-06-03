use std::{sync::Arc, time::Duration};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use tracing::info;

use crate::{
    commands::CommandType,
    state::{InputMode, Pane, SearchSource, State},
    task_manager::TaskManager, // trace_dbg,
};

pub struct InputHandler {
    task_manager: TaskManager,
    state: Arc<State>,
    update: bool,
}

impl InputHandler {
    pub fn new(state: Arc<State>, task_manager: TaskManager) -> Self {
        Self {
            task_manager,
            state,
            update: false,
        }
    }
    pub fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            self.capture_input()?;
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
                    break;
                }
            }
        }
        Ok(())
    }
    fn capture_input(&mut self) -> color_eyre::Result<()> {
        if event::poll(Duration::from_millis(300)).unwrap() {
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
        let mut current_pane = state.current_pane();
        let mut input_mode = state.input_mode.lock().unwrap();

        if let InputMode::Normal = *input_mode {
            match key_event.code {
                KeyCode::Char('1') => {
                    *current_pane = Pane::Info;
                    self.state
                        .update_context(self.state.config.lock().unwrap().system_config.clone());
                }
                KeyCode::Char('2') => {
                    *current_pane = Pane::SearchInput;
                    self.state.update_context(String::default());
                }
                KeyCode::Char('3') => {
                    *current_pane = Pane::SearchResults;
                    self.state.update_context(
                        self.state
                            .search
                            .lock()
                            .unwrap()
                            .selected_result_info
                            .clone(),
                    );
                }
                KeyCode::Char('q') => self.quit()?,
                _ => {}
            }
        }

        match *current_pane {
            Pane::Info => {
                if let InputMode::Normal = *input_mode {
                    match key_event.code {
                        KeyCode::Char('I') => self.state.update_context(
                            self.state.config.lock().unwrap().system_config.clone(),
                        ),
                        KeyCode::Char('C') => {
                            self.task_manager.execute(CommandType::Healthcheck, true)?;
                        }
                        _ => {}
                    }
                }
            }
            Pane::SearchInput => match *input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('i') => *input_mode = InputMode::Insert,
                    KeyCode::Char('l') => {
                        let mut search = self.state.search.lock().unwrap();
                        search.source = SearchSource::Local;
                        self.task_manager.execute(CommandType::Search, true)?;
                    }
                    KeyCode::Char('r') => {
                        let mut search = self.state.search.lock().unwrap();
                        search.source = SearchSource::Remote;
                        self.task_manager.execute(CommandType::Search, true)?;
                    }
                    _ => {}
                },
                InputMode::Insert => match key_event.code {
                    KeyCode::Char(ch) => self.append_search_query(ch)?,
                    KeyCode::Backspace => {
                        self.pop_search_query()?;
                    }
                    KeyCode::Esc => *input_mode = InputMode::Normal,
                    _ => {}
                },
                _ => {}
            },
            Pane::SearchResults => match *input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit()?,
                    KeyCode::Char('k') => self.select_previous_search_result()?,
                    KeyCode::Char('j') => self.select_next_search_result()?,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn quit(&self) -> color_eyre::Result<()> {
        if let Ok(mut should_quit) = self.state.should_quit.lock() {
            *should_quit = true;
        }
        Ok(())
    }

    fn append_search_query(&mut self, ch: char) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.query.push(ch);
        }
        self.reset_selected_search_result()?;
        Ok(())
    }

    fn pop_search_query(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.query.pop();
        }
        self.reset_selected_search_result()?;
        Ok(())
    }

    fn reset_selected_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.selected_result = 0;
            search.list_state.select(None);
        }
        self.task_manager.execute(CommandType::Info, true)?;
        Ok(())
    }

    fn select_previous_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            search.selected_result = search.selected_result.saturating_sub(1);
            search.list_state.select_previous();
        }
        self.task_manager.execute(CommandType::Info, true)?;
        Ok(())
    }

    fn select_next_search_result(&mut self) -> color_eyre::Result<()> {
        if let Ok(mut search) = self.state.search.lock() {
            if search.results.len() == 0 {
                return Ok(());
            }
            search.selected_result =
                search.selected_result.saturating_add(1) % search.results.len();
            search.list_state.select_next();
        }
        self.task_manager.execute(CommandType::Info, true)?;
        Ok(())
    }

    fn update_search(&mut self) -> color_eyre::Result<()> {
        if self.update == false {
            return Ok(());
        };
        self.update = false;
        let current_pane = self.state.current_pane();
        if let Pane::SearchInput = *current_pane {
            drop(current_pane);
            self.task_manager.execute(CommandType::Search, true)?;
        }
        Ok(())
    }
}
