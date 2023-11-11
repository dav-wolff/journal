use core::slice;
use std::{io::{self, Write, Read, ErrorKind}, path::{PathBuf, Path}, fs::{self, File}, ffi::OsString, process::Command};
use aes::{Aes256, cipher::{KeyInit, generic_array::GenericArray, BlockDecrypt, BlockEncrypt}};
use alternate_screen::{enter_alternate_screen, leave_alternate_screen};
use argon2::Config;
use crossterm::event::{self, Event, KeyEventKind, KeyCode};
use entry_list::{EntryList, Entry};
use getrandom::getrandom;
use ratatui::prelude::*;
use rpassword::prompt_password;
use zeroize::Zeroizing;

use crate::alternate_screen::AlternateScreen;

mod alternate_screen;
mod entry_list;

const EDITING_FILE_NAME: &'static str = "PLAIN_TEXT";

struct Context {
	directory: PathBuf,
	editing_file_path: PathBuf,
	editor: OsString,
	aes: Aes256,
}

fn main() -> io::Result<()> {
	let Some(directory) = get_directory() else {
		return Ok(());
	};
	
	let Some(editor) = get_editor() else {
		return Ok(());
	};
	
	let salt = get_salt(&directory)?;
	let password = get_password()?;
	let aes = generate_key(password, &salt);
	
	let editing_file_path = directory.join(EDITING_FILE_NAME);
	
	run_tui(Context {
		directory,
		editing_file_path,
		editor,
		aes,
	})
}

fn get_salt(directory: &Path) -> io::Result<[u8; 32]> {
	let file_path = directory.join(".journal");
	
	let mut file = match File::open(&file_path) {
		Ok(file) => file,
		Err(err) if err.kind() == ErrorKind::NotFound => {
			return generate_salt(&file_path);
		},
		Err(err) => return Err(err),
	};
	
	let mut version = 0u8;
	
	match file.read_exact(slice::from_mut(&mut version)) {
		Ok(()) => (),
		Err(err) if err.kind() == ErrorKind::UnexpectedEof => return generate_salt(&file_path),
		Err(err) => return Err(err),
	}
	
	if version != 0u8 {
		eprintln!("Error: unknown version of .journal file, generating new salt");
		return generate_salt(&file_path);
	}
	
	let mut salt = [0u8; 32];
	
	match file.read_exact(&mut salt) {
		Ok(()) => Ok(salt),
		Err(err) if err.kind() == ErrorKind::UnexpectedEof => {
			eprintln!("Error: incorrect format of .journal file, generating new salt");
			generate_salt(&file_path)
		},
		Err(err) => Err(err),
	}
}

fn generate_salt(file_path: &Path) -> io::Result<[u8; 32]> {
	let mut file = File::create(file_path)?;
	
	file.write_all(&[0u8])?; // version
	
	let mut salt = [0u8; 32];
	getrandom(&mut salt).expect("getrandom failed");
	
	file.write_all(&salt)?;
	
	Ok(salt)
}

fn get_password() -> io::Result<Zeroizing<String>> {
	let password = prompt_password("Please enter your password: ")?;
	let password = Zeroizing::new(password);
	
	Ok(password)
}

fn generate_key(password: Zeroizing<String>, salt: &[u8]) -> Aes256 {
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
	
	Aes256::new_from_slice(&key)
		.expect("Key should have the correct size of 32 bytes")
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

fn run_tui(context: Context) -> io::Result<()> {
	let _alternate_screen_guard = AlternateScreen::enter()?;
	
	let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
	terminal.clear()?;
	
	let entries: Vec<Entry> = fs::read_dir(&context.directory)?
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
					edit_entry(&context, &mut terminal, entry_list.selected_entry())?;
				},
				_ => (),
			}
		}
	}
	
	Ok(())
}

fn edit_entry(context: &Context, terminal: &mut Terminal<impl Backend>, entry: &Entry) -> io::Result<()> {
	decrypt_file(context, &entry.path)?;
	edit_text(context, terminal)?;
	encrypt_file(context, &entry.path)?;
	
	Ok(())
}

fn decrypt_file(context: &Context, file_path: &Path) -> io::Result<()> {
	let aes = &context.aes;
	
	let mut encrypted_file = File::open(file_path)?;
	let mut decrypted_file = File::create(&context.editing_file_path)?;
	let mut block = GenericArray::from([0; 16]);
	
	let file_len: usize = encrypted_file.metadata()?
		.len()
		.try_into()
		.expect("File must be smaller than usize::MAX");
	
	for _ in 0..(file_len / block.len()) - 1 {
		encrypted_file.read_exact(&mut block)?;
		aes.decrypt_block(&mut block);
		decrypted_file.write_all(&block)?;
	}
	
	encrypted_file.read_exact(&mut block)?;
	aes.decrypt_block(&mut block);
	
	let trailing_zeroes = block.iter()
		.rev()
		.take_while(|&&byte| byte == 0)
		.count();
	
	decrypted_file.write_all(&block[0..block.len() - trailing_zeroes])?;
	
	Ok(())
}

fn encrypt_file(context: &Context, file_path: &Path) -> io::Result<()> {
	let aes = &context.aes;
	
	let mut decrypted_file = File::open(&context.editing_file_path)?;
	let mut encrypted_file = File::create(file_path)?;
	let mut block = GenericArray::from([0; 16]);
	
	let file_len: usize = decrypted_file.metadata()?
		.len()
		.try_into()
		.expect("File must be smaller than usize::MAX");
	
	for _ in 0..file_len / block.len() {
		decrypted_file.read_exact(&mut block)?;
		aes.encrypt_block(&mut block);
		encrypted_file.write_all(&block)?;
	}
	
	let remaining_file_len = file_len % block.len();
	
	if remaining_file_len > 0 {
		block.fill(0);
		
		decrypted_file.read_exact(&mut block[0..remaining_file_len])?;
		aes.encrypt_block(&mut block);
		encrypted_file.write_all(&block)?;
	}
	
	Ok(())
}

fn edit_text(context: &Context, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
	// causes quick flicker but is necessary to keep main scrollback uncontaminated
	leave_alternate_screen()?;
	
	// TODO handle Err and Ok(status != 0)
	let _ = Command::new(&context.editor)
		.arg(&context.editing_file_path)
		.status();
	
	enter_alternate_screen()?;
	terminal.clear()?;
	
	Ok(())
}
