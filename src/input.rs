use std::{
    sync::{Arc, mpsc::Sender},
    time::Duration,
};

use color_eyre::eyre::bail;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};

use crate::{
    state::{InputMode, Pane, SearchSource, State},
    worker::WorkerEvent,
    // trace_dbg,
};

pub struct InputHandler {
    tx_worker: Sender<WorkerEvent>,
    state: Arc<State>,
}

impl InputHandler {
    pub fn new(state: Arc<State>, tx_worker: Sender<WorkerEvent>) -> Self {
        Self { tx_worker, state }
    }
    pub fn run(&mut self) {
        loop {
            self.capture_input();
            if let Ok(should_quit) = self.state.should_quit.try_lock() {
                if *should_quit {
                    break;
                }
            }
        }
    }
    fn capture_input(&self) {
        if event::poll(Duration::from_millis(10)).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_press(key_event);
                }
                _ => {}
            };
        } else {
            self.tx_worker.send(WorkerEvent::UpdateSearch).unwrap();
        }
    }

    fn handle_key_press(&self, key_event: event::KeyEvent) {
        let mut current_pane = self.state.current_pane.lock().unwrap();
        let mut input_mode = self.state.input_mode.lock().unwrap();

        match key_event.code {
            KeyCode::Char('1') => *current_pane = Pane::SearchInput,
            KeyCode::Char('2') => *current_pane = Pane::SearchResults,
            _ => {}
        }

        match *current_pane {
            Pane::SearchInput => match *input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('i') => *input_mode = InputMode::Insert,
                    KeyCode::Char('l') => {
                        let mut search_source = self.state.search.source.lock().unwrap();
                        *search_source = SearchSource::Local;
                        self.tx_worker.send(WorkerEvent::UpdateSearch).unwrap();
                    }
                    KeyCode::Char('r') => {
                        let mut search_source = self.state.search.source.lock().unwrap();
                        *search_source = SearchSource::Remote;
                        self.tx_worker.send(WorkerEvent::UpdateSearch).unwrap();
                    }
                    _ => {}
                },
                InputMode::Insert => match key_event.code {
                    KeyCode::Char(ch) => self.append_search_query(ch),
                    KeyCode::Backspace => {
                        self.pop_search_query();
                    }
                    KeyCode::Esc => *input_mode = InputMode::Normal,
                    _ => {}
                },
                _ => {}
            },
            Pane::SearchResults => match *input_mode {
                InputMode::Normal => match key_event.code {
                    KeyCode::Char('q') => self.quit(),
                    KeyCode::Char('k') => self.select_previous_search_result(),
                    KeyCode::Char('j') => self.select_next_search_result(),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn quit(&self) {
        if let Ok(mut should_quit) = self.state.should_quit.lock() {
            *should_quit = true;
        }
    }

    // fn switch_input_mode(&self, input_mode: InputMode) {
    //     if let Ok(mut current_input_mode) = self.state.input_mode.lock() {
    //         *current_input_mode = input_mode;
    //         println!("switched");
    //     }
    // }
    //
    // fn switch_pane(&self, pane: Pane) {
    //     if let Ok(mut current_pane) = self.state.current_pane.lock() {
    //         *current_pane = pane;
    //     }
    // }

    fn append_search_query(&self, ch: char) {
        if let Ok(mut query) = self.state.search.query.lock() {
            query.push(ch);
        }
        self.reset_selected_search_result();
    }

    fn pop_search_query(&self) {
        if let Ok(mut query) = self.state.search.query.lock() {
            query.pop();
        }
        self.reset_selected_search_result();
    }

    fn reset_selected_search_result(&self) {
        if let Ok(mut selected_search_result) = self.state.search.selected_result.lock() {
            *selected_search_result = 0;
        }
    }

    fn select_previous_search_result(&self) {
        if let Ok(mut selected_search_result) = self.state.search.selected_result.lock() {
            *selected_search_result = selected_search_result.saturating_sub(1);
        }
    }

    fn select_next_search_result(&self) {
        if let Ok(mut selected_search_result) = self.state.search.selected_result.lock() {
            if let Ok(results) = self.state.search.results.lock() {
                *selected_search_result = selected_search_result.saturating_add(1) % results.len();
            }
        }
    }
}
