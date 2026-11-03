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
            other => other.display_text(),
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
                InputBlock::Unknown(_) => None,
                other => Some(other.display_text()),
            })
            .collect::<Vec<_>>()
            .join("\n")
    )
}
/// The log's completed tool bodies are the single source for provider history.
/// Start order is retained even if parallel completions arrived in reverse order.
pub fn to_wire(log: &[Envelope]) -> Vec<WireMessage> {
    tagged(log)
        .into_iter()
        .map(|(_, message)| message)
        .collect()
}
/// The origin is ItemStarted, including for the paired tool result (§4d).
pub fn tagged(log: &[Envelope]) -> Vec<(Seq, WireMessage)> {
    let bodies: BTreeMap<_, _> = log
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted { item, body, .. } => Some((*item, body)),
            _ => None,
        })
        .collect();
    let mut out = vec![];
    let mut assistant = json!({"role":"assistant","content":""});
    let mut origin = Seq(0);
    let mut results = vec![];
    fn flush(
        out: &mut Vec<(Seq, Value)>,
        assistant: &mut Value,
        origin: &mut Seq,
        results: &mut Vec<(Seq, Value)>,
    ) {
        if *origin != Seq(0) {
            out.push((
                *origin,
                std::mem::replace(assistant, json!({"role":"assistant","content":""})),
            ));
            out.append(results);
            *origin = Seq(0);
        }
    }
    for e in log {
        match &e.event {
            Event::ItemStarted { item, .. } => {
                let Some(body) = bodies.get(item) else {
                    continue;
                };
                let seq = e.seq.unwrap_or(Seq(0));
                match body {
                    Item::UserMessage { blocks } => {
                        flush(&mut out, &mut assistant, &mut origin, &mut results);
                        out.push((seq, json!({"role":"user","content":input(blocks)})));
                    }
                    Item::AgentMessage { markdown } => {
                        if assistant.get("tool_calls").is_some() {
                            flush(&mut out, &mut assistant, &mut origin, &mut results);
                        }
                        if !markdown.is_empty() {
                            let text =
                                assistant["content"].as_str().unwrap_or("").to_owned() + markdown;
                            assistant["content"] = json!(text);
                            origin = origin.max(seq);
                        }
                    }
                    Item::Reasoning {
                        summary,
                        provider_opaque,
                        ..
                    } => {
                        if !summary.is_empty() || provider_opaque.is_some() {
                            if !summary.is_empty() {
                                let text = assistant["reasoning"].as_str().unwrap_or("").to_owned()
                                    + summary;
                                assistant["reasoning"] = json!(text);
                            }
                            if let Some(value) = provider_opaque {
                                assistant["reasoning_details"] = value.clone();
                            }
                            origin = origin.max(seq);
                        }
                    }
                    Item::ToolCall {
                        call_id,
                        name,
                        input_raw,
                        output,
                        ..
                    } => {
                        if assistant.get("tool_calls").is_none() {
                            assistant["tool_calls"] = json!([]);
                        }
                        assistant["tool_calls"].as_array_mut().unwrap().push(json!({"id":call_id,"type":"function","function":{"name":name,"arguments":input_raw}}));
                        origin = origin.max(seq);
                        results.push((seq,json!({"role":"tool","tool_call_id":call_id,"content":output_text(output)})));
                    }
                    _ => {}
                }
            }
            Event::TurnEnded { .. } => flush(&mut out, &mut assistant, &mut origin, &mut results),
            _ => {}
        }
    }
    flush(&mut out, &mut assistant, &mut origin, &mut results);
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
