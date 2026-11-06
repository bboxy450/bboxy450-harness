// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! These witnesses use real CPython, Git/rg and three independent MCP transports.
use super::*;
use crate::mcp::{ServerConfig, select_sources};

struct Rig {
    _dir: tempfile::TempDir,
    service: Service,
    manager: Manager,
    services: TaskGroup,
}
impl Rig {
    async fn start() -> Self {
        Self::start_managed(false).await
    }
    async fn start_managed(managed: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let cwd = dir.path().join("source");
        tokio::fs::create_dir(&cwd).await.unwrap();
        let python = process::resolve("python3")
            .await
            .expect("fixture requires local Python 3");
        let config = |tag: &str| ServerConfig {
            command: python.to_string_lossy().into(),
            args: vec![
                "-u".into(),
                "-c".into(),
                include_str!("fixture.py").into(),
                tag.into(),
                dir.path().to_string_lossy().into(),
            ],
            env: BTreeMap::new(),
            timeout_s: 2,
            trust_hints: false,
        };
        let servers = [("a".into(), config("a")), ("z".into(), config("z"))].into();
        let launcher = if managed {
            let source = dir.path().join("implementation");
            tokio::fs::create_dir(&source).await.unwrap();
            let kb = dir.path().join("knowledge");
            tokio::fs::create_dir(&kb).await.unwrap();
            let context = json!({"guard_version":1,"seat_id":Ulid::new(),"process_generation":1,"grant_epoch":1,"provider_revision":1,
                "knowledge_binding":{"root_id":"kb","revision":1,"path":kb},
                "workspace":{"workspace_id":Ulid::new(),"revision":1,"state":"in_use"},"cwd":source,"target":{"kind":"local"}});
            tokio::fs::write(
                dir.path().join("context.json"),
                serde_json::to_vec(&context).unwrap(),
            )
            .await
            .unwrap();
            [("native".into(), config("native"))].into()
        } else {
            BTreeMap::new()
        };
        let (mut servers, _) = select_sources(servers, BTreeMap::new(), launcher);
        let endpoint = Endpoint::new().await.unwrap();
        Manager::with_python(&mut servers, &endpoint.socket());
        let mut services = TaskGroup::new(CancellationToken::new());
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let stop = services.cancel.clone();
        services.spawn(async move{loop{tokio::select!{biased;_=stop.cancelled()=>break,r=rx.recv()=>{if r.is_none(){break;}}}}});
        let manager = Manager::start_in(
            servers,
            Set::default(),
            tx,
            &mut services,
            Some(cwd.clone()),
        );
        let service = Service::start(
            endpoint,
            dir.path().join("executions"),
            cwd,
            manager.clone(),
            Set::default(),
            &mut services,
        )
        .await
        .unwrap();
        let rig = Self {
            _dir: dir,
            service,
            manager,
            services,
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if rig
                    .manager
                    .servers()
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|s| s["ready"] == true)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        rig
    }
    async fn call(&self, name: &str, args: Value) -> Value {
        let reply = self
            .manager
            .raw_call(
                &self.manager.capture(),
                "bboxy_python",
                name,
                args,
                None,
                &self.services.cancel,
            )
            .await
            .unwrap();
        reply["structuredContent"].clone()
    }
    async fn run(&self, source: &str) -> Value {
        let start = self.call("python_execute", json!({"source":source})).await;
        self.finish(start["execution_id"].as_str().unwrap()).await
    }
    async fn finish(&self, id: &str) -> Value {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let value = self
                    .call("python_execution_get", json!({"execution_id":id}))
                    .await;
                if !matches!(
                    value["status"].as_str(),
                    Some("starting" | "running" | "cancelling")
                ) {
                    return value;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }
    async fn close(mut self) {
        self.services.cancel_and_join().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn completed_execution_rejects_late_real_callback_frames() {
    let rig = Rig::start().await;
    let done = rig.run("emit(1)").await;
    let id: Ulid = done["execution_id"].as_str().unwrap().parse().unwrap();
    let run = rig.service.0.runs.lock().await.get(&id).cloned().unwrap();
    let before = serde_json::to_value(rig.service.get_record(id).await.unwrap()).unwrap();
    for (method, arguments) in [
        ("call", json!({"server":"a","tool":"write","arguments":{}})),
        ("emit", json!({"value":2})),
    ] {
        let mut stream = tokio::net::UnixStream::connect(rig.service.0.endpoint.socket())
            .await
            .unwrap();
        let mut bytes=serde_json::to_vec(&json!({"jsonrpc":"2.0","id":1,"method":"python/callback","params":{"execution_id":id,"token":run.token,"method":method,"arguments":arguments}})).unwrap();
        bytes.push(b'\n');
        stream.write_all(&bytes).await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(2), reader.read_line(&mut line))
            .await
            .unwrap()
            .unwrap();
        let reply: Value = serde_json::from_str(&line).unwrap();
        assert!(
            matches!(
                reply["error"]["data"]["error"]["code"].as_str(),
                Some("cancelled" | "execution_retired")
            ),
            "{reply}"
        );
    }
    assert!(!rig._dir.path().join("a.writes").exists());
    assert_eq!(
        serde_json::to_value(rig.service.get_record(id).await.unwrap()).unwrap(),
        before
    );
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_python_commands_and_json_work_in_source_checkout() {
    let rig = Rig::start().await;
    let value=rig.run(r#"
import json, pathlib, subprocess
pathlib.Path('example.txt').write_text('needle\n')
found = await commands.rg('--json', 'needle', 'example.txt')
none = await commands.rg('absent', 'example.txt')
quoted = await commands.run(['python3', '-c', 'import sys,json;print(json.dumps(sys.argv[1:]))', 'a b', '$(bad)', '"quoted"', 'snowman ☃', '`literal`'])
pathlib.Path('nested').mkdir()
nested=await commands.run(['python3','-c','import os;print(os.getcwd())'],cwd='nested')
pipeline=await commands.shell("printf 'one\\ntwo\\n' | wc -l")
assert pathlib.Path(nested.stdout.strip())==pathlib.Path.cwd()/'nested'
assert int(pipeline.stdout.strip())==2
git = await commands.git('init', '-q')
assert git.exit_code == 0, (git.stdout, git.stderr)
subprocess.run(['git', 'config', 'user.name', 'fixture'], check=True)
emit({'cwd':str(pathlib.Path.cwd()),'rg':found.exit_code,'none':none.exit_code,
      'argv':quoted.json(),'git':git.exit_code,'matches':[json.loads(row)['type'] for row in found.stdout.splitlines()]})
if (await commands.available('jq')).path:
    result = await commands.jq('.items | map(.n) | add', json_input={'items':[{'n':2},{'n':3}]})
    null = await commands.jq('.', json_input=None)
    emit({'jq':result.json(),'null':null.json()})
try:
    await commands.jq('.', input={'not':'JSON stdin'})
except TypeError:
    emit({'raw_object_rejected':True})
"#).await;
    assert_eq!(value["status"], "completed", "{value}");
    let result = &value["emitted"][0];
    assert_eq!(
        PathBuf::from(result["cwd"].as_str().unwrap()),
        rig.service.0.cwd.canonicalize().unwrap()
    );
    assert_eq!(result["none"], 1);
    assert_eq!(
        result["argv"],
        json!(["a b", "$(bad)", "\"quoted\"", "snowman ☃", "`literal`"])
    );
    assert_eq!(result["rg"], 0);
    assert_eq!(result["git"], 0);
    assert!(
        result["matches"]
            .as_array()
            .unwrap()
            .contains(&json!("match"))
    );
    if value["emitted"].as_array().unwrap().len() == 3 {
        assert_eq!(value["emitted"][1], json!({"jq":5,"null":null}));
    }
    assert_eq!(
        value["emitted"].as_array().unwrap().last().unwrap()["raw_object_rejected"],
        true
    );
    assert!(rig.service.0.cwd.join(".git").is_dir());
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_mcp_servers_preserve_domain_op_ids_schemas_errors_and_media() {
    let rig = Rig::start().await;
    let value = rig
        .run(
            r#"
all_tools=[]
cursor=None
while True:
    page=await mcp.tools('a', cursor)
    all_tools.extend(page.tools)
    cursor=page.next_cursor
    if cursor is None: break
a=await mcp.call('a','echo',{'op_id':42,'payload':{'n':[3,2,1]}})
z=await mcp.call('z','echo',{'op_id':7})
emit({'count':len(all_tools),'a':a.raw,'z':z.structured,'request':a.request_op_id})
try:
    await mcp.call('z','fail',{'op_id':99})
except ToolError as error:
    emit({'error_code':error.code,'hints':error.hints,'result':error.result})
try:
    await mcp.call('a','echo',{'op_id':77,'payload':{'rpc_error':True}})
except ToolError as error:
    emit({'rpc_code':error.result['error']['rpc_code'],'hints':error.hints,'server':error.server,'tool':error.tool})
for form in ['result_unknown','rpc_unknown']:
    try:
        await mcp.call('z','echo',{'op_id':81,'payload':{form:True}})
    except ToolError as error:
        emit({'uncertainty':form,'result':error.result,'hints':error.hints})
"#,
        )
        .await;
    assert_eq!(value["status"], "completed_with_errors", "{value}");
    assert_eq!(value["emitted"][0]["count"], 185);
    assert_eq!(
        value["emitted"][0]["a"]["structuredContent"]["arguments"]["op_id"],
        42
    );
    assert_eq!(value["emitted"][0]["z"]["arguments"]["op_id"], 7);
    assert_eq!(
        value["emitted"][0]["a"]["content"][1]["type"],
        "resource_link"
    );
    assert_eq!(value["emitted"][0]["a"]["_meta"]["preserved"], true);
    assert_eq!(value["emitted"][1]["error_code"], "fixture_rejected");
    assert_eq!(value["children"][2]["status"], "failed");
    assert_eq!(value["emitted"][2]["rpc_code"], -32602);
    assert_eq!(value["emitted"][2]["hints"][0]["method"], "tools/list");
    assert_eq!(value["emitted"][2]["server"], "a");
    assert_eq!(value["emitted"][2]["tool"], "echo");
    assert_eq!(value["children"][4]["status"], "unknown");
    assert_eq!(value["children"][5]["status"], "unknown");
    assert_eq!(
        value["emitted"][3]["result"]["structuredContent"]["status"],
        "unknown"
    );
    assert_eq!(
        value["emitted"][4]["result"]["error"]["data"]["status"],
        "unknown"
    );
    for index in [3, 4] {
        assert_eq!(
            value["emitted"][index]["hints"][0]["params"]["name"],
            "operation_status"
        );
        assert_eq!(
            value["emitted"][index]["hints"][0]["params"]["arguments"]["op_id"],
            81
        );
    }
    // Full discovery reaches >128 tools; runtime remains model-visible first.
    let (defs, _, _) = rig.manager.snapshot();
    assert!(
        defs.iter()
            .any(|d| d.wire.name.ends_with("__python_execute"))
    );
    assert!(
        defs.iter()
            .any(|d| d.wire.name.ends_with("__python_execute_orient"))
    );
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lost_outer_output_duplicate_and_restart_never_replay_successful_write() {
    let rig = Rig::start().await;
    let id = Ulid::new();
    let args = json!({"op_id":id,"source":"await mcp.call('a','write', {'op_id':41})\nemit({'done':True})"});
    // Real MCP request; deliberately discard its successful outer response.
    let _ = rig.call("python_execute", args.clone()).await;
    let value = rig.finish(&id.to_string()).await;
    assert_eq!(value["status"], "completed");
    let retry = rig.call("python_execute", args.clone()).await;
    assert_eq!(retry["execution_id"], json!(id));
    let writes = tokio::fs::read_to_string(rig._dir.path().join("a.writes"))
        .await
        .unwrap();
    assert_eq!(writes, "write\n");
    let conflict = rig
        .call(
            "python_execute",
            json!({"op_id":id,"source":"print('different')"}),
        )
        .await;
    assert_eq!(conflict["error"]["code"], "op_id_reused");
    // A reconstructed service sees a persisted completed result, not runnable code.
    rig.service.0.runs.lock().await.remove(&id);
    rig.service.recover().await.unwrap();
    let after = rig.call("python_execute", args).await;
    assert_eq!(after["status"], "completed");
    assert_eq!(
        tokio::fs::read_to_string(rig._dir.path().join("a.writes"))
            .await
            .unwrap(),
        "write\n"
    );
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancellation_timeout_and_grandchild_pipes_leave_other_execution_live() {
    let rig = Rig::start().await;
    let started=rig.call("python_execute",json!({"source":"import subprocess,time\nsubprocess.Popen(['sleep','60'])\nprint('ready',flush=True)\ntime.sleep(60)"})).await;
    let id = started["execution_id"].as_str().unwrap();
    let other = rig.run("emit(6*7)").await;
    assert_eq!(other["emitted"][0], 42);
    let _ = rig
        .call("python_execution_cancel", json!({"execution_id":id}))
        .await;
    let ended = rig.finish(id).await;
    assert_eq!(ended["status"], "cancelled");
    if let Some(pid) = ended["pid"].as_u64() {
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    }
    let short = rig
        .call(
            "python_execute",
            json!({"source":"while True: pass","timeout_ms":100}),
        )
        .await;
    let ended = rig.finish(short["execution_id"].as_str().unwrap()).await;
    assert_eq!(ended["status"], "timed_out");
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stalled_mcp_batch_cannot_starve_other_run_or_fresh_proxy_cancel() {
    let rig = Rig::start().await;
    let started=rig.call("python_execute",json!({"source":"import asyncio\nfrom concurrent.futures import ThreadPoolExecutor\nasyncio.get_running_loop().set_default_executor(ThreadPoolExecutor(max_workers=40))\nawait asyncio.gather(*(mcp.call('a','echo',{'payload':{'wait':True}}) for _ in range(32)))"})).await;
    let id = started["execution_id"].as_str().unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !rig._dir.path().join("a.waiting").is_file() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let other = rig.run("emit('independent')").await;
    assert_eq!(other["status"], "completed", "{other}");
    let mut stream = tokio::net::UnixStream::connect(rig.service.0.endpoint.socket())
        .await
        .unwrap();
    let mut request=serde_json::to_vec(&json!({"jsonrpc":"2.0","id":1,"method":"python/proxy","params":{"name":"python_execution_cancel","arguments":{"execution_id":id},"context":null}})).unwrap();
    request.push(b'\n');
    stream.write_all(&request).await.unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(1), reader.read_line(&mut line))
        .await
        .unwrap()
        .unwrap();
    let cancelled: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        cancelled["result"]["structuredContent"]["cancel_requested"],
        true
    );
    drop(reader);
    assert_eq!(rig.finish(id).await["status"], "cancelled");
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn output_pages_and_partial_admission_do_not_corrupt_protocol_or_block_startup() {
    let rig = Rig::start().await;
    let value = rig
        .run("import os\nos.write(1,b'x'*200000)\nos.write(2,b'\\xff'*1000)\nemit({'ok':True})")
        .await;
    assert_eq!(value["status"], "completed", "{value}");
    let id = value["execution_id"].as_str().unwrap();
    let mut cursor = Value::Null;
    let mut bytes = 0;
    loop {
        let mut args = json!({"execution_id":id});
        if !cursor.is_null() {
            args["cursor"] = cursor;
        }
        let page = rig.call("python_execution_get", args).await;
        for part in page["output"].as_array().unwrap() {
            bytes += STANDARD
                .decode(part["base64"].as_str().unwrap())
                .unwrap()
                .len();
        }
        cursor = page["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    assert_eq!(bytes, 201000);
    let damaged = Ulid::new();
    tokio::fs::write(
        rig.service.0.root.join(format!("{damaged}.json")),
        b"{partial",
    )
    .await
    .unwrap();
    tokio::fs::write(
        rig.service.0.root.join(".admit-interrupted.tmp"),
        b"{partial",
    )
    .await
    .unwrap();
    rig.service.recover().await.unwrap();
    assert_eq!(
        rig.service.get_record(damaged).await.unwrap().status,
        "unknown"
    );
    let healthy = rig.run("emit('still usable')").await;
    assert_eq!(healthy["emitted"][0], "still usable");
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn large_nested_result_is_paged_as_complete_json_and_stdout_budget_is_explicit() {
    let rig = Rig::start().await;
    let value=rig.run("import os\nresponse=await mcp.call('a','echo',{'op_id':501,'payload':'x'*120000})\nassert len(response.structured.arguments.payload)==120000\nos.write(1,b'z'*1200000)").await;
    assert_eq!(value["status"], "completed", "{value}");
    assert!(value["omitted_bytes"].as_u64().unwrap() > 0);
    let id = value["execution_id"].as_str().unwrap();
    let mut cursor = json!("child:0:0");
    let mut bytes = vec![];
    loop {
        let page = rig
            .call(
                "python_execution_get",
                json!({"execution_id":id,"cursor":cursor}),
            )
            .await;
        bytes.extend(
            STANDARD
                .decode(page["data_base64"].as_str().unwrap())
                .unwrap(),
        );
        cursor = page["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    let original: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(original["structuredContent"]["arguments"]["op_id"], 501);
    assert_eq!(
        original["structuredContent"]["arguments"]["payload"]
            .as_str()
            .unwrap()
            .len(),
        120000
    );
    assert_eq!(original["content"][1]["type"], "resource_link");
    assert!(
        !original["structuredContent"]["hints"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn changed_mcp_catalogue_generation_is_refused_before_new_dispatch() {
    let rig = Rig::start().await;
    let value=rig.run("import asyncio\nawait mcp.call('a','change',{})\nawait asyncio.sleep(1.2)\nawait mcp.call('a','write',{})").await;
    assert_eq!(value["status"], "failed", "{value}");
    assert!(!rig._dir.path().join("a.writes").exists());
    assert_eq!(value["children"][1]["status"], "failed");
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disconnected_server_keeps_unknown_receipt_and_cannot_replay_on_replacement() {
    let rig = Rig::start().await;
    let value=rig.run("import asyncio\ntry:\n    await mcp.call('a','crash',{})\nexcept ToolError as error:\n    emit({'unknown':error.result['error']['unknown'],'hints':error.hints,'execution_id':execution.execution_id})\nawait asyncio.sleep(1)\ntry:\n    await mcp.call('a','write',{})\nexcept ToolError as error:\n    emit({'replacement':error.result['error']['code']})").await;
    assert_eq!(value["status"], "completed_with_errors", "{value}");
    assert_eq!(value["children"][0]["status"], "unknown");
    assert_eq!(value["emitted"][0]["unknown"], true);
    assert_eq!(value["emitted"][0]["execution_id"], value["execution_id"]);
    assert!(!value["emitted"][0]["hints"].as_array().unwrap().is_empty());
    assert!(matches!(
        value["emitted"][1]["replacement"].as_str(),
        Some("server_generation_changed" | "server_unavailable")
    ));
    assert!(!rig._dir.path().join("a.writes").exists());
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn managed_source_and_knowledge_stay_separate_and_revocation_fences_effects() {
    let rig = Rig::start_managed(true).await;
    let value=rig.run("import pathlib\npathlib.Path('source-only.txt').write_text('source')\nemit({'cwd':str(pathlib.Path.cwd()),'kb':execution.knowledge_binding.path,'workspace':workspace})").await;
    assert_eq!(value["status"], "completed", "{value}");
    assert!(
        rig._dir
            .path()
            .join("implementation/source-only.txt")
            .is_file()
    );
    assert!(!rig._dir.path().join("knowledge/source-only.txt").exists());
    assert_eq!(value["emitted"][0]["workspace"]["state"], "in_use");
    let started=rig.call("python_execute",json!({"source":"import asyncio\nemit('admitted')\nawait asyncio.sleep(0.5)\nawait mcp.call('a','write',{})"})).await;
    let id: Ulid = started["execution_id"].as_str().unwrap().parse().unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if !rig.service.get_record(id).await.unwrap().emitted.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let path = rig._dir.path().join("context.json");
    let mut context: Value =
        serde_json::from_slice(&tokio::fs::read(&path).await.unwrap()).unwrap();
    context["grant_epoch"] = json!(2);
    context["workspace"]["revision"] = json!(2);
    tokio::fs::write(&path, serde_json::to_vec(&context).unwrap())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if rig.service.get_record(id).await.unwrap().status == "failed" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(!rig._dir.path().join("a.writes").exists());
    let hidden = rig
        .call("python_execution_get", json!({"execution_id":id}))
        .await;
    assert_eq!(hidden["error"]["code"], "execution_context_changed");
    let remote = context;
    let mut remote = remote;
    remote["target"] = json!({"kind":"ssh","host":"fixture.invalid"});
    tokio::fs::write(&path, serde_json::to_vec(&remote).unwrap())
        .await
        .unwrap();
    let admitted = rig
        .call(
            "python_execute",
            json!({"source":"emit('must not execute remotely')"}),
        )
        .await;
    let execution: Ulid = admitted["execution_id"].as_str().unwrap().parse().unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let r = rig.service.get_record(execution).await.unwrap();
            if r.status == "failed" {
                assert_eq!(r.error.unwrap()["error"]["code"], "unsupported_target");
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn old_native_bridge_cannot_drop_guard_metadata_and_fall_back_to_standalone() {
    let rig = Rig::start_managed(true).await;
    let path = rig._dir.path().join("context.json");
    let mut context: Value =
        serde_json::from_slice(&tokio::fs::read(&path).await.unwrap()).unwrap();
    context["_transport_guard_version"] = json!(0);
    tokio::fs::write(&path, serde_json::to_vec(&context).unwrap())
        .await
        .unwrap();
    let refused = rig
        .call("python_execute", json!({"source":"emit('must not start')"}))
        .await;
    assert_eq!(
        refused["error"]["code"], "native_context_unavailable",
        "{refused}"
    );
    assert!(rig.service.0.runs.lock().await.is_empty());
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_arguments_do_not_create_execution_and_concurrent_duplicates_share_one_admission() {
    let rig = Rig::start().await;
    for args in [
        json!({"source":"emit(1)","timeout_ms":"bad"}),
        json!({"source":"emit(1)","extra":true}),
        json!({"source":"emit(1)","op_id":Ulid::nil()}),
    ] {
        let error = rig.call("python_execute", args).await;
        assert_eq!(error["error"]["code"], "invalid_arguments");
    }
    assert!(rig.service.0.runs.lock().await.is_empty());
    let id = Ulid::new();
    let args = json!({"op_id":id,"source":"await mcp.call('a','write',{})"});
    let (a, b) = tokio::join!(
        rig.call("python_execute", args.clone()),
        rig.call("python_execute", args)
    );
    assert_eq!(a["execution_id"], json!(id));
    assert_eq!(b["execution_id"], json!(id));
    assert_eq!(rig.finish(&id.to_string()).await["status"], "completed");
    assert_eq!(
        tokio::fs::read_to_string(rig._dir.path().join("a.writes"))
            .await
            .unwrap(),
        "write\n"
    );
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn acp_shared_relay_runs_python_and_reconnects_without_retiring_other_clients() {
    let mut rig = Rig::start().await;
    let relay = crate::mcp::relay::Relay::shared(rig.manager.clone(), &mut rig.services)
        .await
        .unwrap();
    let config = relay
        .servers
        .iter()
        .find(|s| s["name"] == "bboxy_python")
        .unwrap();
    let socket = PathBuf::from(config["args"][1].as_str().unwrap());
    let request = |method: &str, params: Value| json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
    async fn rpc(socket: &std::path::Path, request: Value) -> Value {
        let mut stream = tokio::net::UnixStream::connect(socket).await.unwrap();
        let mut bytes = serde_json::to_vec(&request).unwrap();
        bytes.push(b'\n');
        stream.write_all(&bytes).await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut reply = String::new();
        reader.read_line(&mut reply).await.unwrap();
        serde_json::from_str::<Value>(&reply).unwrap()["result"].clone()
    }
    let tools = rpc(&socket, request("tools/list", json!({}))).await;
    assert!(
        tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "python_execute")
    );
    // A second real ACP-facing transport works after the first disconnect.
    let started=rpc(&socket,request("tools/call",json!({"name":"python_execute","arguments":{"source":"result=await mcp.call('z','echo',{'op_id':55})\nemit(result.structured.arguments)"}}))).await;
    let id = started["structuredContent"]["execution_id"]
        .as_str()
        .unwrap();
    let value = rig.finish(id).await;
    assert_eq!(value["status"], "completed", "{value}");
    assert_eq!(value["emitted"][0]["op_id"], 55);
    rig.close().await;
}
