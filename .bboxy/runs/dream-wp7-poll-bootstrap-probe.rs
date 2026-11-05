// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Diagnostic only: same first-poll CPU observation, fresh cold/ready processes.
use std::future::Future;
fn cpu()->u64 {
    let mut t=std::mem::MaybeUninit::<libc::timespec>::uninit();
    assert_eq!(unsafe{libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID,t.as_mut_ptr())},0);
    let t=unsafe{t.assume_init()};t.tv_sec as u64*1_000_000_000+t.tv_nsec as u64
}
fn main() {
    let ready=std::env::args().nth(1).as_deref()==Some("ready");
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let bootstrap=if ready {let before=cpu();let _=bbt_rt::handle();Some(cpu()-before)}else{None};
    let folder=std::env::temp_dir().join(format!("wp7-cpu-probe-{}",std::process::id()));std::fs::create_dir(&folder).unwrap();
    let request=bboxy450_harness::process::Request{program:"/usr/bin/true".into(),args:vec!["bounded-d1-witness---".to_owned();3000],stdin:None,cwd:folder.clone(),timeout:std::time::Duration::from_secs(5),spill_dir:folder.clone(),cap:1024,redactor:Default::default()};
    let first=rt.block_on(async move {
        let mut future=Box::pin(bboxy450_harness::process::run(request,Default::default()));let mut polls=vec![];
        let result=tokio::time::timeout(std::time::Duration::from_secs(10),std::future::poll_fn(|cx|{let begin=cpu();let poll=future.as_mut().poll(cx);polls.push(cpu()-begin);poll})).await.unwrap().unwrap();
        assert_eq!(result.code,Some(0));polls[0]
    });
    std::fs::remove_dir(folder).unwrap();
    println!("{{\"mode\":\"{}\",\"bootstrap_cpu_ns\":{},\"first_poll_cpu_ns\":{},\"owned_child_reaped\":true,\"within_unchanged_1ms\":{}}}",if ready{"ready"}else{"cold"},bootstrap.map(|n|n.to_string()).unwrap_or("null".into()),first,first<=1_000_000);
    std::process::exit(if first<=1_000_000 {0}else{1});
}
