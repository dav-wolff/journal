use std::path::PathBuf;

use ratatui::{widgets::{ListItem, Block, List, Borders, ListState}, style::{Style, Color}};

pub struct Entry {
	pub path: PathBuf,
	pub name: String,
}

pub struct EntryList {
	entries: Vec<Entry>,
	list: List<'static>,
	state: ListState,
}

impl EntryList {
	pub fn new(entries: Vec<Entry>) -> Self {
		let list_items: Vec<_> = entries.iter()
			.map(|entry| ListItem::new(entry.name.clone()))
			.collect();
		
		let list = List::new(list_items)
			.block(Block::default().title("Files").borders(Borders::ALL))
			.highlight_style(Style::default().fg(Color::LightBlue));
		
		let state = ListState::default()
			.with_selected(Some(0));
		
		Self {
			entries,
			list,
			state,
		}
	}
	
	pub fn list(&self) -> List<'static> {
		self.list.clone()
	}
	
	pub fn state(&mut self) -> &mut ListState {
		&mut self.state
	}
	
	pub fn select_prev(&mut self) {
		let selected = self.state.selected().unwrap_or(0);
		
		let selected = if selected == 0 {
			self.entries.len() - 1
		} else {
			selected - 1
		};
		
		self.state.select(Some(selected));
	}
	
	pub fn select_next(&mut self) {
		let selected = self.state.selected().unwrap_or(0);
		let selected = (selected + 1) % self.entries.len();
		
		self.state.select(Some(selected));
	}
	
	pub fn selected_entry(&self) -> &Entry {
		&self.entries[self.state.selected().unwrap_or(0)]
	}
}