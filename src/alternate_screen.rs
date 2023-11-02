use std::io;
use crossterm::{terminal::{EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};

pub struct AlternateScreen(());

impl AlternateScreen {
	pub fn enter() -> io::Result<AlternateScreen> {
		io::stdout().execute(EnterAlternateScreen)?;
		
		Ok(AlternateScreen(()))
	}
}

impl Drop for AlternateScreen {
	fn drop(&mut self) {
		let result = io::stdout().execute(LeaveAlternateScreen).map(|_| ());
		
		if let Err(err) = result {
			eprintln!("Failed to leave alternate screen: {err}");
		}
	}
}