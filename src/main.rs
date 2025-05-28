use ratatui::crossterm::event::{self, KeyEventKind};
use std::{sync::mpsc, thread};

mod app;
mod input;
mod tui;
mod worker;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let (tx_worker, rx_worker) = mpsc::channel::<worker::WorkerEvent>();
    let (tx_input, rx_input) = mpsc::channel::<input::InputEvent>();
    let (tx_tui, rx_tui) = mpsc::channel::<app::TuiEvent>();

    let tx_tui_from_worker = tx_tui.clone();
    // let tx_input_capture_from_worker = tx_input_capture.clone();
    let _worker_thread =
        thread::spawn(move || worker::worker_thread(rx_worker, tx_tui_from_worker));

    let tx_tui_from_input = tx_tui.clone();
    // let tx_worker_from_input_capture = tx_tui.clone();
    let _input_thread = thread::spawn(move || input_thread(rx_input, tx_tui_from_input));

    let tx_worker_from_tui = tx_worker.clone();
    let tx_input_capture_from_tui = tx_input.clone();
    let mut terminal = tui::init()?;
    let tui = app::App::default().run(
        &mut terminal,
        rx_tui,
        tx_worker_from_tui,
        tx_input_capture_from_tui,
    );
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    tui
}

fn input_thread(_rx: mpsc::Receiver<input::InputEvent>, tx_tui: mpsc::Sender<app::TuiEvent>) {
    loop {
        match event::read().unwrap() {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                tx_tui.send(app::TuiEvent::KeyInput(key_event)).unwrap();
            }
            _ => {}
        };
    }
}
