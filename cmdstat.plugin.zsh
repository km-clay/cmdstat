# ~/.config/zsh/custom/cmdstat/cmdstat.plugin.zsh

# Configuration (allow user override)
CMDSTAT_FILE="${CMDSTAT_FILE:-$HOME/.local/share/cmdstat/stats.json}"

cmdstat_log_command() {
	local cmd="$(basename ${1%% *})"
	local stats_dir="${CMDSTAT_FILE:h}"
	local dir="$PWD"
	local resolved=$(whence -w "$cmd" | cut -d' ' -f2)

	# Ensure directory and file exist
	# Skip if not a proper command
	[[ ! -e "$stats_dir" ]] && mkdir -p "$stats_dir"
	[[ ! -e "$CMDSTAT_FILE" ]] && echo "[]" > "$CMDSTAT_FILE"
	[[ -z $(< "$CMDSTAT_FILE") ]] && echo "[]" > "$CMDSTAT_FILE"
	[[ -z "$resolved" || "$resolved" == "none" || "$cmd" == /* || "$cmd" == ./* ]] && return

	(
		# flock for safe write
		exec {fd}>"$CMDSTAT_FILE.lock"
		flock --timeout 5 $fd

		local new_json=$(jq --arg cmd "$cmd" --arg dir "$dir" --arg kind "$resolved" \
		'if any(.[]; .command == $cmd) then
			map(if .command == $cmd then
				.count += 1
				| .dirs[$dir] = (.dirs[$dir] // 0) + 1
			else
				.
			end)
		else
			. + [
				{
					"command": $cmd,
					"count": 1,
					"kind": $kind,
					"dirs": ({ ($dir): 1 }),
				}
			]
		end' "$CMDSTAT_FILE")

	printf "%s" "$new_json" > "$CMDSTAT_FILE"

	) &!
}

if [[ ${preexec_functions[(r)cmdstat_log_command]} != "cmdstat_log_command" ]]; then
    preexec_functions+=cmdstat_log_command
fi
