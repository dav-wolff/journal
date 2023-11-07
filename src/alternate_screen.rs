use std::io;
use crossterm::{terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}, ExecutableCommand};

pub struct AlternateScreen(());

impl AlternateScreen {
	pub fn enter() -> io::Result<AlternateScreen> {
		enter_alternate_screen()?;
		
		Ok(AlternateScreen(()))
	}
}

impl Drop for AlternateScreen {
	fn drop(&mut self) {
		let result = leave_alternate_screen();
		
		if let Err(err) = result {
			eprintln!("Failed to leave alternate screen: {err}");
		}
	}
}

pub fn enter_alternate_screen() -> io::Result<()> {
	io::stdout().execute(EnterAlternateScreen)?;
	enable_raw_mode()?;
	
	Ok(())
}

pub fn leave_alternate_screen() -> io::Result<()> {
	io::stdout().execute(LeaveAlternateScreen)?;
	disable_raw_mode()?;
	
	Ok(())
}