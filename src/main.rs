use input::InputHandler;
use logging::initialize_logging;
use state::State;
use std::{sync::Arc, thread};
use worker::Worker;

mod app;
mod input;
mod logging;
mod state;
mod tui;
mod worker;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    initialize_logging()?;
    trace_dbg!("initialized logging");
    let state = Arc::new(State::new());

    let mut worker = Worker::new(state.clone());
    let _worker_thread = thread::spawn(move || worker.run());
    trace_dbg!("Worker thread initiated");

    let mut input_handler = InputHandler::new(state.clone());
    let _input_thread = thread::spawn(move || input_handler.run());
    trace_dbg!("Input handler thread initiated");

    let mut terminal = tui::init()?;
    let tui = app::Tui::new(state).run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    tui
}
