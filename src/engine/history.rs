// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{protocol::*, provider::WireMessage};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn output_text(output: &[OutputBlock]) -> String {
    output
        .iter()
        .map(|b| match b {
            OutputBlock::Text { text } | OutputBlock::Error { text } => text.clone(),
            OutputBlock::Json { value } => value.to_string(),
            OutputBlock::Diff { unified, .. } => unified.clone(),
            OutputBlock::Image { blob } => format!("image blob {}", blob.sha256),
            OutputBlock::Unknown => "unknown output".into(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn input(blocks: &[InputBlock]) -> Value {
    json!(
        blocks
            .iter()
            .filter_map(|b| match b {
                InputBlock::Text { text } => Some(text.clone()),
                InputBlock::Mention { path } => Some(format!("@{}", path.display())),
                InputBlock::Image { blob } => Some(format!("image blob {}", blob.sha256)),
                InputBlock::Unknown => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    )
}
/// The log's completed tool bodies are the single source for provider history.
/// Start order is retained even if parallel completions arrived in reverse order.
pub fn to_wire(log: &[Envelope]) -> Vec<WireMessage> {
    let mut bodies = BTreeMap::new();
    for e in log {
        if let Event::ItemCompleted { item, body, .. } = &e.event {
            bodies.insert(*item, body);
        }
    }
    let mut out = vec![];
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut opaque = None;
    let mut calls: Vec<Value> = vec![];
    let mut results: Vec<Value> = vec![];
    fn flush(
        out: &mut Vec<Value>,
        text: &mut String,
        reasoning: &mut String,
        opaque: &mut Option<Value>,
        calls: &mut Vec<Value>,
        results: &mut Vec<Value>,
    ) {
        if !text.is_empty() || !reasoning.is_empty() || !calls.is_empty() {
            let mut a = json!({"role":"assistant","content":std::mem::take(text)});
            if !reasoning.is_empty() {
                a["reasoning"] = json!(std::mem::take(reasoning));
            }
            if let Some(v) = opaque.take() {
                a["reasoning_details"] = v;
            }
            if !calls.is_empty() {
                a["tool_calls"] = json!(std::mem::take(calls));
            }
            out.push(a);
            out.append(results);
        }
    }
    for e in log {
        match &e.event {
            Event::ItemStarted { item, .. } => {
                let Some(body) = bodies.get(item) else {
                    continue;
                };
                match body {
                    Item::UserMessage { blocks } => {
                        flush(
                            &mut out,
                            &mut text,
                            &mut reasoning,
                            &mut opaque,
                            &mut calls,
                            &mut results,
                        );
                        out.push(json!({"role":"user","content":input(blocks)}));
                    }
                    Item::AgentMessage { markdown } => {
                        if !calls.is_empty() {
                            flush(
                                &mut out,
                                &mut text,
                                &mut reasoning,
                                &mut opaque,
                                &mut calls,
                                &mut results,
                            );
                        }
                        text.push_str(markdown);
                    }
                    Item::Reasoning {
                        summary,
                        provider_opaque,
                        ..
                    } => {
                        reasoning.push_str(summary);
                        opaque = provider_opaque.clone();
                    }
                    Item::ToolCall {
                        call_id,
                        name,
                        input_raw,
                        output,
                        ..
                    } => {
                        calls.push(json!({"id":call_id,"type":"function","function":{"name":name,"arguments":input_raw}}));
                        results.push(json!({"role":"tool","tool_call_id":call_id,"content":output_text(output)}));
                    }
                    _ => {}
                }
            }
            Event::TurnEnded { .. } => flush(
                &mut out,
                &mut text,
                &mut reasoning,
                &mut opaque,
                &mut calls,
                &mut results,
            ),
            _ => {}
        }
    }
    flush(
        &mut out,
        &mut text,
        &mut reasoning,
        &mut opaque,
        &mut calls,
        &mut results,
    );
    out
}
pub fn check_wire(messages: &[WireMessage]) -> Result<(), String> {
    let mut pending = std::collections::BTreeSet::new();
    let mut seen = std::collections::BTreeSet::new();
    for m in messages {
        if m["role"] == "tool" {
            let id = m["tool_call_id"].as_str().ok_or("missing tool result id")?;
            if !pending.remove(id) {
                return Err("orphan tool result".into());
            }
        } else {
            if !pending.is_empty() {
                return Err("unpaired call before next message".into());
            }
            if let Some(calls) = m["tool_calls"].as_array() {
                for call in calls {
                    let id = call["id"].as_str().ok_or("missing call id")?;
                    if !seen.insert(id.to_string()) || !pending.insert(id.to_string()) {
                        return Err("duplicate tool id".into());
                    }
                }
            }
        }
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err("unpaired calls".into())
    }
}
