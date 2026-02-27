mod app;
mod cli;
mod clipboard;
mod config;
mod discovery;
mod search;
mod ui;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::App;

fn main() -> Result<()> {
    let args = cli::parse()?;
    let mut app = App::new(&args)?;
    let mut terminal = ratatui::init();

    let result = run(&mut terminal, &mut app);

    ratatui::restore();

    if let Some(cmd) = result? {
        clipboard::copy(&cmd)?;
        println!("Copied: {cmd}");
    }

    Ok(())
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match app.handle_key(key) {
                app::Action::Quit => return Ok(None),
                app::Action::Copy(cmd) => return Ok(Some(cmd)),
                app::Action::Continue => {}
            }
        }
    }
}
