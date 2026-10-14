use crate::fact::{AgentKind, Fact, FactKind, Outcome, Statement};
use crate::provider::summary::truncate_summary;
use chrono::{DateTime, Utc};

pub mod discovery;
pub mod wire;

#[derive(Debug, Clone, Default)]
pub struct Stream {
    adopted: bool,
}

impl Stream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, line: &str) -> Option<Statement> {
        let v = wire::parse_line(line)?;
        let ts = v["timestamp"]
            .as_str()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());
        let mut kinds = Vec::new();
        if wire::is_header(&v) {
            if self.adopted {
                return None;
            }
            self.adopted = true;
            kinds.push(FactKind::Agent {
                kind: AgentKind::Main,
                parent: None,
                agent_type: Some("pi".into()),
                description: None,
                spawned_by: None,
                interactive: true,
            });
        } else if !self.adopted {
            return None;
        } else {
            match v["type"].as_str()? {
                "model_change" => {
                    if let Some(model) = v["modelId"].as_str() {
                        kinds.push(FactKind::Model(model.into()));
                    }
                }
                "message" => {
                    let m = &v["message"];
                    match m["role"].as_str() {
                        Some("user") => {
                            let text = text_content(&m["content"]);
                            if !text.is_empty() {
                                kinds.push(FactKind::Prompt(text));
                            }
                        }
                        Some("assistant") => {
                            if let Some(model) = m["model"].as_str() {
                                kinds.push(FactKind::Model(model.into()));
                            }
                            if let Some(n) = m["usage"]["output"].as_u64() {
                                kinds.push(FactKind::Tokens {
                                    output: n,
                                    dedup: v["id"].as_str().map(str::to_owned),
                                });
                            }
                            if let Some(blocks) = m["content"].as_array() {
                                for b in blocks {
                                    match b["type"].as_str() {
                                        Some("text" | "thinking") => {
                                            if let Some(s) = b["text"]
                                                .as_str()
                                                .or_else(|| b["thinking"].as_str())
                                            {
                                                kinds.push(FactKind::Reasoning(s.into()));
                                            }
                                        }
                                        Some("toolCall") => {
                                            if let (Some(id), Some(name)) =
                                                (b["id"].as_str(), b["name"].as_str())
                                            {
                                                kinds.push(FactKind::ToolStart {
                                                    call: id.into(),
                                                    name: name.into(),
                                                    summary: summary(&b["arguments"]),
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Some("toolResult") => {
                            if let (Some(id), Some(error)) =
                                (m["toolCallId"].as_str(), m["isError"].as_bool())
                            {
                                kinds.push(FactKind::ToolEnd {
                                    call: id.into(),
                                    outcome: if error { Outcome::Err } else { Outcome::Ok },
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if kinds.is_empty() {
            kinds.push(FactKind::Activity);
        }
        Some(Statement {
            at: ts,
            facts: kinds
                .into_iter()
                .map(|kind| Fact {
                    agent: Some("main".into()),
                    ts,
                    kind,
                })
                .collect(),
        })
    }
}

fn text_content(v: &serde_json::Value) -> String {
    if let Some(s) = v.as_str() {
        return s.into();
    }
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

pub(crate) fn summary(v: &serde_json::Value) -> Option<String> {
    ["command", "path", "query", "description"]
        .into_iter()
        .find_map(|k| v[k].as_str())
        .map(truncate_summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conformance() {
        crate::provider::harness::conform("pi", "synthetic", || {
            let mut s = Stream::new();
            vec![
                include_str!("../../../assets/pi/synthetic.jsonl")
                    .lines()
                    .filter_map(|l| s.push(l))
                    .collect(),
            ]
        });
    }
    #[test]
    fn defensive() {
        let mut s = Stream::new();
        assert!(s.push("broken").is_none());
        assert!(s.push(r#"{"type":"message"}"#).is_none());
    }

    #[test]
    fn discovery_uses_content() {
        let head = r#"{"type":"session","version":3,"id":"session-id","cwd":"/project"}"#;
        assert_eq!(
            crate::provider::provider_of(head),
            Some(crate::provider::Provider::Pi)
        );
        let file = discovery::classify_head(
            std::path::Path::new("arbitrary.txt"),
            head,
            std::time::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(file.session, "session-id");
        assert_eq!(
            file.project_key,
            discovery::project_key(std::path::Path::new("/project"))
        );
        assert!(
            discovery::classify_head(std::path::Path::new("x"), "{}", std::time::UNIX_EPOCH)
                .is_none()
        );
    }
}
