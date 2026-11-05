// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Canonical transport corpus, independent hashes and private worker witnesses.
use bboxy450_harness::dream::c1_wire as w;
use std::path::Path;
fn root()-> &'static Path {Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/dream-c1"))}
fn dto(name:&str,bytes:&[u8])->Result<Vec<u8>,w::Code> {
    macro_rules! read {($t:ty)=>{w::canonical(&w::decode::<$t>(bytes)?).map_err(|_|w::Code::ProtocolError)}}
    match name {
        "manifest"=>read!(w::Manifest),"page"=>read!(w::Page),"result"=>read!(w::ResultDescriptor),
        "start"|"activity"|"worker-cancel"=>read!(w::WorkerIn),
        "hello"|"progress"|"done"=>read!(w::WorkerOut),
        "inspect"|"status"|"cancel"=>read!(w::Control),
        "success"|"failure"=>read!(w::ControlReply),_=>panic!("unknown fixture {name}")
    }
}
#[test]
fn wire_schema_and_vectors() {
    let schema=w::schema_bytes();
    let path=Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/schema/dream-c1-v1.schema.json"));
    if std::env::var_os("DREAM_C1_GENERATE").is_some(){std::fs::write(path,&schema).unwrap();}
    assert_eq!(std::fs::read(path).unwrap(),schema,"regenerate schema from c1_wire only");
    let lock:serde_json::Value=serde_json::from_slice(&std::fs::read(root().join("lock.json")).unwrap()).unwrap();
    assert_eq!(w::hash(&schema).0,lock["schema_sha256"]);
    for (path,hash) in lock["files"].as_object().unwrap(){assert_eq!(w::hash(std::fs::read(root().join(path)).unwrap()).0,hash.as_str().unwrap());}
    let vectors:Vec<serde_json::Value>=serde_json::from_slice(&std::fs::read(root().join("vectors/canonical.json")).unwrap()).unwrap();
    for v in vectors {let bytes=w::canonical(&v["value"]).unwrap();assert_eq!(bytes,v["canonical"].as_str().unwrap().as_bytes());assert_eq!(w::hash(bytes).0,v["sha256"]);}
    for entry in std::fs::read_dir(root().join("valid")).unwrap(){let path=entry.unwrap().path();let bytes=std::fs::read(&path).unwrap();assert_eq!(dto(path.file_stem().unwrap().to_str().unwrap(),&bytes).unwrap(),bytes);}
    for entry in std::fs::read_dir(root().join("invalid")).unwrap(){let path=entry.unwrap().path();assert!(w::decode::<w::WorkerIn>(&std::fs::read(&path).unwrap()).is_err(),"{}",path.display());}
}
#[test]
fn independent_python_canonical_hashes() {
    let result=std::process::Command::new("python3").args(["-I","-c",r#"
import hashlib,json,pathlib,sys
p=pathlib.Path(sys.argv[1]); vectors=json.loads((p/'vectors/canonical.json').read_bytes())
for v in vectors:
    b=json.dumps(v['value'],ensure_ascii=False,sort_keys=True,separators=(',',':'),allow_nan=False).encode()
    assert b == v['canonical'].encode()
    assert hashlib.sha256(b).hexdigest() == v['sha256']
print('independent canonical vectors:',len(vectors))
"#]).arg(root()).output().unwrap();assert!(result.status.success(),"{:?}",result);
}
