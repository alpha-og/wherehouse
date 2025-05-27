mod app;
mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let app = app::App::default().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!("failed to restore terminal {err}");
    }
    app
}
