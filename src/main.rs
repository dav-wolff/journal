use std::io;
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use ratatui::{prelude::*, widgets::Paragraph};

use crate::{alternate_screen::AlternateScreen, raw_mode::RawMode};

mod alternate_screen;
mod raw_mode;

fn main() -> io::Result<()> {
	let _alternate_screen_guard = AlternateScreen::enter();
	let _raw_mode_guard = RawMode::enable();
	
	let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
	terminal.clear()?;
	
	loop {
		terminal.draw(|frame| {
			frame.render_widget(
				Paragraph::new("Hello, world! ('q' to quit)")
					.light_blue()
					.on_black(),
				frame.size()
			)
		})?;
		
		if !event::poll(std::time::Duration::from_millis(16))? {
			continue;
		}
		
		if let Event::Key(key) = event::read()? {
			if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
				break;
			}
		}
	}
	
	Ok(())
}