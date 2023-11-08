use std::{io::{self, Write}, path::{PathBuf, Path}, fs::{self, File}, ffi::{OsString, OsStr}, process::Command};
use alternate_screen::{enter_alternate_screen, leave_alternate_screen};
use argon2::Config;
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use entry_list::{EntryList, Entry};
use ratatui::prelude::*;
use rpassword::prompt_password;
use zeroize::Zeroizing;

use crate::alternate_screen::AlternateScreen;

mod alternate_screen;
mod entry_list;

fn main() -> io::Result<()> {
	let Some(directory) = get_directory() else {
		return Ok(());
	};
	
	let Some(editor) = get_editor() else {
		return Ok(());
	};
	
	let password = get_password()?;
	let key = generate_key(password, b"salty_salt"); // TODO use proper salt
	
	println!("Your key is: {:x?}", *key);
	
	run_tui(&directory, &editor)
}

fn get_password() -> io::Result<Zeroizing<String>> {
	let password = prompt_password("Please enter your password: ")?;
	let password = Zeroizing::new(password);
	
	Ok(password)
}

fn generate_key(password: Zeroizing<String>, salt: &[u8]) -> Zeroizing<Vec<u8>> {
	let config = Config {
		ad: b"journal_key",
		hash_length: 32,
		lanes: 4,
		mem_cost: 256 * 1024,
		secret: &[],
		time_cost: 1,
		variant: argon2::Variant::Argon2id,
		version: argon2::Version::Version13,
	};
	
	let key = argon2::hash_raw(password.as_bytes(), salt, &config).unwrap();
	let key = Zeroizing::new(key);
	
	key
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
	let _alternate_screen_guard = AlternateScreen::enter()?;
	
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
					edit_entry(&directory, &editor, &mut terminal, entry_list.selected_entry())?;
				},
				_ => (),
			}
		}
	}
	
	Ok(())
}

fn edit_entry(directory: &Path, editor: &OsStr, terminal: &mut Terminal<impl Backend>, entry: &Entry) -> io::Result<()> {
	let text = fs::read_to_string(&entry.path)?;
	
	edit_text(directory, editor, terminal, text)
}

fn edit_text(directory: &Path, editor: &OsStr, terminal: &mut Terminal<impl Backend>, text: String) -> io::Result<()> {
	let file_path = directory.join("PLAIN_TEXT");
	let mut file = File::create(&file_path)?;
	file.write_all(text.as_bytes())?;
	
	// causes quick flicker but is necessary to keep main scrollback uncontaminated
	leave_alternate_screen()?;
	
	// TODO handle Err and Ok(status != 0)
	let _ = Command::new(editor)
		.arg(&file_path)
		.status();
	
	enter_alternate_screen()?;
	terminal.clear()?;
	
	Ok(())
}