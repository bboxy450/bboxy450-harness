// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::{r, sufficient};
use bboxy450_harness::{protocol::*, redact::Set, store::Store};
pub fn lower_fixture() -> sufficient::AuditFixture {
    let mut f = sufficient::AuditFixture::new(false);
    let home = f.fixture.repo.home.path();
    let canonical = home.canonicalize().unwrap();
    f.domains
        .sink_roots
        .extend(f.domains.sink_roots.clone().iter().filter_map(|p| {
            p.strip_prefix(home)
                .ok()
                .map(|suffix| canonical.join(suffix))
        }));
    f
}
pub async fn lower_store(f: &sufficient::AuditFixture) -> Store {
    let session = SessionId::new();
    let log = r::start(r::sid(444), &f.fixture.repo.path, true);
    let Event::SessionStarted { mut meta } = log[0].event.clone() else {
        unreachable!()
    };
    meta.session = session;
    meta.dream.as_mut().unwrap().phase = DreamPhase::Lower;
    Store::open(f.sessions.clone(), meta, None, Set::default())
        .await
        .unwrap()
}
pub fn context_sink(f: &sufficient::AuditFixture) -> SinkRef {
    SinkRef {
        kind: SinkKind::EngineContext,
        path: f
            .fixture
            .repo
            .home
            .path()
            .canonicalize()
            .unwrap()
            .join(".bboxy450-harness/dream/context")
            .join(&bboxy450_harness::dream::fingerprint::digest("fixture.invalid/dream/repo")[..16])
            .join(format!("{}.md", ulid::Ulid::new()))
            .to_str()
            .unwrap()
            .into(),
    }
}
pub async fn finish_run(
    mut store: Store,
    f: &sufficient::AuditFixture,
    owner: &bboxy450_harness::dream::seal::Owner,
) -> (Store, bboxy450_harness::dream::receipt::Accepted) {
    use bboxy450_harness::{
        dream::{receipt, seal},
        runtime,
    };
    let item = ItemId::new();
    store
        .append(
            Event::ItemStarted {
                item,
                kind: ItemKind::Receipt,
                turn: None,
                parent: None,
                head: None,
            },
            None,
        )
        .await
        .unwrap();
    store.sync().await.unwrap();
    let mut body = r::run_body(store.session, f.producer.clone());
    body.sources = Some(f.sources(&f.logs.keys().copied().collect::<Vec<_>>()));
    let (body, bytes, carrier) = body
        .freeze(owner, item, &store.events, &Set::default())
        .unwrap();
    let Item::Receipt {
        blob: Some(blob), ..
    } = &carrier
    else {
        unreachable!()
    };
    let blob = blob.clone();
    let b = body.clone();
    let expected = blob.clone();
    store = runtime::blocking(move || {
        seal::persist(&store, &b, &bytes, &expected, &Set::default()).unwrap();
        store
    })
    .await;
    let event = store
        .append(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: carrier,
            },
            None,
        )
        .await
        .unwrap();
    store.append(Event::SessionClosed, None).await.unwrap();
    store.sync().await.unwrap();
    (
        store,
        receipt::Accepted {
            body,
            blob,
            item,
            seq: event.seq.unwrap(),
            temp_leaks: vec![],
        },
    )
}
