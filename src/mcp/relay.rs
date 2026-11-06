// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! ACP receives only session-private sockets; direct calls and Python share MCP clients.
use super::*;
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

pub(crate) struct Relay {
    pub servers: Vec<Value>,
    directory: PathBuf,
}
impl Relay {
    pub(crate) async fn shared(
        manager: Manager,
        services: &mut TaskGroup,
    ) -> std::io::Result<Self> {
        let directory = PathBuf::from("/tmp").join(format!("bh-mcp-{}", ulid::Ulid::new()));
        tokio::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .await?;
        let mut relay = Self {
            servers: vec![],
            directory,
        };
        let mut slots = manager.slots.iter().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.origin != Origin::Runtime);
        for (i, slot) in slots.into_iter().enumerate() {
            if validate_environment(slot.origin, &slot.config.env).is_err() {
                continue;
            }
            let path = relay.directory.join(i.to_string());
            let listener = UnixListener::bind(&path)?;
            tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await?;
            relay.servers.push(
                json!({"name":slot.name,"command":"/usr/bin/nc","args":["-U",path],"env":[]}),
            );
            let manager = manager.clone();
            let name = slot.name.clone();
            let cancel = services.cancel.child_token();
            services.spawn(async move {
                let mut tasks=tokio::task::JoinSet::new();
                loop {
                    tokio::select!{biased;
                        _=cancel.cancelled()=>break,
                        Some(_)=tasks.join_next(),if !tasks.is_empty()=>{},
                        incoming=listener.accept()=>{
                            let Ok((stream,_))=incoming else{break};
                            if !stream.peer_cred().is_ok_and(|p|p.uid()==unsafe{libc::geteuid()}){continue;}
                            if tasks.len()>=32{continue;}
                            let manager=manager.clone();let name=name.clone();let cancel=cancel.child_token();
                            tasks.spawn(async move{serve(stream,manager,name,cancel).await;});
                        }
                    }
                }
                while tasks.join_next().await.is_some(){}
            });
        }
        Ok(relay)
    }
    pub fn remove(&self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
impl Drop for Relay {
    fn drop(&mut self) {
        self.remove();
    }
}

async fn serve(stream: UnixStream, manager: Manager, name: String, cancel: CancellationToken) {
    let (read, write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let writer = Arc::new(tokio::sync::Mutex::new(write));
    let mut tasks = tokio::task::JoinSet::new();
    let capture = Arc::new(tokio::sync::Mutex::new(None::<Capture>));
    loop {
        let mut bytes = vec![];
        let mut bounded = (&mut reader).take(8 * 1024 * 1024 + 1);
        let count = tokio::select! {biased;_=cancel.cancelled()=>break,
        n=bounded.read_until(b'\n',&mut bytes)=>n};
        let Ok(n) = count else { break };
        if n == 0 || n > 8 * 1024 * 1024 {
            break;
        }
        let Ok(request) = serde_json::from_slice::<Value>(&bytes) else {
            break;
        };
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        if tasks.len() >= 128 {
            break;
        }
        let manager = manager.clone();
        let name = name.clone();
        let cancel = cancel.clone();
        let writer = writer.clone();
        let capture = capture.clone();
        tasks.spawn(async move {
            let result=match request["method"].as_str() {
                Some("initialize")=>Ok(json!({"protocolVersion":request["params"]["protocolVersion"].as_str().unwrap_or("2025-11-25"),"capabilities":{"tools":{}},"serverInfo":{"name":name,"version":env!("CARGO_PKG_VERSION")}})),
                Some("ping")=>Ok(json!({})),
                Some("tools/list"|"tools/call")=>{
                    let deadline=tokio::time::Instant::now()+Duration::from_secs(10);
                    loop {
                        let ready=manager.servers().as_array().is_some_and(|ss|ss.iter().any(|s|s["name"]==name&&(s["ready"]==true||matches!(s["state"].as_str(),Some("failed"|"dead")))));
                        if ready||tokio::time::Instant::now()>=deadline{break;}
                        tokio::select!{biased;_=cancel.cancelled()=>return,_=tokio::time::sleep(Duration::from_millis(10))=>{}}
                    }
                    let captured={let mut bound=capture.lock().await;bound.get_or_insert_with(||manager.capture()).clone()};
                    if request["method"]=="tools/list" {
                        let offset=request["params"]["cursor"].as_str().unwrap_or("0").parse::<usize>().unwrap_or(usize::MAX);
                        manager.tools_page(&captured,&name,offset).map(|page|{
                            let mut reply=json!({"tools":page["tools"]});
                            if !page["next_cursor"].is_null(){reply["nextCursor"]=page["next_cursor"].clone();}reply
                        })
                    }else{
                        manager.raw_call(&captured,&name,request["params"]["name"].as_str().unwrap_or(""),request["params"]["arguments"].clone(),None,&cancel).await
                    }
                },
                _=>Err(json!({"code":"method_not_found","message":"unsupported MCP relay method"})),
            };
            let reply=match result {Ok(value)=>json!({"jsonrpc":"2.0","id":id,"result":value}),Err(error)=>{
                notice(&manager.notices,&cancel,Level::Warn,format!("mcp {name}: relay unavailable or request deadline exceeded")).await;
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":"MCP relay request failed","data":error}})
            }};
            if let Ok(mut bytes)=serde_json::to_vec(&reply) {bytes.push(b'\n');let _=writer.lock().await.write_all(&bytes).await;}
        });
        while tasks.try_join_next().is_some() {}
    }
    // Disconnect stops corresponding requests, never restarts a singleton server.
    cancel.cancel();
    while tasks.join_next().await.is_some() {}
}
