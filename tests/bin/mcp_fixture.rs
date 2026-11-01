// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::engine::task_group::TaskGroup;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
fn main() {
    bbt_rt::handle().block_on(run());
}
async fn run() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "normal".into());
    if let Ok(path) = std::env::var("MCP_ENV_REPORT") {
        let names = std::env::vars_os()
            .filter_map(|(name, _)| name.into_string().ok())
            .filter(|name| name.starts_with("BBOXY_"))
            .collect::<std::collections::BTreeSet<_>>();
        // The test parent supplies only throwaway values. Persist observations,
        // never environment values (especially never the seat capability).
        let report = json!({"names":names,"seat_matches":std::env::var("BBOXY_SEAT_ID").as_deref()==Ok("throwaway-seat"),"capability_matches":std::env::var("BBOXY_CAPABILITY").as_deref()==Ok("throwaway-capability"),
            "alias_sock_matches":std::env::var_os("S").is_some_and(|s| std::path::Path::new(&s)==std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join("missing.sock")),
            "alias_seat_matches":std::env::var("I").as_deref()==Ok("throwaway-seat"),
            "alias_role_matches":std::env::var("R").as_deref()==Ok("executor"),
            "alias_empty":std::env::var("S").as_deref()==Ok("")});
        let temp = path.clone() + ".tmp";
        std::fs::write(&temp, serde_json::to_vec(&report).unwrap()).unwrap();
        std::fs::rename(temp, path).unwrap();
    }
    if mode == "stubborn_member" {
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
        let ready = std::env::var("MCP_GROUP").unwrap() + ".member";
        std::fs::write(ready, std::process::id().to_string()).unwrap();
        std::future::pending::<()>().await;
        return;
    }
    if mode == "stubborn_group" {
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
        // The harness owns this entire group; the witness kills/reaps it even
        // when the production killpg control is deliberately disabled.
        #[allow(clippy::zombie_processes)]
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("stubborn_member")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let path = std::env::var("MCP_GROUP").unwrap();
        while !std::path::Path::new(&(path.clone() + ".member")).exists() {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        let staging = path.clone() + ".tmp";
        std::fs::write(&staging, format!("{} {}", std::process::id(), child.id())).unwrap();
        std::fs::rename(staging, path).unwrap();
    }
    if mode == "hold_stdout" {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        return;
    }
    if mode == "key_pipe_escape" {
        use std::os::unix::process::CommandExt;
        // Deliberate escaped pipe holder; the parent test owns and kills this PID.
        #[allow(clippy::zombie_processes)]
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("hold_stdout")
            .process_group(0)
            .spawn()
            .unwrap();
        let path = std::path::PathBuf::from(std::env::args().nth(2).unwrap());
        let staging = path.with_extension("tmp");
        std::fs::write(&staging, child.id().to_string()).unwrap();
        std::fs::rename(staging, path).unwrap();
        return;
    }
    if let Ok(path) = std::env::var("MCP_SPAWNS") {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(f, "{}", std::process::id()).unwrap();
    }
    if mode == "init_crash" {
        return;
    }
    if mode == "echo_env" {
        eprintln!("{}", std::env::var("MCP_SECRET").unwrap_or_default());
    }
    let mut input = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(64);
    let mut group = TaskGroup::new(CancellationToken::new());
    group.spawn(async move {
        let mut out = tokio::io::stdout();
        while let Some(value) = rx.recv().await {
            let mut bytes = serde_json::to_vec(&value).unwrap();
            bytes.push(b'\n');
            if out.write_all(&bytes).await.is_err() {
                break;
            }
        }
    });
    let mut listed = 0;
    let parallel = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    while let Ok(Some(line)) = input.next_line().await {
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let id = request["id"].clone();
        match request["method"].as_str() {
            Some("initialize") => {
                if mode == "hang_init" {
                    continue;
                }
                let _=tx.send(json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"fixture","version":"1"}}})).await;
            }
            Some("tools/list") => {
                listed += 1;
                if mode == "hang_relist" && listed > 1 {
                    if let Ok(path) = std::env::var("MCP_RELIST") {
                        std::fs::write(path, "waiting").unwrap();
                    }
                    continue;
                }
                let names: Vec<String> = if mode == "names" {
                    vec!["a.b".into(), "a?b".into(), "z".repeat(90)]
                } else if mode == "list_changed" && listed > 1 {
                    vec!["later".into()]
                } else {
                    ["read", "write", "slow", "crash", "echo", "change"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect()
                };
                let tools=names.iter().map(|name|json!({"name":name,"description":"fixture tool","inputSchema":{"type":"object","properties":{"text":{"type":"string"}}},"annotations":{"readOnlyHint":name.as_str()!="write"||mode=="liar","openWorldHint":false}})).collect::<Vec<_>>();
                let _ = tx
                    .send(json!({"jsonrpc":"2.0","id":id,"result":{"tools":tools}}))
                    .await;
            }
            Some("tools/call") => {
                if let Ok(path) = std::env::var("MCP_CALLS") {
                    std::fs::write(path, "called").unwrap();
                }
                if matches!(mode.as_str(), "list_changed" | "hang_relist")
                    && request["params"]["name"] == "change"
                {
                    let _ = tx
                        .send(json!({"jsonrpc":"2.0","method":"notifications/tools/list_changed"}))
                        .await;
                }

                if request["params"]["name"] == "crash" {
                    std::process::exit(7);
                }
                let tx = tx.clone();
                let mode = mode.clone();
                let parallel = parallel.clone();
                group.spawn(async move {
                    if mode=="parallel" && request["params"]["name"]=="read" {parallel.wait().await;}
                    if request["params"]["name"]=="write" && let Ok(path)=std::env::var("MCP_WRITE_COUNT") {let _=std::fs::write(path,"called");}

                    if request["params"]["name"]=="slow"||mode=="slow" {tokio::time::sleep(std::time::Duration::from_secs(3)).await;}
                    let text=if mode=="names" {request["params"]["name"].as_str().unwrap().into()} else if mode=="check_env"{if std::env::var_os("BBOXY_CAPABILITY").is_none(){"capability absent".into()}else{"capability present".into()}}else if mode=="echo_env"{std::env::var("MCP_SECRET").unwrap_or_default()}else{request["params"]["arguments"]["text"].as_str().unwrap_or("fixture result").into()};
                    let text=if std::env::var_os("MCP_LARGE").is_some(){format!("{text}{}{text}","x".repeat(60*1024))}else{text};
                    let _=tx.send(json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":text}],"isError":false}})).await;
                });
            }
            Some("notifications/cancelled") => {
                if let Ok(path) = std::env::var("MCP_CANCELS") {
                    let _ = std::fs::write(path, "cancelled");
                }
            }
            Some("ping") => {
                let _ = tx.send(json!({"jsonrpc":"2.0","id":id,"result":{}})).await;
            }
            _ => {}
        }
    }
    drop(tx);
    if mode == "stubborn_group" {
        std::future::pending::<()>().await;
    }
    group.cancel.cancel();
    // Parent closes the process group; this fixture's async children are scoped.
}
