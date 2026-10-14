#!/usr/bin/env bash
# Pane command: resolve the session of the pane this was opened from, and run
# `zoe` on it. Every failure is printed here and held until you press enter,
# since this terminal is what the user is looking at.
set -euo pipefail

root="${HERDR_PLUGIN_ROOT:-.}"

die() {
  printf '\n  %s\n\n' "$*" >&2
  read -r -p '  press enter to close ' _ || true
  exit 1
}

zoe="$root/bin/zoe"
[ -x "$zoe" ] || die "fork zoe is missing; reinstall the plugin to build it"

if ! target=$(bash "$root/herdr/resolve.sh" 2>&1); then
  die "$target"
fi

# The pane's agent is running, so ride the live edge; space and the scrubber
# go back.
"$zoe" --follow "${target#* }" \
  || die "zoe exited with status $? (zoe --follow ${target#* })"
