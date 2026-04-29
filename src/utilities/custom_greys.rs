use ratatui::style::Color;

pub trait CustomGreys {
	fn selection_tail_grey() -> Self;
	fn secondary_selection_head_grey() -> Self;
	fn ui_grey() -> Self;
}

impl CustomGreys for Color {
	fn selection_tail_grey() -> Self {
		Self::Indexed(242)
	}
	
	fn secondary_selection_head_grey() -> Self {
		Self::Indexed(246)
	}
	
	fn ui_grey() -> Self {
		Self::Indexed(238)
	}
}
