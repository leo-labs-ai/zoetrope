#!/usr/bin/env bash
# Print "<agent> <session-reference>" for the pane the plugin was invoked from, or
# exit non-zero with the reason on stderr.
#
# The pane id comes from `focused_pane_id` in HERDR_PLUGIN_CONTEXT_JSON, never
# from HERDR_PANE_ID: in a pane command HERDR_PANE_ID is the plugin's own new
# pane, and asking Herdr about that one returns a pane with no agent. The
# context names the pane that was focused when the plugin was invoked, and it
# is the same field for an action and for a pane command.
#
set -euo pipefail

herdr="${HERDR_BIN_PATH:-herdr}"
ctx="${HERDR_PLUGIN_CONTEXT_JSON:-{\}}"

command -v jq >/dev/null 2>&1 || { echo "jq is not on PATH, and the plugin reads Herdr's JSON with it" >&2; exit 1; }

pane_id=$(printf '%s' "$ctx" | jq -r '.focused_pane_id // empty')
[ -n "$pane_id" ] || { echo "no focused pane in the invocation context" >&2; exit 1; }

resp=$("$herdr" pane get "$pane_id" 2>&1) || { echo "herdr pane get failed: $resp" >&2; exit 1; }
err=$(printf '%s' "$resp" | jq -r '.error.message // empty' 2>/dev/null || true)
[ -z "$err" ] || { echo "herdr: $err" >&2; exit 1; }

# `pane.get` answers `{"result": {"pane": {...}, "type": "pane_info"}}`: the
# record is under `.result.pane`, not `.result` itself.
pane=$(printf '%s' "$resp" | jq '.result.pane // .result')
agent=$(printf '%s' "$pane" | jq -r '.agent_session.agent // .agent // empty')
kind=$(printf  '%s' "$pane" | jq -r '.agent_session.kind  // empty')
value=$(printf '%s' "$pane" | jq -r '.agent_session.value // empty')

case "$agent" in
  claude | codex | pi | prime-agent | rusty) ;;
  "") echo "pane $pane_id has no agent: focus a supported coding-agent pane" >&2; exit 1 ;;
  *)  echo "agent '$agent' in pane $pane_id is not one zoe reads (Claude Code, Codex, Pi, Prime Agent and Rusty)" >&2; exit 1 ;;
esac

if [ "$agent" = prime-agent ] && [ -z "$kind" ]; then
  cwd=$(printf '%s' "$pane" | jq -r '.cwd // empty')
  [ -n "$cwd" ] || { echo "Prime Agent pane has no cwd for session fallback" >&2; exit 1; }
  sessions="$HOME/.prime/agent/sessions"
  value=$(
    if [ -d "$sessions" ]; then
      while IFS= read -r path; do
        header=$(head -n 1 "$path")
        if printf '%s' "$header" | jq -e --arg cwd "$cwd" '.type == "session" and .version == 3 and .cwd == $cwd' >/dev/null 2>&1; then
          printf '%s' "$path"
          break
        fi
      done < <(find "$sessions" -maxdepth 1 -name '*.jsonl' -exec ls -t {} +)
    fi
  )
  [ -n "$value" ] || { echo "no Prime Agent session header matches pane cwd: $cwd" >&2; exit 1; }
  kind=path
fi

case "$kind" in
  id | path) ;;
  "") cat >&2 <<MSG
Herdr has no session id for this $agent pane.

  The id comes from the agent's SessionStart hook, which fires only when a
  session begins. So: install the integration if it is missing, then start the
  agent in that pane again. A session that was already running when the
  integration was installed never reports one.

    herdr integration install $agent    (herdr integration status lists them)
MSG
     exit 1 ;;
  *)  echo "Herdr reports a $kind for this $agent pane, and the plugin expects an id or path" >&2; exit 1 ;;
esac

[ -n "$value" ] || { echo "Herdr reports an empty session reference for pane $pane_id" >&2; exit 1; }
printf '%s %s\n' "$agent" "$value"
