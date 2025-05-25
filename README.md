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
-a, --all                 Display all commands (ignore limit)
-n, --num <NUM>           Number of entries to display [default: 10]
-l                        Show detailed info for each command
    --command <COMMAND>   Display stats for a single command
    --columns <COLUMNS>   Select columns to display (comma-separated)
    --sort <COLUMN>       Sort output by column
    --reverse             Reverse the sort order
    --json                Output raw JSON data
    --no-header           Omit table headers (for scripts)
-h, --help                Show help
-V, --version             Show version
```

---

## Screenshots

![cmdstat1](https://github.com/user-attachments/assets/ba2abdfe-efb0-422d-8b16-3882e3e71d10)
---
![cmdstat2](https://github.com/user-attachments/assets/ad260431-0b60-4a79-b2f3-d1af413b9598)
---
![cmdstat3](https://github.com/user-attachments/assets/6331e9bd-e011-43b4-a865-79d1ae362ca8)



---

## Notes

* The Zsh plugin only tracks commands after it has been loaded.
* The stats file is stored at `~/.local/share/cmdstat/stats.json` by default.

---

## License

MIT

---
