use crate::fact::{AgentKind, AgentStatus, Fact, FactKind, Outcome, Statement};
use chrono::{DateTime, Utc};

pub mod discovery;
pub mod wire;

#[derive(Debug, Clone)]
pub struct Stream {
    owner: String,
    adopted: bool,
}

impl Stream {
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            adopted: false,
        }
    }

    pub fn push(&mut self, line: &str) -> Option<Statement> {
        let v = wire::parse_line(line)?;
        let ts = v["ts_ms"]
            .as_i64()
            .and_then(DateTime::<Utc>::from_timestamp_millis);
        let d = &v["data"];
        let mut facts = Vec::new();
        let by = |kind| Fact {
            agent: Some(self.owner.clone()),
            ts,
            kind,
        };
        if wire::is_header(&v) {
            if self.adopted {
                return None;
            }
            self.adopted = true;
            let main = self.owner == "main";
            facts.push(by(FactKind::Agent {
                kind: if main {
                    AgentKind::Main
                } else {
                    AgentKind::Subagent
                },
                parent: None,
                agent_type: Some("rusty".into()),
                description: None,
                spawned_by: None,
                interactive: main,
            }));
            if let Some(model) = d["model"].as_str() {
                facts.push(by(FactKind::Model(model.into())));
            }
        } else if !self.adopted {
            return None;
        } else {
            match v["kind"].as_str()? {
                "effect.intent" => {
                    if let (Some(call), Some(name)) =
                        (d["tool_use_id"].as_str(), d["tool"].as_str())
                    {
                        facts.push(by(FactKind::ToolStart {
                            call: call.into(),
                            name: name.into(),
                            summary: crate::provider::pi::summary(&d["input"]),
                        }));
                    }
                }
                "effect.outcome" => {
                    if let (Some(call), Some(status)) =
                        (d["tool_use_id"].as_str(), d["status"].as_str())
                    {
                        let outcome = match status {
                            "ok" => Some(Outcome::Ok),
                            "error" | "failed" | "err" => Some(Outcome::Err),
                            _ => None,
                        };
                        if let Some(outcome) = outcome {
                            facts.push(by(FactKind::ToolEnd {
                                call: call.into(),
                                outcome,
                            }));
                        }
                    }
                }
                "assistant.text" => {
                    if let Some(s) = d["text"].as_str() {
                        facts.push(by(FactKind::Reasoning(s.into())));
                    }
                }
                "input.admitted" if d["source"] == "user" => {
                    if let Some(s) = d["text"].as_str() {
                        facts.push(by(FactKind::Prompt(s.into())));
                    }
                }
                "provider.usage" => {
                    if let Some(output) = d["usage"]["output_tokens"].as_u64() {
                        facts.push(by(FactKind::Tokens {
                            output,
                            dedup: v["seq"].as_u64().map(|s| s.to_string()),
                        }));
                    }
                }
                "child.spawned" => {
                    if let Some(child) = d["child_id"].as_str() {
                        facts.push(Fact {
                            agent: Some(child.into()),
                            ts,
                            kind: FactKind::Agent {
                                kind: AgentKind::Subagent,
                                parent: Some(self.owner.clone()),
                                agent_type: Some("rusty".into()),
                                description: d["name"].as_str().map(str::to_owned),
                                spawned_by: None,
                                interactive: false,
                            },
                        });
                    }
                }
                "child.outcome" => {
                    if let (Some(child), Some(status)) =
                        (d["child_id"].as_str(), d["status"].as_str())
                    {
                        let end = match status {
                            "completed" => Some(AgentStatus::Done),
                            "failed" => Some(AgentStatus::Failed),
                            "stopped" | "cancelled" => Some(AgentStatus::Stopped),
                            _ => None,
                        };
                        if let Some(end) = end {
                            facts.push(Fact {
                                agent: Some(child.into()),
                                ts,
                                kind: FactKind::Ended(end),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        if facts.is_empty() {
            facts.push(by(FactKind::Activity));
        }
        Some(Statement { at: ts, facts })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn conformance() {
        crate::provider::harness::conform("rusty", "synthetic", || {
            let dir = crate::provider::harness::fixture_dir("rusty")
                .unwrap()
                .join("synthetic");
            crate::provider::pi::discovery::paths(&dir, 2)
                .into_iter()
                .map(|p| {
                    let file = super::discovery::session_file(&p).unwrap();
                    let mut s = super::discovery::stream_for(&file);
                    std::fs::read_to_string(p)
                        .unwrap()
                        .lines()
                        .filter_map(|l| s.push(l))
                        .collect()
                })
                .collect()
        });
    }
}
