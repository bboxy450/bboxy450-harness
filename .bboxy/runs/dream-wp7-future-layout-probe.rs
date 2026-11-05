// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Pure construction/layout probe: no future is polled and no IO is admitted.
use bboxy450_harness::{engine::{Config,SessionActor},gateway::Host,protocol::*,provider::{ScriptedProvider,profile::Unattended},policy::Policy};
fn main(){
    let meta=SessionMeta{session:SessionId::new(),cwd:"/tmp".into(),profile:None,fork:None,protocol:PROTOCOL,harness:"layout-probe".into(),created_ms:0,vcs:None,dream:None,source:None};
    let cfg=Config{meta:meta.clone(),sessions:"/tmp/never-created-wp7-layout".into(),profiles:"/tmp/never-created-wp7-profiles".into(),provider:std::sync::Arc::new(ScriptedProvider::new(vec![])),model:"scripted".into(),policy:Policy::new(Unattended::Deny),redactor:Default::default(),max_rounds:2,profile_error:None};
    let host=Host::new(cfg.clone());let op=OpEnvelope{v:PROTOCOL,session:meta.session,op_id:OpId::new(),op:Op::StartSession{cwd:meta.cwd.clone(),profile:None,fork:None}};
    let req=bboxy450_harness::process::Request{program:"/usr/bin/true".into(),args:vec![],stdin:None,cwd:"/tmp".into(),timeout:std::time::Duration::from_secs(1),spill_dir:"/tmp/never-created-wp7-layout".into(),cap:1,redactor:Default::default()};
    println!("{{\"host_open\":{},\"host_create\":{},\"engine_load\":{},\"store_open\":{},\"process_run\":{}}}",std::mem::size_of_val(&host.open(meta.session)),std::mem::size_of_val(&host.create(op)),std::mem::size_of_val(&SessionActor::open(cfg,OpId::new())),std::mem::size_of_val(&bboxy450_harness::store::Store::open("/tmp/never-created-wp7-layout".into(),meta,None,Default::default())),std::mem::size_of_val(&bboxy450_harness::process::run(req,Default::default())));
}
