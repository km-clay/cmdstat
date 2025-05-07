use std::{collections::HashMap, fmt::Display, fs, path::PathBuf, str::FromStr};

use clap::{arg, command, Parser};
use crossterm::{style::{Color, Stylize}, terminal};
use dirs::data_local_dir;
use serde::Deserialize;
use table::{Cell, Row, Table};

pub mod table;

const BAR_CHARS: [&str;8] = [
	"▏",
	"▎",
	"▍",
	"▌",
	"▋",
	"▊",
	"▉",
	"█",
];

#[derive(Parser,Debug)]
#[command(author, version, about)]
struct Cli {
	/// Display all commands
	#[arg(short, long, help = "Display all commands from the stats file. Ignores --num.")]
	all: bool,

	/// Number of entries to display
	#[arg(short, long, default_value = "10", help = "Choose a specific number of commands to show.")]
	num: usize,

	/// Display extra info about each command
	#[arg(short)]
	long: bool,

	/// Display info on a single command
	#[arg(long, help = "Displays detailed information for a specific command.")]
	command: Option<String>,

	/// Specify which columns to display
	#[arg(long, value_delimiter = ',', long_help = "Choose specific columns to display. Possible options are:
		'command/cmd',
		'count/calls',
		'usage/bar',
		'percent/pct/%',
		'type'.")]
	columns: Vec<TableColumn>,

	/// Specify which column to sort by
	#[arg(long)]
	sort: Option<TableColumn>,

	/// Reverse the sort
	#[arg(long)]
	reverse: bool,

	/// Dump raw json
	#[arg(long)]
	json: bool,

	/// Omit the table headers
	#[arg(long)]
	no_header: bool
}

#[derive(Clone,Copy,Debug)]
pub enum TableColumn {
	Command,
	Count,
	Usage,
	Percent,
	Dirs,
	Type
}

impl FromStr for TableColumn {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
	  match s.to_lowercase().as_str() {
			"command" | "cmd" => Ok(TableColumn::Command),
			"count" | "calls" => Ok(TableColumn::Count),
			"usage" | "bar" => Ok(TableColumn::Usage),
			"percent" | "pct" | "%" => Ok(TableColumn::Percent),
			"type" => Ok(TableColumn::Type),
			_ => Err(format!("cmdstat: invalid column name `{}'", s))
		}
	}
}

impl Display for TableColumn {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TableColumn::Command => write!(f,"Command"),
			TableColumn::Count => write!(f,"Count"),
			TableColumn::Usage => write!(f,"Usage"),
			TableColumn::Percent => write!(f,"Percent"),
			TableColumn::Dirs => write!(f,"Dirs"),
			TableColumn::Type => write!(f,"Type"),
		}
	}
}

#[derive(Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
pub enum CmdKind {
	Alias,
	Function,
	Builtin,
	Command,
	Reserved,
	#[serde(other)]
	Unknown
}

impl Display for CmdKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CmdKind::Alias => write!(f,"alias"),
			CmdKind::Function => write!(f,"function"),
			CmdKind::Builtin => write!(f,"builtin"),
			CmdKind::Command => write!(f,"command"),
			CmdKind::Reserved => write!(f,"reserved"),
			CmdKind::Unknown => write!(f,"unknown"),
		}
	}
}

#[derive(Deserialize,Debug)]
pub struct Entry {
	command: String,
	count: u32,
	kind: CmdKind,
	dirs: HashMap<PathBuf,u32>,
}

#[derive(Deserialize,Debug)]
pub struct Entries(Vec<Entry>);

impl Entries {
	pub fn sort_entries(&mut self) {
		self.0.sort_by(|ent_a, ent_b| ent_b.count.cmp(&ent_a.count));
	}
	pub fn prune_entries(&mut self, num: usize) {
		self.0.truncate(num);
	}
}

#[derive(Debug)]
pub struct CmdStats {
	entries: Entries,
	cli: Cli
}

impl CmdStats {
	pub fn prepare_entries(&mut self) {
		self.entries.sort_entries();
		if !self.cli.all {
			self.entries.prune_entries(self.cli.num);
		}
	}
	pub fn get_entry_table(&self) -> Table {
		let mut table = if !self.cli.columns.is_empty() {
			self.get_specified_table()
		} else {
			self.get_default_table()
		};
		if let Some(col) = &self.cli.sort {
			let col_idx = table.find_col_idx(col).unwrap(); // TODO: handle this unwrap
			table.set_sort_column(col_idx);
		} else {
			let col_idx = table.find_col_idx(TableColumn::Count).unwrap_or(0);
			table.set_sort_column(col_idx);
		}
		if self.cli.reverse {
			table.reverse();
		}
		table.sort();
		table.omit_header(self.cli.no_header);
		table
	}
	pub fn get_specified_table(&self) -> Table {
		let total: usize = self.entries.0.iter().map(|ent| ent.count as usize).sum();
		let columns = &self.cli.columns;
		let mut table = Table::new()
			.with_n_columns(columns.len());


		for (i,column) in columns.iter().enumerate() {
			table = table.with_heading(i, column);
		}

		for entry in &self.entries.0 {
			let Entry { command, count, kind, dirs } = entry;
			let percentage = ((*count as f64 / total as f64) * 100.0) as usize;
			let mut row = Row::new();
			for column in columns {
				match column {
					TableColumn::Command => {
						row = row.with_cell(Cell::new(command))
					}
					TableColumn::Count => {
						row = row.with_cell(Cell::new(count))
					}
					TableColumn::Usage => {
						row = row.with_cell(Cell::new(get_bar(percentage, bar_width())).with_color(Color::Green));
					}
					TableColumn::Percent => {
						row = row.with_cell(Cell::new(format!("{percentage}%")));
					}
					TableColumn::Dirs => todo!(),
					TableColumn::Type => {
						row = row.with_cell(Cell::new(kind))
					}
				}
			}
			table.add_row(row);
		}

		table
	}
	pub fn get_default_table(&self) -> Table {
		let total: usize = self.entries.0.iter().map(|ent| ent.count as usize).sum();
		let mut table = Table::new()
			.with_n_columns(4)
			.with_heading(0, "Command")
			.with_heading(1, "Count")
			.with_heading(2, "Percent")
			.with_heading(3, "Usage");

		for entry in &self.entries.0 {
			let Entry { command, count, kind: _, dirs: _ } = entry;
			let percentage = ((*count as f64 / total as f64) * 100.0) as usize;
			let cmd_cell = Cell::new(command);
			let count_cell = Cell::new(count);
			let bar_cell = Cell::new(get_bar(percentage, bar_width())).with_color(Color::Green);
			let perc_cell = Cell::new(format!("{percentage}%"));

			let row = Row::new()
				.with_cell(cmd_cell)
				.with_cell(count_cell)
				.with_cell(perc_cell)
				.with_cell(bar_cell);
			table.add_row(row);
		}

		table
	}
	pub fn print_entries(&mut self) {
		let table = self.get_entry_table()
			.with_title("Command Statistics".with(Color::Cyan).bold());
		if !self.cli.no_header {
			println!();
		}
		print!("{table}");
	}
}

fn stats_file() -> PathBuf {
	data_local_dir()
		.unwrap()
		.join("cmdstat")
		.join("stats.json")
}

fn term_width() -> usize {
	terminal::size().map(|(width, _)| width as usize).unwrap_or(80)
}

fn bar_width() -> usize {
	(term_width() as f64 * 0.70) as usize
}

fn read_stats() -> String {
	fs::read_to_string(stats_file())
		.unwrap_or_default()
}

fn get_bar(percentage: usize, term_width: usize) -> String {
	let scaled_percentage = (percentage as f64 / 100.0) * term_width as f64;
	let full_bars = scaled_percentage.floor() as usize;
	let remainder = ((scaled_percentage - full_bars as f64) * 10.0).round() as usize;
	let mut bar = BAR_CHARS[7].repeat(full_bars);
	if remainder > 0 {
		let bar_index = match remainder {
			1 => 0,
			2 => 1,
			3 |
			4 => 2,
			5 |
			6 => 3,
			7 => 4,
			8 => 5,
			9 => 6,
			10 => 7,
			_ => unreachable!(),
		};
		bar.push_str(BAR_CHARS[bar_index]);
	}
	bar
}

fn main() {
	let cli = Cli::parse();
	let raw = read_stats();
	if cli.json {
		println!("{raw}");
		return
	}
	let entries: Entries = serde_json::from_str(&raw).unwrap();
	let mut cmd_stats = CmdStats { entries, cli };
	cmd_stats.print_entries();
}
