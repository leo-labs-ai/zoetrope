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
