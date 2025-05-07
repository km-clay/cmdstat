use std::fmt::Display;

use crossterm::style::{Color, Stylize};
use unicode_width::UnicodeWidthStr;

use crate::TableColumn;

#[derive(Default,Debug)]
pub struct Table {
	headings: Vec<String>,
	columns: usize,
	rows: Vec<Row>,
	spacer: Option<char>,
	sort_by: Option<usize>,
	reverse: bool,
	no_header: bool
}

impl Table {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn with_n_columns(self, n: usize) -> Self {
		let Self { headings, columns: _, rows, spacer, sort_by, reverse, no_header } = self;
		Self { headings, columns: n, rows, spacer, sort_by, reverse, no_header }
	}
	pub fn with_heading<S: ToString>(self, field_num: usize, heading: S) -> Self {
		assert!(field_num < self.columns);
		let Self { mut headings, columns, rows, spacer, sort_by, reverse, no_header } = self;
		headings.insert(field_num, heading.to_string());
		Self { headings, columns, rows, spacer, sort_by, reverse, no_header }
	}
	pub fn omit_header(&mut self, yn: bool) {
		self.no_header = yn;
	}
	pub fn add_row(&mut self, row: Row) {
		self.rows.push(row)
	}
	pub fn find_col_idx<S: ToString>(&self, column: S) -> Option<usize> {
		let col_name = column.to_string();
		self.headings.iter().position(|h| h == &col_name)
	}
	pub fn reverse(&mut self) {
		self.reverse = true
	}
	pub fn calc_cell_widths(&self) -> Vec<usize> {
		let mut widths = vec![];
		for row in &self.rows {
			let Row { cells } = &row;
			assert!(cells.len() == self.columns);
			for (i, cell) in cells.iter().enumerate() {
				let visible_width: usize = console::strip_ansi_codes(&cell.content).width();
				if let Some(width) = widths.get(i) {
					if visible_width > *width {
						widths[i] = visible_width;
					}
				} else {
					widths.insert(i, visible_width);
				}
			}
		}
		widths
	}
	pub fn set_sort_column(&mut self, col_idx: usize) {
		self.sort_by = Some(col_idx)
	}
	pub fn sort(&mut self) {
		let col_idx = self.sort_by.unwrap_or_default();
		assert!((0..self.columns).contains(&col_idx));

		self.rows.sort_by(|a, b| {
			let cell_a = &a.cells[col_idx];
			let cell_b = &b.cells[col_idx];
			match (cell_a.as_number(), cell_b.as_number()) {
				(Some(an), Some(bn)) => {
					let ord = bn.cmp(&an);
					if self.reverse {
						ord.reverse()
					} else {
						ord
					}
				}
				_ => {
					let ord = if &self.headings[col_idx] == "Usage" { // FIXME: I don't like hard coding this
						cell_a.content.width().cmp(&cell_b.content.width())
					} else {
						cell_b.content.cmp(&cell_a.content)
					};
					if self.reverse {
						ord
					} else {
						ord.reverse()
					}
				}
			}
		});
	}
}

impl Display for Table {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut widths = self.calc_cell_widths();
		for (i,width) in widths.iter_mut().enumerate() {
			if let Some(heading) = &self.headings.get(i) {
				let heading_width = console::strip_ansi_codes(heading).width();
				if heading_width > *width {
					*width = heading_width;
				}
			}
		}

		// headings
		if !self.headings.is_empty() && !self.no_header {
			for (i, heading) in self.headings.iter().enumerate() {
				write!(f, "{:<width$} ", heading, width = widths[i])?;
			}
			writeln!(f)?;
			writeln!(f, "{}", "-".repeat(widths.iter().sum::<usize>() + self.columns))?;
		}

		// rows
		for row in &self.rows {
			for (i, cell) in row.cells.iter().enumerate() {
				let padded = format!("{:<width$} ", cell.content, width = widths[i]);
				if let Some(color) = cell.color {
					write!(f, "{}", padded.with(color))?;
				} else {
					write!(f, "{padded}")?;
				}
			}
			writeln!(f)?;
		}

		Ok(())
	}
}

#[derive(Default,Debug)]
pub struct Row {
	cells: Vec<Cell>
}

impl Row {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn with_cell(self, cell: Cell) -> Self {
		let Self { mut cells } = self;
		cells.push(cell);
		Self { cells }
	}
}

#[derive(Default,Debug)]
pub struct Cell {
	content: String,
	append_spacer: bool,
	truncate_for_space: bool,
	color: Option<Color>
}

impl Cell {
	pub fn new<S: ToString>(content: S) -> Self {
		Self {
			content: content.to_string(),
			append_spacer: true,
			truncate_for_space: false,
			color: None
		}
	}
	pub fn append_spacer(self, yn: bool) -> Self {
		let Self { content, append_spacer: _, truncate_for_space, color } = self;
		Self { content, append_spacer: yn, truncate_for_space, color }
	}
	pub fn truncate_for_space(self, yn: bool) -> Self {
		let Self { content, append_spacer, truncate_for_space: _, color } = self;
		Self { content, append_spacer, truncate_for_space: yn, color }
	}
	pub fn with_color(self, color: Color) -> Self {
		let Self { content, append_spacer, truncate_for_space, color: _ } = self;
		Self { content, append_spacer, truncate_for_space, color: Some(color) }
	}
	pub fn as_number(&self) -> Option<u64> {
		self.content.trim().parse::<u64>().ok()
	}
}
