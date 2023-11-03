use ratatui::{widgets::{ListItem, Block, List, Borders, ListState}, style::{Style, Color}};

pub struct EntryList {
	list: List<'static>,
	count: usize,
	state: ListState,
}

impl EntryList {
	pub fn new(items: Vec<ListItem<'static>>) -> Self {
		let count = items.len();
		
		let list = List::new(items)
			.block(Block::default().title("Files").borders(Borders::ALL))
			.highlight_style(Style::default().fg(Color::LightBlue));
		
		let state = ListState::default()
			.with_selected(Some(0));
		
		Self {
			list,
			count,
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
			self.count - 1
		} else {
			selected - 1
		};
		
		self.state.select(Some(selected));
	}
	
	pub fn select_next(&mut self) {
		let selected = self.state.selected().unwrap_or(0);
		let selected = (selected + 1) % self.count;
		
		self.state.select(Some(selected));
	}
}