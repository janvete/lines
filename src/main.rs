mod app;
mod commands;
mod config;
mod events;
mod parser;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::App;
use crate::config::Config;
use crate::events::{handle_event, poll_event};

#[derive(Parser, Debug)]
#[command(name = "lines")]
#[command(about = "TUI pro rychlé spouštění často používaných příkazů")]
#[command(version)]
struct Args {
    /// Cesta k datové složce (výchozí: ~/.lines)
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let config = Config::new(args.dir);

    // Ensure data directory exists
    if let Err(e) = std::fs::create_dir_all(&config.data_dir) {
        eprintln!("Nepodařilo se vytvořit datovou složku {}: {}", config.data_dir.display(), e);
    }

    let app = App::new(config.data_dir, config.terminal, config.shell);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()>
where
    io::Error: From<<B as Backend>::Error>,
{
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(event) = poll_event(tick_rate)?
            && !handle_event(&mut app, event)
        {
            return Ok(());
        }
    }
}
