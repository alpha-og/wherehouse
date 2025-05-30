use input::InputHandler;
use state::State;
use std::{sync::Arc, thread};
use worker::Worker;

mod app;
mod input;
mod state;
mod tui;
mod worker;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut state = Arc::new(State::new());

    let worker = Worker::new(state.clone());
    let _worker_thread = thread::spawn(move || worker.run());

    let mut input_handler = InputHandler::new(state.clone());
    let _input_thread = thread::spawn(move || input_handler.run());

    let mut terminal = tui::init()?;
    let tui = app::Tui::new(state).run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    tui
}
