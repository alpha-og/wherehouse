use std::sync::mpsc::Sender;

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};

use crate::state::{
    self, InputMode, Pane, State, StateEvent,
    StateItemType::{self, ShouldQuit},
    StateResponse,
};

pub struct InputHandler {
    tx_state: Sender<state::StateEvent>,
}

impl InputHandler {
    pub fn new(tx_state: Sender<state::StateEvent>) -> Self {
        Self { tx_state }
    }
    pub fn run(&mut self) {
        loop {
            if let Some(StateResponse::ShouldQuit(should_quit)) =
                State::get(&self.tx_state, ShouldQuit)
            {
                if !should_quit {
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
        if let Some(StateResponse::CurrentPane(current_pane)) =
            State::get(&self.tx_state, StateItemType::CurrentPane)
        {
            if let Some(StateResponse::InputMode(input_mode)) =
                State::get(&self.tx_state, StateItemType::InputMode)
            {
                match current_pane {
                    Pane::SearchInput => match input_mode {
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
                    Pane::SearchResults => match input_mode {
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
        State::set(&self.tx_state, StateEvent::SetShouldQuit(true)).unwrap();
    }

    fn switch_input_mode(&self, input_mode: InputMode) {
        State::set(&self.tx_state, StateEvent::SetInputMode(input_mode)).unwrap();
    }

    fn switch_pane(&self, pane: Pane) {
        State::set(&self.tx_state, StateEvent::SetCurrentPane(pane)).unwrap();
    }

    fn append_search_query(&self, ch: char) {
        if let Some(StateResponse::SearchQuery(mut search_query)) =
            State::get(&self.tx_state, StateItemType::SearchQuery)
        {
            search_query.push(ch);

            self.tx_state
                .send(StateEvent::SetSearchQuery(search_query))
                .unwrap();
        }
    }

    fn pop_search_query(&self) {
        if let Some(StateResponse::SearchQuery(mut search_query)) =
            State::get(&self.tx_state, StateItemType::SearchQuery)
        {
            search_query.pop();

            State::set(&self.tx_state, StateEvent::SetSearchQuery(search_query)).unwrap();
        }
    }

    fn select_previous_search_result(&self) {
        if let Some(StateResponse::SelectedSearchResult(mut selected_search_result)) =
            State::get(&self.tx_state, StateItemType::SearchSelectedResult)
        {
            selected_search_result = selected_search_result.saturating_sub(1);
            State::set(
                &self.tx_state,
                StateEvent::SetSearchSelectedResult(selected_search_result),
            )
            .unwrap();
        }
    }

    fn select_next_search_result(&self) {
        if let Some(StateResponse::SelectedSearchResult(mut selected_search_result)) =
            State::get(&self.tx_state, StateItemType::SearchSelectedResult)
        {
            selected_search_result = selected_search_result.saturating_add(1);
            State::set(
                &self.tx_state,
                StateEvent::SetSearchSelectedResult(selected_search_result),
            )
            .unwrap();
        }
    }
}
