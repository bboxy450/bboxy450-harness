// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::{Entry, Scan, tag};
use serde_json::{Value, json};
use std::{io, path::Path};

fn snake(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        if c.is_ascii_uppercase() && !result.is_empty() && !result.ends_with('_') {
            result.push('_');
        }
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    let result = result.trim_matches('_');
    if result.is_empty() {
        "native_record".into()
    } else {
        result.into()
    }
}

/// The fragment replaces one definition in a schema *copy*. Existing fields and
/// variants are cloned exactly; all added payload fields are optional opaque JSON.
fn fragment(entry: &Entry, schema: &Value) -> Value {
    let mut name = entry.definition.as_str();
    if schema["$defs"].get(name).is_none() || entry.carrier == "native" {
        name = "Item";
    }
    let mut def = schema["$defs"][name].clone();
    if let Some(values) = def.get_mut("enum").and_then(Value::as_array_mut) {
        let value = Value::String(snake(&entry.record_type));
        if !values.contains(&value) {
            values.push(value);
        }
    } else if entry.carrier == "field" || (entry.carrier == "ext" && entry.variant.is_some()) {
        let target = if let Some(variant) = &entry.variant {
            def.get_mut("oneOf")
                .and_then(Value::as_array_mut)
                .and_then(|choices| {
                    choices
                        .iter_mut()
                        .find(|node| tag(node).is_some_and(|(_, t)| t == variant))
                })
        } else {
            Some(&mut def)
        };
        if let Some(target) = target
            && let Some(props) = target.get_mut("properties").and_then(Value::as_object_mut)
        {
            for key in keys(entry) {
                props.entry(key).or_insert(json!({}));
            }
        }
    } else {
        let key = if def.get("oneOf").is_some() {
            "oneOf"
        } else {
            "anyOf"
        };
        if let Some(choices) = def.get_mut(key).and_then(Value::as_array_mut) {
            let tag_key = choices
                .iter()
                .find_map(|c| tag(c).map(|(key, _)| key.to_owned()))
                .unwrap_or_else(|| "kind".into());
            let mut kind = snake(&entry.record_type);
            if choices
                .iter()
                .any(|c| tag(c).is_some_and(|(_, k)| k == kind))
            {
                kind = format!("{kind}_{}", &entry.fingerprint[..8]);
            }
            let mut props = serde_json::Map::new();
            props.insert(tag_key.clone(), json!({"type":"string", "const":kind}));
            for field in keys(entry) {
                let field = snake(&field);
                if field != tag_key {
                    props.insert(field, json!({}));
                }
            }
            choices.push(json!({"type":"object", "properties":props, "required":[tag_key]}));
        }
    }
    json!({"$defs":{name:def}})
}

fn keys(entry: &Entry) -> std::collections::BTreeSet<String> {
    entry
        .shape
        .iter()
        .filter_map(|path| {
            let (path, _) = path.rsplit_once(':')?;
            let key = path.strip_prefix('/')?;
            (!key.contains('/') && key != "*").then(|| key.replace("~1", "/").replace("~0", "~"))
        })
        .collect()
}

pub(super) async fn write(scan: &Scan, schema: &Value, dir: &Path) -> io::Result<()> {
    let scan = scan.entries.clone();
    let schema = schema.clone();
    let dir = dir.to_owned();
    bbt_rt::handle().spawn_blocking(move || {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixtures = dir.join("tests/fixtures/ir-v1/pending");
        let tasks = dir.join(".bboxy/tasks");
        let proposals = dir.join("proposals");
        for path in [&fixtures, &tasks, &proposals] { std::fs::create_dir_all(path)?; }
        for entry in scan.values() {
            let fingerprint = &entry.fingerprint;
            let fragment = fragment(entry, &schema);
            let file = proposals.join(format!("{fingerprint}.schema.json"));
            std::fs::write(&file, serde_json::to_vec_pretty(&fragment)?)?;
            let gate = std::process::Command::new("python3").args(["-I", "scripts/ir-compat.py", "check", "--fragment"])
                .arg(&file).current_dir(root).output()?;
            if !gate.status.success() { return Err(io::Error::other(format!("scanner generated a refused proposal: {}", String::from_utf8_lossy(&gate.stdout)))); }
            let header = "// Copyright 2026 Boxy (bboxy450) and contributors.\n// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft\n// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness\n// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE\n// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.\n// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md\n";
            // A compiling no-op is not a projector. This is deliberately a task
            // skeleton, with the exact source identity retained as JSON literals.
            std::fs::write(proposals.join(format!("{fingerprint}.rs")), format!("{header}\n// Provider: {}\nmatch record.get(\"type\").and_then(serde_json::Value::as_str) {{\n    Some({}) => {{\n        // Map to the proposed typed variant and retain unconsumed keys in ext.\n        todo!(\"implement from the attached redacted fixture\");\n    }}\n    _ => preserve_native(record),\n}}\n", json!(entry.provider), json!(entry.record_type)))?;
            let line = json!({"fingerprint":fingerprint,"provider":entry.provider,"record_type":entry.record_type,"sample":entry.sample});
            std::fs::write(fixtures.join(format!("{fingerprint}.jsonl")), format!("{line}\n"))?;
            let number = 1_000_000_000_u64 + u64::from_str_radix(&fingerprint[..12], 16).unwrap();
            let date = date();
            let task = format!("---\nid: {}\nnumber: {number}\ntype: task\ntitle: {}\nstatus: open\nrole: executor\nallowed_paths:\n- src/import/\n- src/backend/acp_agent.rs\n- src/protocol/\n- tests/\nverify: make test-ir\n---\nROLE: EXECUTOR. Worktree: a private worktree from origin/main.\n\nPromote observed {} from {} into the additive IR.\nFingerprint: {fingerprint}.\nSchema: proposals/{fingerprint}.schema.json.\nProjector skeleton: proposals/{fingerprint}.rs.\nFixture: tests/fixtures/ir-v1/pending/{fingerprint}.jsonl.\n\nKeep all existing fields and meanings; map the sample, retain unconsumed data in ext, add a real projector witness and a broken drop control. Move the fixture into a new frozen corpus file. Verify with `make test-ir` and `make test`. No push or merge.\n", ulid::Ulid::new(), json!(format!("Extend IR: {} {}", entry.provider, entry.record_type)), json!(entry.record_type), json!(entry.provider));
            std::fs::write(tasks.join(format!("{date}-ir-extend-{fingerprint}.md")), task)?;
        }
        Ok(())
    }).await.map_err(io::Error::other)?
}
fn date() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;
    let mut tm = std::mem::MaybeUninit::<libc::tm>::uninit();
    unsafe {
        libc::gmtime_r(&now, tm.as_mut_ptr());
    }
    let tm = unsafe { tm.assume_init() };
    format!(
        "{:04}-{:02}-{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday
    )
}
