use commands::CommandType;
use input::InputHandler;
use logging::initialize_logging;
use state::State;
use std::{sync::Arc, thread};
use task_manager::TaskManager;
use tracing::info;
use wherehouse::package_manager::{Command, homebrew::Homebrew};

mod commands;
mod input;
mod logging;
mod state;
mod task_manager;
mod tui;
mod widget;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    initialize_logging()?;
    info!("initialized logging");
    let state = Arc::new(State::new());
    let package_manager = Arc::new(Homebrew);
    let mut task_manager = TaskManager::new(state.clone(), package_manager);
    task_manager.execute(Command::Config, false)?;

    let mut input_handler = InputHandler::new(state.clone(), task_manager);
    let _input_thread = thread::spawn(move || input_handler.run());
    info!("Input handler thread initiated");

    let mut terminal = tui::init()?;
    let tui = tui::Tui::new(state).run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    tui
}
