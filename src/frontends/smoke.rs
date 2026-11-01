// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    engine::{Config, SessionActor},
    gateway::{Gateway, Subscribe},
    policy::{Policy, Rule, RuleDecision},
    protocol::*,
    provider::{
        OpenAiCompat, Options,
        profile::{Profile, Unattended},
    },
    redact::Set,
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;
fn summary(outcome: &str, reason: &str, requests: u32) -> Value {
    json!({"outcome":outcome,"reason":reason,"requests":requests,"model":"nvidia/nemotron-3.5-super-vl:free","redacted":true})
}
pub async fn run() -> i32 {
    let (record, code) = match smoke().await {
        Ok(result) => result,
        Err(_) => (summary("FAIL", "smoke runner failed", 0), 1),
    };
    println!("{record}");
    code
}
async fn smoke() -> Result<(Value, i32), Box<dyn std::error::Error>> {
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?);
    let old = home.join(".kittens/profiles/nemotron.md");
    let old = match tokio::fs::read_to_string(old).await {
        Ok(s) => s,
        Err(_) => return Ok((summary("SKIP", "key profile unavailable", 0), 0)),
    };
    let original = match Profile::parse(&old) {
        Ok(p) => p,
        Err(_) => return Ok((summary("SKIP", "key profile invalid", 0), 0)),
    };
    let Some(command) = original.api_key.filter(|s| s.starts_with('!')) else {
        return Ok((summary("SKIP", "no key command configured", 0), 0));
    };
    let profile = Profile {
        name: "nemotron".into(),
        provider: Some("openrouter".into()),
        model: "nvidia/nemotron-3.5-super-vl:free".into(),
        endpoint: "https://openrouter.ai/api/v1".into(),
        api_key: Some(command),
        api_key_env: None,
        retry_deadline: Some(90),
        on_unattended: Unattended::Deny,
        policy: None,
    };
    let path = home.join(".bboxy450-harness/profiles/nemotron.md");
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    let text = format!("---\n{}---\n", serde_yaml::to_string(&profile)?);
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .await?;
    f.write_all(text.as_bytes()).await?;
    f.flush().await?;
    let redactor = Set::default();
    if let Ok(cap) = std::env::var("BBOXY_CAPABILITY") {
        redactor.register(&cap);
    }
    let key = match profile.key(&redactor, CancellationToken::new()).await {
        Ok(k) => k,
        Err(_) => {
            return Ok((
                summary("SKIP", "key command unavailable or requires interaction", 0),
                0,
            ));
        }
    };
    let client = Arc::new(OpenAiCompat::new(
        &profile.endpoint,
        key,
        redactor.clone(),
        Options {
            retry_deadline: Some(Duration::from_secs(90)),
            max_requests: Some(
                std::env::var("HARNESS_SMOKE_REQUEST_BUDGET")
                    .ok()
                    .and_then(|n| n.parse::<u32>().ok())
                    .unwrap_or(3)
                    .min(3),
            ),
            ..Default::default()
        },
    )?);
    let temp = std::env::temp_dir().join(format!("harness-live-{}", ulid::Ulid::new()));
    tokio::fs::create_dir(&temp).await?;
    tokio::fs::write(temp.join("smoke.txt"), "before\n").await?;
    let session = SessionId::new();
    let sessions = temp.join("sessions");
    let meta = SessionMeta {
        session,
        cwd: temp.clone(),
        profile: Some("nemotron".into()),
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: crate::store::now_ms(),
    };
    let policy = Policy {
        on_unattended: Unattended::Deny,
        rules: vec![
            Rule {
                id: "smoke-deny-other".into(),
                tool: "*".into(),
                effect: None,
                decision: RuleDecision::Deny,
            },
            Rule {
                id: "smoke-read".into(),
                tool: "read".into(),
                effect: None,
                decision: RuleDecision::Allow,
            },
            Rule {
                id: "smoke-edit".into(),
                tool: "edit".into(),
                effect: None,
                decision: RuleDecision::Allow,
            },
        ],
    };
    let config = Config {
        meta,
        sessions: sessions.clone(),
        profiles: home.join(".bboxy450-harness/profiles"),
        provider: client.clone(),
        model: profile.model,
        policy,
        redactor,
        profile_error: None,
        max_rounds: 3,
    };
    let (g, task) = SessionActor::open(config, OpId::new()).await?;
    let mut sub = g
        .subscribe(Subscribe {
            session,
            after: None,
            ephemeral: false,
        })
        .map_err(|_| "subscribe refused")?;
    let op = |op| OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op,
    };
    g.send(op(Op::UserTurn{blocks:vec![InputBlock::Text{text:"Use read to inspect smoke.txt, then use edit exactly once to replace the full string before\n with after\n. Finish with a brief Done. Do not run any other tools. You have at most three model requests.".into()}],mode:TurnMode::Queue})).map_err(|_|"turn refused")?;
    let ending = tokio::time::timeout(Duration::from_secs(90), async {
        while let Some(e) = sub.recv().await {
            if let Event::TurnEnded { reason, .. } = e.event {
                return Some(reason);
            }
        }
        None
    })
    .await
    .ok()
    .flatten();
    let _ = g.send(op(Op::Close));
    let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    let text = tokio::fs::read_to_string(sessions.join(format!("{session}.jsonl"))).await?;
    let transcript = text
        .lines()
        .map(serde_json::from_str::<Envelope>)
        .collect::<Result<Vec<_>, _>>()?;
    let laws = crate::protocol::laws::check_transcript(&transcript).is_ok();
    let mut reads = 0;
    let mut edits = 0;
    for e in &transcript {
        if let Event::ItemCompleted {
            status: ItemStatus::Completed,
            body: Item::ToolCall { name, .. },
            ..
        } = &e.event
        {
            if name == "read" {
                reads += 1;
            }
            if name == "edit" {
                edits += 1;
            }
        }
    }
    let changed = tokio::fs::read_to_string(temp.join("smoke.txt")).await? == "after\n";
    let requests = client.request_count();
    let (outcome, reason, code) = if matches!(ending, Some(StopReason::EndTurn))
        && laws
        && reads >= 1
        && edits == 1
        && changed
        && requests <= 3
    {
        ("PASS", "read and edit completed; transcript laws held", 0)
    } else if matches!(
        ending,
        Some(StopReason::Error {
            class: ErrorClass::RateLimited,
            ..
        })
    ) {
        ("UNAVAILABLE", "provider quota unavailable", 75)
    } else if ending.is_none() {
        ("UNAVAILABLE", "live deadline expired", 75)
    } else {
        ("FAIL", "live turn did not satisfy read/edit contract", 1)
    };
    let mut record = summary(outcome, reason, requests);
    record["ending"] = json!(ending);
    record["check_transcript"] = json!(laws);
    record["read_calls"] = json!(reads);
    record["edit_calls"] = json!(edits);
    record["file_changed"] = json!(changed);
    tokio::fs::remove_dir_all(temp).await?;
    Ok((record, code))
}
