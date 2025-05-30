# cmdstat

A simple and composable command usage statistics tool for Zsh. Tracks which commands you use, how often, and where you use them from.

This project consists of:

* **cmdstat (Rust CLI program)** — Displays command usage statistics in a clean and scriptable way.
* **cmdstat.plugin.zsh (Zsh plugin)** — Captures command usage in real-time and writes statistics for `cmdstat` to read.

---

## Dependencies

* This plugin relies on `jq` to serialize command statistics.

---

## Features

* Human-friendly tabular output
* Machine-friendly modes (JSON, column selection, no-header)
* Per-command and per-directory usage stats
* Zsh plugin for automatic logging
* Easy integration into scripts

---

## Installation

### Install the CLI tool

First, build `cmdstat` with Rust:

```bash
cargo build --release
install -Dm755 target/release/cmdstat ~/.local/bin/  # or any directory in your PATH
```

### Install the Zsh plugin

Copy the plugin to your Zsh configuration directory:

```bash
cp plugin/cmdstat.plugin.zsh ~/.config/zsh/custom/cmdstat.plugin.zsh
```

Then, source it in your `.zshrc`:

```bash
source ~/.config/zsh/custom/cmdstat.plugin.zsh
```

This will enable automatic command logging to `~/.local/share/cmdstat/stats.json`.

---

## Usage

### View top commands

```bash
cmdstat
```

### Show all commands

```bash
cmdstat --all
```

### Machine-friendly output (for scripts)

```bash
cmdstat --no-header --columns command,count
```

### JSON output

```bash
cmdstat --json
```

---

## Options

```
Usage: cmdstat [OPTIONS] [COMMANDS]...

Arguments:
  [COMMANDS]...  Display statistics for specific commands

Options:
  -a, --all                    Display all commands from the stats file. Ignores --num.
  -n, --num <NUM>              Choose a specific number of commands to show. [default: 20]
  -l                           Display extra info about each command
      --columns <COLUMNS>      Specify which columns to display
      --sort <SORT>            Specify which column to sort by
      --reverse                Reverse the sort
      --json                   Dump raw json
      --no-header              Omit the table headers
      --bar-color <BAR_COLOR>  Choose a custom bar color
      --no-pager               
      --clear-stats            
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

---

## Screenshots

![cmdstat1](https://github.com/user-attachments/assets/ba2abdfe-efb0-422d-8b16-3882e3e71d10)
---
![cmdstat2](https://github.com/user-attachments/assets/ad260431-0b60-4a79-b2f3-d1af413b9598)
---
![cmdstat3](https://github.com/user-attachments/assets/6331e9bd-e011-43b4-a865-79d1ae362ca8)

## Notes

* The stats file is saved to `~/.local/share/cmdstat/stats.json`. The `$CMDSTAT_FILE` environment variable can override this path.
* Only commands executed interactively will be tracked by the plugin. Commands executed in scripts will not be written to the stats file.

---

## Why this instead of zsh_stats?
Zsh technically does include a builtin function called `zsh_stats` which prints command usage statistics to the terminal. However, the numbers that `zsh_stats` bases it's output on comes from reading your command history, which may not actually be an accurate representation of your command usage. For instance, running the same command several times in a row will not be counted in zsh_stats if you have HIST_IGNORE_DUPS set. I decided that I want a higher resolution image of my command usage statistics, so I wrote this thing.
