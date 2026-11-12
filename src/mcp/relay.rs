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
    // A relay is a live MCP server too. Forward catalogue changes from the
    // existing upstream slot, without polling or reopening its process.
    if let Some(slot) = manager.slots.iter().find(|slot| slot.name == name).cloned() {
        let writer = writer.clone();
        let cancel = cancel.clone();
        tasks.spawn(async move {
            let mut previous = None;
            loop {
                let notified = slot.initialized.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                let current = slot.ready.load_full();
                let version = current.as_ref().as_ref().map(|ready| (ready.generation, ready.schema_digest.clone()));
                if previous.as_ref().is_some_and(|previous| previous != &version) {
                    let notification = b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/tools/list_changed\"}\n";
                    // Do not queue behind a blocked response for an optional
                    // notification. A pre-dispatch tool miss also refreshes.
                    if let Ok(mut writer) = writer.try_lock() {
                        tokio::select! {biased;_=cancel.cancelled()=>return,_=writer.write_all(notification)=>{}}
                    }
                }
                previous = Some(version);
                tokio::select! {biased;_=cancel.cancelled()=>return,_=notified=>{}}
            }
        });
    }
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
                Some("initialize")=>Ok(json!({"protocolVersion":request["params"]["protocolVersion"].as_str().unwrap_or("2025-11-25"),"capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":name,"version":env!("CARGO_PKG_VERSION")}})),
                Some("ping")=>Ok(json!({})),
                Some("tools/list"|"tools/call")=>{
                    let deadline=tokio::time::Instant::now()+Duration::from_secs(10);
                    loop {
                        let ready=manager.servers().as_array().is_some_and(|ss|ss.iter().any(|s|s["name"]==name&&(s["ready"]==true||matches!(s["state"].as_str(),Some("failed"|"dead")))));
                        if ready||tokio::time::Instant::now()>=deadline{break;}
                        tokio::select!{biased;_=cancel.cancelled()=>return,_=tokio::time::sleep(Duration::from_millis(10))=>{}}
                    }
                    // A new catalogue starts a new generation binding. Calls
                    // and subsequent pages retain exactly that advertised view.
                    let captured={let mut bound=capture.lock().await;
                        if request["method"]=="tools/list" && request["params"]["cursor"].as_str().is_none_or(|cursor|cursor=="0") {
                            *bound=Some(manager.capture());
                        }
                        bound.get_or_insert_with(||manager.capture()).clone()};
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

#[cfg(test)]
mod tests {
    use super::*;

    // An actual stdio MCP server exits after a restart request. Its next
    // supervised process advertises the same schema with a new generation.
    const SERVER: &str = r#"
import json, os, pathlib, sys
for line in sys.stdin:
    q = json.loads(line)
    if 'id' not in q:
        continue
    method = q['method']
    if method == 'initialize':
        result = {'protocolVersion':'2024-11-05','capabilities':{'tools':{}},'serverInfo':{'name':'restart-fixture','version':'1'}}
    elif method == 'tools/list':
        result = {'tools':[{'name':n,'inputSchema':{'type':'object'}} for n in ['restart','echo']]}
    elif method == 'tools/call':
        name = q['params']['name']
        if name == 'echo':
            with pathlib.Path('calls').open('a') as f:
                f.write('echo\n')
        result = {'content':[{'type':'text','text':name}]}
    else:
        result = {}
    print(json.dumps({'jsonrpc':'2.0','id':q['id'],'result':result}), flush=True)
    if method == 'tools/call' and q['params']['name'] == 'restart':
        os._exit(0)
"#;

    async fn request(write: &mut tokio::net::unix::OwnedWriteHalf, id: u64, method: &str) {
        let params = match method {
            "initialize" => {
                json!({"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"relay-test","version":"1"}})
            }
            "restart" | "echo" => json!({"name":method,"arguments":{}}),
            _ => json!({}),
        };
        let method = if matches!(method, "restart" | "echo") {
            "tools/call"
        } else {
            method
        };
        write
            .write_all(
                format!(
                    "{}\n",
                    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    }

    async fn read(reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>) -> Value {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(8), reader.read_line(&mut line))
            .await
            .expect("MCP reply or notification within eight seconds")
            .unwrap();
        assert!(n > 0, "relay ended");
        serde_json::from_str(&line).unwrap()
    }

    async fn reply(reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>, id: u64) -> Value {
        loop {
            let value = read(reader).await;
            if value["id"] == id {
                return value;
            }
        }
    }

    #[tokio::test]
    async fn restart_notifies_restoration_and_relist_refreshes_only_future_calls() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = dir.path().join("server.py");
        std::fs::write(&fixture, SERVER).unwrap();
        let config = ServerConfig {
            command: "python3".into(),
            args: vec!["-u".into(), fixture.to_string_lossy().into()],
            env: BTreeMap::new(),
            timeout_s: 2,
            trust_hints: false,
        };
        let (notices, _output) = mpsc::channel(256);
        let mut services = TaskGroup::new(CancellationToken::new());
        let manager = Manager::start_in(
            [(
                "probe".into(),
                SpawnConfig {
                    config,
                    origin: Origin::Profile,
                },
            )]
            .into(),
            Set::default(),
            notices,
            &mut services,
            Some(dir.path().into()),
        );
        tokio::time::timeout(Duration::from_secs(8), async {
            while manager.slots[0].ready.load().is_none() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("initial fixture startup");
        let relay = Relay::shared(manager.clone(), &mut services).await.unwrap();
        let stream = UnixStream::connect(relay.directory.join("0"))
            .await
            .unwrap();
        let (read_half, mut write) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        request(&mut write, 1, "initialize").await;
        assert!(reply(&mut reader, 1).await["error"].is_null());
        request(&mut write, 2, "tools/list").await;
        assert_eq!(
            reply(&mut reader, 2).await["result"]["tools"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        request(&mut write, 3, "restart").await;
        let mut changes = 0;
        while changes < 2 {
            if read(&mut reader).await["method"] == "notifications/tools/list_changed" {
                changes += 1;
            }
        }
        request(&mut write, 4, "echo").await;
        let stale = reply(&mut reader, 4).await;
        assert_eq!(
            stale["error"]["data"]["code"], "server_generation_changed",
            "{stale}"
        );
        assert!(
            !dir.path().join("calls").exists(),
            "a stale call reached the replacement server"
        );
        request(&mut write, 5, "tools/list").await;
        let refreshed = reply(&mut reader, 5).await;
        assert!(
            refreshed["error"].is_null(),
            "catalogue remained stale: {refreshed}"
        );
        request(&mut write, 6, "echo").await;
        let current = reply(&mut reader, 6).await;
        assert!(current["error"].is_null(), "{current}");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("calls")).unwrap(),
            "echo\n"
        );
        drop(write);
        drop(reader);
        services.cancel_and_join().await.unwrap();
    }
}
