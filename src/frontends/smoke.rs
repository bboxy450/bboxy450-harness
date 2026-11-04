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
    let (record, code) = match smoke(false).await {
        Ok(result) => result,
        Err(_) => (summary("FAIL", "smoke runner failed", 0), 1),
    };
    println!("{record}");
    code
}
async fn smoke(acp_mode: bool) -> Result<(Value, i32), Box<dyn std::error::Error>> {
    let redactor = Set::default();
    crate::capability::register(&redactor).await;
    if let Some(notice) = redactor.capability_notice() {
        eprintln!("Notice: {notice}");
    }
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
        auto_compact: true,
        context_tokens: None,
        mcp_servers: Default::default(),
        agent: None,
        name: "nemotron".into(),
        provider: Some("openrouter".into()),
        model: "nvidia/nemotron-3.5-super-vl:free".into(),
        endpoint: "https://openrouter.ai/api/v1".into(),
        api_key: Some(command),
        api_key_env: None,
        retry_deadline: Some(90),
        max_tokens: None,
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
        vcs: None,
        dream: None,
        source: None,
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
    let (session, ending) = if acp_mode {
        #[cfg(feature = "acp")]
        {
            acp_turn(config, &temp).await?
        }
        #[cfg(not(feature = "acp"))]
        {
            return Ok((summary("FAIL", "ACP feature unavailable", 0), 1));
        }
    } else {
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
        let result = tokio::time::timeout(Duration::from_secs(90), async {
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
        let _ = task.await;
        (session, result)
    };
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
    ) || transcript.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text.contains("RateLimited"))) {
        ("UNAVAILABLE", "provider quota unavailable", 75)
    } else if ending.is_none() {
        ("UNAVAILABLE", "live deadline expired", 75)
    } else {
        ("FAIL", "live turn did not satisfy read/edit contract", 1)
    };
    let mut record = summary(outcome, reason, requests);
    record["face"] = json!(if acp_mode { "acp" } else { "headless" });
    record["ending"] = json!(ending);
    record["check_transcript"] = json!(laws);
    record["read_calls"] = json!(reads);
    record["edit_calls"] = json!(edits);
    record["file_changed"] = json!(changed);
    tokio::fs::remove_dir_all(temp).await?;
    Ok((record, code))
}

#[cfg(feature = "acp")]
pub async fn run_acp() -> i32 {
    let (record, code) = match smoke(true).await {
        Ok(result) => result,
        Err(_) => (summary("FAIL", "ACP smoke runner failed", 0), 1),
    };
    println!("{record}");
    code
}
#[cfg(feature = "acp")]
async fn acp_turn(
    config: Config,
    cwd: &std::path::Path,
) -> Result<(SessionId, Option<StopReason>), Box<dyn std::error::Error>> {
    use tokio::io::AsyncBufReadExt;
    let stop = CancellationToken::new();
    let (client, server) = tokio::io::duplex(1024 * 1024);
    let (read, mut write) = tokio::io::split(client);
    let (read_server, write_server) = tokio::io::split(server);
    let token = stop.clone();
    let task = bbt_rt::handle().spawn(async move {
        super::acp::serve_cancelled(read_server, write_server, config, token).await
    });
    let run=async {
        let mut lines=tokio::io::BufReader::new(read).lines();
        async fn send(write:&mut tokio::io::WriteHalf<tokio::io::DuplexStream>,value:Value)->std::io::Result<()> {let mut bytes=serde_json::to_vec(&value)?;bytes.push(b'\n');write.write_all(&bytes).await}
        send(&mut write,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1}})).await?;
        let _=lines.next_line().await?;
        send(&mut write,json!({"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":cwd,"mcpServers":[]}})).await?;
        let response:Value=serde_json::from_str(&lines.next_line().await?.ok_or("ACP disconnected")?)?;
        let session=SessionId(response["result"]["sessionId"].as_str().ok_or("ACP new failed")?.parse()?);
        send(&mut write,json!({"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":session.to_string(),"prompt":[{"type":"text","text":"Use read to inspect smoke.txt, then use edit exactly once to replace before\n with after\n. Finish with a brief Done. Do not run other tools. You have at most three model requests."}]}})).await?;
        let ending=tokio::time::timeout(Duration::from_secs(90),async {
            while let Some(line)=lines.next_line().await? {
                let response:Value=serde_json::from_str(&line)?;
                if response["method"]=="session/request_permission" {send(&mut write,json!({"jsonrpc":"2.0","id":response["id"],"result":{"outcome":{"outcome":"selected","optionId":"deny"}}})).await?;}
                if response["id"]==3 {
                    let reason=if response.get("error").is_some(){StopReason::Error {class:serde_json::from_value(response["error"]["data"]["class"].clone()).unwrap_or(ErrorClass::Internal),message:"ACP live request failed".into()}}else{match response["result"]["stopReason"].as_str(){Some("end_turn")=>StopReason::EndTurn,Some("max_tokens")=>StopReason::MaxTokens,Some("max_turn_requests")=>StopReason::MaxRounds,_=>StopReason::Cancelled}};
                    return Ok::<_,Box<dyn std::error::Error>>(Some(reason));
                }
            }
            Ok(None)
        }).await.ok().transpose()?.flatten();
        Ok::<_,Box<dyn std::error::Error>>((session,ending))
    }.await;
    stop.cancel();
    let joined = task.await?;
    joined?;
    run
}
