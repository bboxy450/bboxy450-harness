// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Diagnostic: a queued lazy opening must keep being polled until admission.
use std::{future::Future,sync::{Arc,Mutex,Condvar,atomic::{AtomicBool,Ordering}}};
struct Release(Arc<(Mutex<bool>,Condvar)>);
impl Drop for Release {fn drop(&mut self){*self.0.0.lock().unwrap()=true;self.0.1.notify_all();}}
fn main(){tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    let gate=Arc::new((Mutex::new(false),Condvar::new()));let release=Release(gate.clone());let mut jobs=vec![];
    for _ in 0..4 {
        let (send,receive)=tokio::sync::oneshot::channel();let gate=gate.clone();
        jobs.push(bbt_rt::handle().spawn(async move {bboxy450_harness::runtime::blocking(move ||{send.send(()).unwrap();let mut open=gate.0.lock().unwrap();while !*open {open=gate.1.wait(open).unwrap();}}).await}));
        tokio::time::timeout(std::time::Duration::from_secs(3),receive).await.unwrap().unwrap();
    }
    let started=Arc::new(AtomicBool::new(false));let entered=started.clone();let (send,mut receive)=tokio::sync::oneshot::channel();
    let mut opening=Box::pin(bboxy450_harness::runtime::blocking(move ||{entered.store(true,Ordering::SeqCst);send.send(()).unwrap();42}));
    std::future::poll_fn(|cx|{assert!(opening.as_mut().poll(cx).is_pending());std::task::Poll::Ready(())}).await;
    drop(release);for job in jobs {job.await.unwrap();}
    assert!(!started.load(Ordering::SeqCst));assert!(matches!(receive.try_recv(),Err(tokio::sync::oneshot::error::TryRecvError::Empty)));
    let result=tokio::time::timeout(std::time::Duration::from_secs(3),opening).await.unwrap();assert_eq!(result,42);receive.await.unwrap();assert!(started.load(Ordering::SeqCst));
    println!("{{\"four_busy_permits_confirmed\":true,\"first_poll_queued\":true,\"all_four_jobs_joined\":true,\"unpolled_opening_did_not_start_after_permits_returned\":true,\"driven_opening_entered_and_completed\":true,\"claim\":\"diagnostic only; full ACP failure remains red\"}}");
});}
