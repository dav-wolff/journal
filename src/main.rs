use std::{io, path::{PathBuf, Path}, fs, collections::btree_map::Entry};
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use entry_list::EntryList;
use ratatui::{prelude::*, widgets::{List, ListItem, Block, Borders, ListState}};

use crate::{alternate_screen::AlternateScreen, raw_mode::RawMode};

mod alternate_screen;
mod raw_mode;
mod entry_list;

fn main() -> io::Result<()> {
	let Some(directory) = get_directory() else {
		return Ok(());
	};
	
	run_tui(&directory)
}

fn get_directory() -> Option<PathBuf> {
	let mut args = std::env::args();
	
	args.next(); // journal
	
	if let Some(path_arg) = args.next() {
		let path: PathBuf = path_arg.into();
		
		if !path.exists() {
			eprintln!("`{}` does not exist", path.display());
			return None;
		}
		
		if !path.is_dir() {
			eprintln!("`{}` is not a directory", path.display());
			return None;
		}
		
		return Some(path);
	} else {
		eprintln!("No directory specified");
		return None;
	}
}

fn run_tui(directory: &Path) -> io::Result<()> {
	let _alternate_screen_guard = AlternateScreen::enter();
	let _raw_mode_guard = RawMode::enable();
	
	let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
	terminal.clear()?;
	
	let list_items: Vec<ListItem<'static>> = fs::read_dir(directory)?
		.map(|result| result.map(
			|file|
				ListItem::new(file.file_name().to_string_lossy().into_owned())
			)
		)
		.collect::<io::Result<_>>()?;
	
	let mut entry_list = EntryList::new(list_items);
	
	loop {
		terminal.draw(|frame| {
			frame.render_stateful_widget(
				entry_list.list(),
				frame.size(),
				entry_list.state()
			)
		})?;
		
		if !event::poll(std::time::Duration::from_millis(16))? {
			continue;
		}
		
		if let Event::Key(key) = event::read()? {
			if key.kind != KeyEventKind::Press {
				continue;
			}
			
			match key.code {
				KeyCode::Char('q') => {
					break;
				},
				KeyCode::Up => {
					entry_list.select_prev();
				},
				KeyCode::Down => {
					entry_list.select_next();
				},
				_ => (),
			}
		}
	}
	
	Ok(())
}