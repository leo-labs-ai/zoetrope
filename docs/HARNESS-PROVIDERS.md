# Harness providers

## Pi v3 (pi, pii, prime-agent)

The pi provider recognizes a version-3 session header by content. Discovery
searches ~/.pi/vanilla-agent/sessions, ~/.pi/agent/sessions,
~/.pi/pii/sessions and ~/.prime/agent/sessions. The header id and cwd, not
filenames, determine session and project identity. Assistant toolCall blocks
join toolResult messages by toolCallId; only recorded boolean isError values
state an outcome. Custom messages and child_usage_attributed records are
activity, not human prompts or duplicate token usage.

A header's parentSession names the source file. Native discovery follows that
chain to the root (cycle-safe) and joins linked files in the same session
folder. The installed pi-subagents extension writes it for persisted children;
other extensions use it for forks and observers. All linked nodes are therefore
interactive subagents: parentage is recorded, but the field alone cannot prove
spawn provenance or completion. In-memory children have no transcript to read.
Custom session directories outside the parent's folder are not discovered from
the root; opening the child directly follows its parent chain. Browser head-only
classification currently treats Pi files as independent roots.

Entries' parentId links transcript entries, not agents. Prime's
child_usage_attributed has origin spawn_task and a target entry id, not a
child session id. It is not sufficient to join a child transcript.

## Rusty

The first session.created event identifies the format. Session identity comes
from the enclosing session directory; sub-*.jsonl logs can live directly in
it or inside sub-* directories. The session's cwd sidecar supplies project
identity. child.spawned states parentage and child.outcome states terminal
status; child headers never guess a parent. effect.intent/outcome join by
tool_use_id. assistant.text is reasoning; input.admitted from a user is a
prompt (input.queued is not another copy of it). provider.usage adds output
tokens. Turn endings are not session endings. No new core facts were needed;
the UI only gained exhaustive provider-mark arms.

## Herdr and Prime Agent

The bridge accepts id and path references for all supported agents, with no
forced provider: file content wins even when Rusty is reported as pi. Missing
Prime Agent references fall back to the most recently modified JSONL file
whose v3 header cwd exactly matches the pane cwd. This is a heuristic when
several sessions share a cwd; explicit reporting should replace it.

Preferred external change (not landed here): in Prime Agent's Herdr
integration, after creating or resuming a session, run
herdr pane report-agent-session with agent prime-agent, kind path, and the
session JSONL path. Clear/update that report on session switches. Verify the
installed Herdr CLI argument schema before implementing it. Prime Agent is
an independently installed product, not source in this repository.

The installed Prime product is the Rust build at ~/.local/share/prime-agent
(version 0.10.0), launched by ~/.local/bin/prime-agent. No editable Prime
integration source was found in this checkout; do not patch its binary.
The verified CLI shape for the proposed integration is:

    herdr pane report-agent-session "$HERDR_PANE_ID" --source prime-agent --agent prime-agent --agent-session-path "$session_file"

Invoke at session creation/resume/switch, only in a Herdr pane.
