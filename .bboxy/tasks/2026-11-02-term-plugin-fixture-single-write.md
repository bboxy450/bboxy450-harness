---
allowed_paths:
- crates/bbt-plugin-fixture/src/reference.rs
- crates/bbt-plugin-fixture/
- .bboxy/
authorship:
  kind: seat
  process_generation: '2'
  public_key: V9Fswn0eQw6hTzOemJw9n9Ceh/T/tel4fwr1bzL3B+o=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-02T04:12:47.978Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6D7BDBHX5H0E44JRQ2HBT7A
number: 5171
provenance:
  digest: sha256:b30efc8b2b0e9a833bf7973647eacf95d13f2d6408c585bfb712311b65b0d026
  frontmatter_digest: sha256:52ae880f2f2d5a5302685e43c22a1f4854dc3aad0743384d29b12a025a010e47
  log: .bboxy/log/2026-11-02.jsonl#34378
  sig: 3gsEPbrcso0J++uz5s5QTjVqHmy4M1ZT74V6khgcdKHw8tYDjPHsMRUOrvUyj98ECEDy1cJDKyEtBhLAAQx7Dg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1c: plugin fixture writes each hook-log record in one write (scope add for #5165)'
type: task
updated: 2026-11-02T04:12:47.978Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: cargo test -p bboxy450-term-mux --test plugin_host
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. This is a scope addition to #5165. #5165 was claimed before the writer's path was known, and the record can no longer be edited.

Branch: `resource-ledger-race`, the same branch as #5117, #5165 and #5169. Do not merge or run land.sh.

**Defect.** `crates/bbt-plugin-fixture/src/reference.rs::append` (line 43) uses `writeln!(file, "{value}")`, which can split one record across several `write` calls. A concurrent reader can then see a partial line. E1c found this.

**Fix.** Serialize the record and its trailing newline into one buffer, then call `write_all` once. With O_APPEND, a single write is the unit the reader relies on. Keep the newline-safe reader from #5165 as well: belt and braces.

**Tests.**
- Targeted only. `cargo test -p bboxy450-term-mux --test plugin_host` passes 20 times in a row.
- The split-record witness from #5165 stays green.
- The branch's one final full `make test` covers this task.

**Acceptance.**
- Signed commit.
- Covered by the branch's shared typed report.
- One task_report citing the shared gate log.