use std::io;

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

pub struct RawMode(());

impl RawMode {
	pub fn enable() -> io::Result<RawMode> {
		enable_raw_mode()?;
		
		Ok(RawMode(()))
	}
}

impl Drop for RawMode {
	fn drop(&mut self) {
		let result = disable_raw_mode();
		
		if let Err(err) = result {
			eprintln!("Failed to disable raw mode: {err}");
		}
	}
}