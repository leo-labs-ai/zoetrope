# Harness providers

## Pi v3 (pi, pii, prime-agent)

The pi provider recognizes a version-3 session header by content. Discovery
searches ~/.pi/vanilla-agent/sessions, ~/.pi/agent/sessions,
~/.pi/pii/sessions and ~/.prime/agent/sessions. The header id and cwd, not
filenames, determine session and project identity. Assistant toolCall blocks
join toolResult messages by toolCallId; only recorded boolean isError values
state an outcome. Custom messages and child_usage_attributed records are
activity, not human prompts or duplicate token usage.

A header's parentSession is a fork's source path, not proof of a spawned
agent (observer and suggested-task forks also use it). Entries' parentId
links transcript entries, not agents. Without an explicit subagent join,
these sessions remain independent roots rather than inventing child nodes.
Prime's child_usage_attributed targets usage attribution, not session
parentage. Subagent protocols beyond these observed records are not modeled.

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
