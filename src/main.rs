use std::{cmp::Reverse, collections::HashMap, env, fmt::{Display, Write}, fs, io::Write as IoWrite, path::{Path, PathBuf}, process::Stdio, str::FromStr};
use regex::Regex;

use clap::{arg, command, Parser};
use crossterm::{style::{Color, Stylize}, terminal};
use dirs::data_local_dir;
use serde::Deserialize;
use serde_json::Value;
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
	/// Display statistics for specific commands. 
	commands: Vec<String>,

	/// Display all commands
	#[arg(short, long, help = "Display all commands from the stats file. Ignores --num.")]
	all: bool,

	/// Number of entries to display
	#[arg(short, long, default_value = "20", help = "Choose a specific number of commands to show.")]
	num: usize,

	/// Display extra info about each command
	#[arg(short)]
	long: bool,

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
	no_header: bool,

	/// Choose a custom bar color
	#[arg(long, long_help = "Choose a custom bar color. Can take the name of any valid ansi color, as well as rgb or raw ansi codes
		Examples:
		'green'
		'darkred'
		'dark_magenta'
		'132,50,1'
		'31'")]
	bar_color: Option<String>,

	#[arg(long)]
	no_pager: bool,

	#[arg(long)]
	clear_stats: bool
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

impl Entry {
	fn detail_display(&self) -> String {
		let mut display = String::new();
		let Entry { command, count, kind, dirs } = self;
		let mut dirs: Vec<(PathBuf, u32)> = dirs.iter()
			.map(|(p,n)| (p.clone(),*n))
			.collect();
		dirs.sort_by_key(|&(_,n)| Reverse(n));
		dirs.truncate(10);

		let calls = "calls".with(Color::Cyan).bold();
		let class = "class".with(Color::Cyan).bold();
		let top_dirs = "top directories".with(Color::Cyan).bold();

		let term_width  = term_dimensions().0;
		let bar = "-".repeat((term_width as f64 * 0.5) as usize);
		writeln!(display, "{bar}").unwrap();
		writeln!(display, "\t{}",command.clone().with(Color::Cyan).bold()).unwrap();
		writeln!(display, "{bar}").unwrap();
		writeln!(display).unwrap();
		writeln!(display, "{calls}: {count}").unwrap();
		writeln!(display, "{class}: {kind}").unwrap();
		writeln!(display, "{top_dirs}: ").unwrap();
		for (dir,count) in dirs {
			let fmt_dir = prettify_dir(dir);
			writeln!(display, "\t{fmt_dir}: {count}").unwrap()
		}

		display
	}
}

fn prettify_dir<P: AsRef<Path>>(dir: P) -> String {
	let path = dir.as_ref();
	let raw = path.display().to_string();
	let is_home = if let Ok(home) = env::var("HOME") { raw.starts_with(&home) } else { false };
	if is_home {
		let home_dir_segs = PathBuf::from(env::var("HOME").unwrap()).components().count();
		let path_segs = path.components().skip(home_dir_segs);
		let mut pretty = "~".with(Color::Blue).to_string();
		for seg in path_segs {
			let slash = "/".with(Color::DarkCyan);
			let seg = seg.as_os_str().to_string_lossy();
			let seg_pretty = seg.with(Color::Blue);
			pretty.push_str(&format!("{slash}{seg_pretty}"));
		}
		pretty
	} else {
		let path_segs = path.components().skip(1);
		let mut pretty = "/".with(Color::DarkCyan).to_string();
		for seg in path_segs {
			let slash = "/".with(Color::DarkCyan);
			let seg = seg.as_os_str().to_string_lossy();
			let seg_pretty = seg.with(Color::Blue);
			pretty.push_str(&format!("{slash}{seg_pretty}"));
		}
		pretty
	}
}

#[derive(Deserialize,Debug,Default)]
pub struct Entries(Vec<Entry>);

impl Entries {
	pub fn sort_entries(&mut self) {
		self.0.sort_by(|ent_a, ent_b| ent_b.count.cmp(&ent_a.count));
	}
	pub fn prune_entries(&mut self, num: usize) {
		self.0.truncate(num);
	}
	pub fn retain_entries<F: FnMut(&Entry) -> bool>(&mut self, predicate: F) {
		self.0.retain(predicate)
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
	pub fn get_entry_table(&self, bar_color: Option<Color>) -> Table {
		let mut table = if !self.cli.columns.is_empty() {
			self.get_specified_table(bar_color)
		} else {
			self.get_default_table(bar_color)
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
	pub fn get_specified_table(&self, bar_color: Option<Color>) -> Table {
		let bar_color = bar_color.unwrap_or(Color::Green);
		let total: usize = self.entries.0.iter().map(|ent| ent.count as usize).sum();
		let columns = &self.cli.columns;
		let mut table = Table::new()
			.with_n_columns(columns.len());


		for (i,column) in columns.iter().enumerate() {
			table = table.with_heading(i, column);
		}

		for entry in &self.entries.0 {
			let Entry { command, count, kind, dirs: _ } = entry;
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
						row = row.with_cell(Cell::new(get_bar(percentage, bar_width())).with_color(bar_color));
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
	pub fn get_default_table(&self, bar_color: Option<Color>) -> Table {
		let bar_color = bar_color.unwrap_or(Color::Green);
		let total: usize = self.entries.0.iter().map(|ent| ent.count as usize).sum();
		let mut table = Table::new()
			.with_n_columns(4)
			.with_heading(0, "Command")
			.with_heading(1, "Count")
			.with_heading(2, "Percent")
			.with_heading(3, "Usage");

		for entry in &self.entries.0 {
			let Entry { command, count, kind: _, dirs: _ } = entry;
			let percentage = (*count as f64 / total as f64) * 100.0;
			let cmd_cell = Cell::new(command);
			let count_cell = Cell::new(count);
			let bar_cell = Cell::new(get_bar(percentage as usize, bar_width())).with_color(bar_color);
			let perc_cell = Cell::new(format!("{percentage:.01}%"));

			let row = Row::new()
				.with_cell(cmd_cell)
				.with_cell(count_cell)
				.with_cell(perc_cell)
				.with_cell(bar_cell);
			table.add_row(row);
		}

		table
	}
	pub fn format_entries(&mut self, bar_color: Option<Color>) -> String {
		self.prepare_entries();
		let table = self.get_entry_table(bar_color)
			.with_title("Command Statistics".with(Color::Cyan).bold());
		if !self.cli.no_header {
			println!();
		}
		format!("{table}")
	}
}

fn stats_file() -> PathBuf {
	if let Ok(var) = env::var("CMDSTAT_FILE") {
		var.into()
	} else {
		data_local_dir()
			.unwrap()
			.join("cmdstat")
			.join("stats.json")
	}
}

fn term_dimensions() -> (usize,usize) {
	terminal::size().map(|(w, h)| (w as usize, h as usize)).unwrap_or((80,24))
}

fn bar_width() -> usize {
	(term_dimensions().0 as f64 * 0.70) as usize
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
	let bar_index = match remainder {
		0 | // Zero percent still gets a bar
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
	bar
}

fn clear_stats() {
	use std::io::{self, Write};
	use std::fs::{self, OpenOptions};

	println!("This will irreversibly clear the stats file.");
	let mut answer = String::new();

	while answer.trim() != "y" && answer.trim() != "n" {
		print!("Are you sure? y/n ");
		io::stdout().flush().unwrap(); 
		if io::stdin().read_line(&mut answer).is_err() {
			eprintln!("Failed to read input, exiting.");
			return;
		}
		match answer.trim() {
			"n" => {
				println!("Exiting.");
				return;
			}
			"y" => break,
			_ => continue,
		}
	}

	let stats_path = stats_file();

	if let Some(parent) = stats_path.parent() {
		if let Err(e) = fs::create_dir_all(parent) {
			eprintln!("Failed to create directory {}: {}", parent.display(), e);
			return;
		}
	}

	match OpenOptions::new().write(true).truncate(true).create(true).open(&stats_path) {
		Ok(mut file) => {
			if let Err(e) = file.write_all(b"[]") {
				eprintln!("Failed to write to stats file: {}", e);
			} else {
				println!("Stats file cleared.");
			}
		}
		Err(e) => {
			eprintln!("Failed to open stats file: {}", e);
		}
	}
}

fn get_color(color: &str) -> Result<Color,String> {
	let color = color.to_ascii_lowercase();
	let rgb_regex = Regex::new(r"^(?P<r>\d{1,3}),(?P<g>\d{1,3}),(?P<b>\d{1,3})$").unwrap();
	let ansi_regex = Regex::new(r"^(?P<code>\d{1,3})$").unwrap();

	if rgb_regex.is_match(&color) {
		let caps = rgb_regex.captures(&color).unwrap();
		let r = caps["r"].parse::<u8>().map_err(|_| "Invalid number for 'r' value in rgb pattern. Valid numbers are 0-255")?;
		let g = caps["g"].parse::<u8>().map_err(|_| "Invalid number for 'g' value in rgb pattern. Valid numbers are 0-255")?;
		let b = caps["b"].parse::<u8>().map_err(|_| "Invalid number for 'b' value in rgb pattern. Valid numbers are 0-255")?;
		Ok(Color::Rgb { r, g, b })
	} else if ansi_regex.is_match(&color) {
		let caps = ansi_regex.captures(&color).unwrap();
		let code = caps["code"].parse::<u8>().map_err(|_| "Invalid number for 'code' value in ansi pattern. Valid numbers are 0-255")?;
		Ok(Color::AnsiValue(code))
	} else {
		match color.as_str() {
			"black"        => Ok(Color::Black),
			"darkgrey"     |
			"dark_grey"    => Ok(Color::DarkGrey),
			"red"          => Ok(Color::Red),
			"darkred"      |
			"dark_red"     => Ok(Color::DarkRed),
			"green"        => Ok(Color::Green),
			"darkgreen"    |
			"dark_green"   => Ok(Color::DarkGreen),
			"yellow"       => Ok(Color::Yellow),
			"darkyellow"   |
			"dark_yellow"  => Ok(Color::DarkYellow),
			"blue"         => Ok(Color::Blue),
			"darkblue"     |
			"dark_blue"    => Ok(Color::DarkBlue),
			"magenta"      => Ok(Color::Magenta),
			"darkmagenta"  |
			"dark_magenta" => Ok(Color::DarkMagenta),
			"cyan"         => Ok(Color::Cyan),
			"darkcyan"     |
			"dark_cyan"    => Ok(Color::DarkCyan),
			"white"        => Ok(Color::White),
			"grey"         => Ok(Color::Grey),
			_ => Err(format!("Invalid color name: '{color}'"))
		}
	}
}

fn page_output(output: &str) -> Result<(), ()> {
	let pager = std::env::var("PAGER").unwrap_or("less".into());
	let mut child = std::process::Command::new(pager)
		.stdin(Stdio::piped())
		.spawn()
		.map_err(|_| ())?;
	
	if let Some(stdin) = child.stdin.as_mut() {
		stdin.write_all(output.as_bytes()).map_err(|_| ())?;
	}
	child.wait().map_err(|_| ())?;
	Ok(())
}

fn handle_output(output: &str, no_pager: bool) {
	let term_height = term_dimensions().1;
	if no_pager {
		print!("{output}")
	} else if output.lines().count() > term_height {
		match page_output(output) {
			Ok(()) => std::process::exit(0),
			Err(()) => print!("{output}")
		}
	} else {
		print!("{output}")
	}
}

fn main() {
	let cli = Cli::parse();
	let raw = read_stats();
	let no_pager = cli.no_pager;
	if cli.json {
		if !cli.commands.is_empty() {
			let mut json: Value = serde_json::from_str(&raw).unwrap_or_default();
			let filtered;
			if let Some(array) = json.as_array_mut() {
				array.retain(|obj| {
					obj.get("command")
						.and_then(Value::as_str)
						.map(|cmd| cli.commands.contains(&cmd.to_string()))
						.unwrap_or(false)
				});
				filtered = array;
			} else {
				eprintln!("Failed to convert stats file into json array. Corrupted formatting?");
				return
			}
			let filtered = serde_json::to_string_pretty(filtered).unwrap();
			println!("{filtered}")
		} else {
			println!("{raw}");
		}
		return
	}
	if cli.clear_stats {
		clear_stats();
		return
	}
	let bar_color = cli.bar_color
		.as_ref()
		.map(|s| get_color(s))
		.transpose()
		.unwrap_or_else(|e| {
			eprintln!("{e}");
			std::process::exit(1);
		});
	let mut entries: Entries = serde_json::from_str(&raw).unwrap_or_default();
	if !cli.commands.is_empty() {
		entries.0.retain(|ent| cli.commands.contains(&ent.command));
		if cli.long {
			let mut output = String::new();
			entries.sort_entries();
			for entry in entries.0 {
				writeln!(output, "{}",entry.detail_display()).unwrap();
			}
			writeln!(output, "{}", "-".repeat((term_dimensions().0 as f64 * 0.5) as usize)).unwrap();
			handle_output(&output, no_pager);
		} else {
			let mut cmd_stats = CmdStats { entries, cli };
			let output = cmd_stats.format_entries(bar_color);
			handle_output(&output, no_pager);
		}
	} else if cli.long {
		let mut output = String::new();
		entries.sort_entries();
		for entry in entries.0 {
			writeln!(output, "{}",entry.detail_display()).unwrap();
		}
		writeln!(output, "{}", "-".repeat((term_dimensions().0 as f64 * 0.5) as usize)).unwrap();
		handle_output(&output, no_pager);
	} else {
		let mut cmd_stats = CmdStats { entries, cli };
		let output = cmd_stats.format_entries(bar_color);
		handle_output(&output, no_pager);
	}
}
