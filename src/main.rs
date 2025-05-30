use input::InputHandler;
use ratatui::crossterm::event::{self, KeyEventKind};
use state::State;
use std::{sync::mpsc, thread};
use worker::Worker;

mod app;
mod input;
mod state;
mod tui;
mod worker;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let (tx_state, rx_state) = mpsc::channel::<state::StateEvent>();
    let mut state = State::default();
    let _state_thread = thread::spawn(move || state.manage(rx_state));

    let worker = Worker::new(tx_state.clone());
    let _worker_thread = thread::spawn(move || worker.run());

    let mut input_handler = InputHandler::new(tx_state.clone());
    let _input_thread = thread::spawn(move || input_handler.run());

    let mut terminal = tui::init()?;
    let tui = app::Tui::new(tx_state.clone()).run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    tui
}
