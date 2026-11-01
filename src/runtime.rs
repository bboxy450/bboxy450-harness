// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use std::{
    cell::Cell,
    fs::File,
    io,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, ThreadId},
};
use tokio::sync::Semaphore;

thread_local! { static LEASE: Cell<Option<ThreadId>> = const { Cell::new(None) }; }
static HELD: AtomicUsize = AtomicUsize::new(0);
static MAX_HELD: AtomicUsize = AtomicUsize::new(0);
static HOOKS: [AtomicUsize; 7] = [const { AtomicUsize::new(0) }; 7];
type Observer = Arc<dyn Fn(BlockingWork) + Send + Sync>;
static OBSERVER: Mutex<Option<Observer>> = Mutex::new(None);

/// All expensive entry points share the same permit-backed assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BlockingWork {
    Grep,
    Glob,
    DirectoryRead,
    Diff,
    Fsync,
    MarkdownPrepare,
    Walk,
}

/// A lease is installed only inside `spawn_blocking`, while its permit is owned.
/// Checking the originating thread also catches accidental inline execution.
pub fn assert_off_rt_worker() {
    LEASE.with(|lease| {
        let caller = lease
            .get()
            .expect("blocking work ran without a harness permit");
        assert_ne!(
            caller,
            thread::current().id(),
            "blocking work ran on its IO worker"
        );
    });
}

pub fn blocking_hook(work: BlockingWork) {
    assert_off_rt_worker();
    HOOKS[work as usize].fetch_add(1, Ordering::Relaxed);
    let observer = OBSERVER.lock().unwrap().clone();
    if let Some(observer) = observer {
        observer(work);
    }
}

#[derive(Clone, Debug)]
pub struct BlockingStats {
    pub max_held: usize,
    hooks: [usize; 7],
}
impl BlockingStats {
    pub fn calls(&self, work: BlockingWork) -> usize {
        self.hooks[work as usize]
    }
}
pub fn blocking_stats() -> BlockingStats {
    BlockingStats {
        max_held: MAX_HELD.load(Ordering::Relaxed),
        hooks: std::array::from_fn(|i| HOOKS[i].load(Ordering::Relaxed)),
    }
}

/// Temporary diagnostic observer. Called outside the observer lock, after the
/// worker/permit assertion; tests can synchronize real work without a clock.
pub struct BlockingObservation;
pub fn observe_blocking(
    observer: impl Fn(BlockingWork) + Send + Sync + 'static,
) -> BlockingObservation {
    let mut slot = OBSERVER.lock().unwrap();
    assert!(slot.is_none(), "a blocking observer is already installed");
    *slot = Some(Arc::new(observer));
    BlockingObservation
}
impl Drop for BlockingObservation {
    fn drop(&mut self) {
        *OBSERVER.lock().unwrap() = None;
    }
}

struct HeldPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}
impl Drop for HeldPermit {
    fn drop(&mut self) {
        HELD.fetch_sub(1, Ordering::SeqCst);
    }
}
struct Lease;
impl Drop for Lease {
    fn drop(&mut self) {
        LEASE.with(|lease| lease.set(None));
    }
}

pub fn sync_all(file: &File) -> io::Result<()> {
    blocking_hook(BlockingWork::Fsync);
    file.sync_all()
}

/// CPU-heavy IO shares four blocking permits, never the two shared IO workers.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    static LIMIT: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let permit = LIMIT
        .get_or_init(|| Arc::new(Semaphore::new(4)))
        .clone()
        .acquire_owned()
        .await
        .expect("blocking semaphore");
    let held = HELD.fetch_add(1, Ordering::SeqCst) + 1;
    MAX_HELD.fetch_max(held, Ordering::SeqCst);
    let permit = HeldPermit { _permit: permit };
    let caller = thread::current().id();
    bbt_rt::handle()
        .spawn_blocking(move || {
            let _permit = permit;
            LEASE.with(|lease| {
                assert!(lease.get().is_none());
                lease.set(Some(caller));
            });
            let _lease = Lease;
            f()
        })
        .await
        .expect("blocking task panicked")
}
