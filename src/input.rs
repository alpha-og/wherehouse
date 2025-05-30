use std::sync::{Arc, mpsc::Sender};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};

use crate::state::{self, InputMode, Pane, State};

pub struct InputHandler {
    state: Arc<State>,
}

impl InputHandler {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
    pub fn run(&mut self) {
        loop {
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
                    break;
                } else {
                    self.capture_input();
                }
            }
        }
    }
    fn capture_input(&self) {
        match event::read().unwrap() {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_press(key_event);
            }
            _ => {}
        };
    }

    fn handle_key_press(&self, key_event: event::KeyEvent) {
        if let Ok(current_pane) = self.state.current_pane.lock() {
            if let Ok(input_mode) = self.state.input_mode.lock() {
                match *current_pane {
                    Pane::SearchInput => match *input_mode {
                        InputMode::Normal => match key_event.code {
                            KeyCode::Char('q') => self.quit(),
                            KeyCode::Char('i') => self.switch_input_mode(InputMode::Insert),
                            KeyCode::Tab => self.switch_pane(Pane::SearchResults),
                            _ => {}
                        },
                        InputMode::Insert => match key_event.code {
                            KeyCode::Char(ch) => self.append_search_query(ch),
                            KeyCode::Backspace => {
                                self.pop_search_query();
                            }
                            KeyCode::Esc => self.switch_input_mode(InputMode::Normal),
                            // KeyCode::Enter => {
                            //     tx_worker
                            //         .send(WorkerEvent::Search(self.search_query.clone()))
                            //         .unwrap();
                            // }
                            _ => {}
                        },
                        _ => {}
                    },
                    Pane::SearchResults => match *input_mode {
                        InputMode::Normal => match key_event.code {
                            KeyCode::Char('q') => self.quit(),
                            KeyCode::Char('k') => self.select_previous_search_result(),
                            KeyCode::Char('j') => self.select_next_search_result(),
                            KeyCode::Tab => self.switch_pane(Pane::SearchInput),
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn quit(&self) {
        if let Ok(mut should_quit) = self.state.should_quit.lock() {
            *should_quit = true;
        }
    }

    fn switch_input_mode(&self, input_mode: InputMode) {
        if let Ok(mut current_input_mode) = self.state.input_mode.lock() {
            *current_input_mode = input_mode;
        }
    }

    fn switch_pane(&self, pane: Pane) {
        if let Ok(mut current_pane) = self.state.current_pane.lock() {
            *current_pane = pane;
        }
    }

    fn append_search_query(&self, ch: char) {
        if let Ok(mut search) = self.state.search.lock() {
            search.query.push(ch);
            search.selected_result = 0;
        }
    }

    fn pop_search_query(&self) {
        if let Ok(mut search) = self.state.search.lock() {
            search.query.pop();
            search.selected_result = 0;
        }
    }

    fn select_previous_search_result(&self) {
        if let Ok(mut search) = self.state.search.lock() {
            search.selected_result = search.selected_result.saturating_sub(1);
        }
    }

    fn select_next_search_result(&self) {
        if let Ok(mut search) = self.state.search.lock() {
            search.selected_result =
                search.selected_result.saturating_add(1) % search.results.len();
        }
    }
}
