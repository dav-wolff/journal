use std::{io::{self, Write}, path::{PathBuf, Path}, fs::{self, File}, ffi::{OsString, OsStr}, process::Command, os::unix::prelude::FileExt};
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use entry_list::{EntryList, Entry};
use ratatui::prelude::*;

use crate::{alternate_screen::AlternateScreen, raw_mode::RawMode};

mod alternate_screen;
mod raw_mode;
mod entry_list;

fn main() -> io::Result<()> {
	let Some(directory) = get_directory() else {
		return Ok(());
	};
	
	let Some(editor) = get_editor() else {
		return Ok(());
	};
	
	run_tui(&directory, &editor)
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

fn get_editor() -> Option<OsString> {
	match std::env::var_os("EDITOR") {
		Some(editor) => Some(editor),
		None => {
			eprintln!("No default editor set");
			None
		}
	}
}

fn run_tui(directory: &Path, editor: &OsStr) -> io::Result<()> {
	let _alternate_screen_guard = AlternateScreen::enter();
	let _raw_mode_guard = RawMode::enable();
	
	let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
	terminal.clear()?;
	
	let entries: Vec<Entry> = fs::read_dir(directory)?
		.map(|result| result.map(
			|file| Entry {
				path: file.path(),
				name: file.file_name().to_string_lossy().into_owned(),
			}
		))
		.collect::<io::Result<_>>()?;
	
	let mut entry_list = EntryList::new(entries);
	
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
				KeyCode::Enter => {
					edit_entry(&directory, &editor, entry_list.selected_entry());
				},
				_ => (),
			}
		}
	}
	
	Ok(())
}

fn edit_entry(directory: &Path, editor: &OsStr, entry: &Entry) -> io::Result<()> {
	let text = fs::read_to_string(&entry.path)?;
	
	edit_text(directory, editor, text)
}

fn edit_text(directory: &Path, editor: &OsStr, text: String) -> io::Result<()> {
	let file_path = directory.join("PLAIN_TEXT");
	let mut file = File::create(&file_path)?;
	file.write_all(text.as_bytes())?;
	
	Command::new(editor)
		.arg(&file_path)
		.status()
		.unwrap();
	
	Ok(())
}