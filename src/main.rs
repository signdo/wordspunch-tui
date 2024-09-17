use std::{error::Error, io::stderr, path::PathBuf};

use ratatui::{backend::CrosstermBackend, crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}}, Terminal};
use clap::Parser;

mod app;
mod word;
mod data;

/// A software for memorizing words.
#[derive(Parser, Debug)]
#[command(about, version)]
pub struct Args {
    /// FILE content format: 'english:chinese'
    #[arg(value_name = "FILE")]
    src_words_file: Option<PathBuf>,

    /// DATA_FILE that save learning data
    #[arg(short, long, value_name = "DATA_FILE")]
    data_file: Option<PathBuf>,

    /// Count that you want to learn per time
    #[arg(short, long, default_value_t = 20)]
    count: usize,

    /// Proficiency that you want to finishing a word
    #[arg(short, long, default_value_t = 50)]
    max_proficiency: isize,
}

fn main() -> Result<(), Box<dyn Error>> {
    use app::*;

    let args = Args::parse();
    let data_path = args.data_file.unwrap_or(PathBuf::from("./data/learning_data.json"));

    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;

    let mut app = App::new(data_path, args.src_words_file);
    app.run(&mut terminal, args.count, args.max_proficiency)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    Ok(())
}
