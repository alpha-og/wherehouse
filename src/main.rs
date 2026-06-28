use std::sync::Arc;
use std::thread;

use logging::initialize_logging;
use state::Event;
use task_manager::TaskManager;
use tracing::info;
use wherehouse::package_manager::{self, Command};

mod input;
mod logging;
mod state;
mod task_manager;
mod tui;
mod widget;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    initialize_logging()?;
    info!("Initialized logging");

    let state = Arc::new(state::State::new());
    let (event_tx, event_rx) = std::sync::mpsc::channel::<Event>();

    let package_manager = Arc::new(package_manager::detect_package_manager());
    let mut task_manager = TaskManager::new(state.clone(), package_manager, event_tx.clone());

    let mut input_handler = input::InputHandler::new(state.clone(), event_tx.clone());
    let _input_thread = thread::spawn(move || input_handler.run());
    info!("Input handler thread initiated");

    task_manager.execute(Command::Config)?;
    task_manager.execute(Command::FilterPackages)?;

    let mut terminal = tui::init()?;

    loop {
        while let Ok(event) = event_rx.try_recv() {
            if let Some(cmd) = state::update::update(&state, &event) {
                task_manager.execute(cmd)?;
            }
        }

        if state.debounce_search() {
            let e = Event::CommandIssued(Command::FilterPackages);
            if let Some(cmd) = state::update::update(&state, &e) {
                task_manager.execute(cmd)?;
            }
        }

        terminal.draw(|frame| tui::draw(&state, frame))?;

        if state.exit() {
            break;
        }
    }

    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    Ok(())
}
