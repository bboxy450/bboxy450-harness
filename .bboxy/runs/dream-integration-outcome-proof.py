#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Check the authorized outcome-only delta and byte-identical old model graphs."""
import hashlib, json, os, re, subprocess, tomllib
from pathlib import Path
BASE="9c3251e01348a6ab96804a7473fe17f8d1234e4a"
SOURCE="9fb4b1111b6e524a7c6de070156b33856ef87b1e"
PREVIOUS="41ffbe70e3d5cbbd0f6cc680a1c402fbcd8e651936336692b66d2cf58ed008ab"
ALLOWED={"src/process.rs","tests/cancel_process.rs","tests/formal/models.rs","tests/formal_nonvacuity.rs","verification/corpus.toml","verification/contracts.toml","verification/coverage.toml"}
def git(*args):return subprocess.check_output(["git",*args])
def old(path):return git("show",BASE+":"+path)
def sha(raw):return hashlib.sha256(raw).hexdigest()
assert set(git("diff","--name-only",BASE,SOURCE).decode().splitlines())==ALLOWED
assert subprocess.run(["git","diff","--quiet",SOURCE,"--","src","tests","scripts","Cargo.toml","Cargo.lock","verification","protocol.schema.json","ATTRIBUTION.md"]).returncode==0
retained=[]
for line in git("ls-tree","-r","-z",BASE).split(b"\0"):
 if not line:continue
 meta,name=line.split(b"\t",1);mode,kind,oid=meta.split();path=name.decode()
 if path in ALLOWED or path==".bboxy/runs/formal-native.json":continue
 raw=os.readlink(path).encode() if mode==b"120000" else Path(path).read_bytes()
 assert hashlib.sha1(b"blob "+str(len(raw)).encode()+b"\0"+raw).hexdigest()==oid.decode(),path
 retained.append(dict(path=path,sha256=sha(raw)))
p="src/process.rs";before=old(p).decode();after=Path(p).read_text()
start=after.index("// Decision d20261103136567:");end=after.index("async fn spawn_admission(",start)
restored=after[:start]+after[end:]
start=restored.index("    #[tokio::test]\n    async fn closed_elapsed_result_beats_a_not_ready_caller_timer()")
end=restored.index("    #[tokio::test]\n    async fn queued_cancel_drop_and_deadline_do_not_need_permit_service()",start)
restored=restored[:start]+restored[end:]
new="        result=receive=>observe_spawn_result(result, cancel.is_cancelled(), Instant::now(), deadline),"
start=before.index("        result=receive=>{");end=before.index("        },",start)+len("        },")
assert new in restored;restored=restored.replace(new,before[start:end],1)
assert restored==before,"process changes escaped the authorized helper/receive/tests boundary"
for p in ["tests/cancel_process.rs","tests/formal_nonvacuity.rs","verification/corpus.toml","verification/contracts.toml"]:
 assert Path(p).read_bytes().startswith(old(p)),p
p="tests/formal/models.rs"
def functions(raw):
 t=raw.decode();out={}
 for m in re.finditer(r"(?m)^(?:pub )?fn ([A-Za-z0-9_]+)\b",t):
  e=re.search(r"(?m)^}",t[m.end():]);out[m[1]]=t[m.start():m.end()+e.end()]
 return out
original=functions(old(p));current=functions(Path(p).read_bytes())
for name,body in original.items():
 if name!="corpus":assert current[name]==body,name
assert set(current)-set(original)=={"process_admission_result"}
for p,key,expected in [("verification/corpus.toml","model",147),("verification/contracts.toml","contract",48)]:
 original_doc=tomllib.loads(old(p).decode())[key];new_doc=tomllib.loads(Path(p).read_text())[key]
 assert new_doc[:len(original_doc)]==original_doc and len(new_doc)==expected
 assert len({r['name'] for r in new_doc})==expected
p="verification/coverage.toml";a=tomllib.loads(old(p).decode())["crate"];b=tomllib.loads(Path(p).read_text())["crate"]
assert len(a)==len(b)==1
for key,value in a[0].items():
 actual=b[0][key]
 if key in ["models","implementation_only"]:assert actual[1:]==value
 else:assert actual==value
previous=Path("verification/evidence")/PREVIOUS/"models";current=Path("target/formal/models")
models=[]
for row in (previous/"manifest.tsv").read_text().splitlines():
 name=row.split("\t")[0];hashes={}
 for suffix in [".states",".edges",".metadata.json",".tla",".cfg","Graph.cfg"]:
  raw=(current/(name+suffix)).read_bytes();assert raw==(previous/(name+suffix)).read_bytes(),name+suffix
  hashes[suffix]=sha(raw)
 assert row in (current/"manifest.tsv").read_text().splitlines()
 models.append(dict(name=name,artifacts=hashes))
assert len(models)==145
manifest=(current/"manifest.tsv").read_text().splitlines();assert len(manifest)==147
receipt=json.loads(Path("target/formal/evidence.json").read_text())
assert len(receipt['models'])==147 and len(receipt['contracts'])==48 and all(m['graph_equal'] for m in receipt['models'])
result=dict(source=SOURCE,base=BASE,allowed_delta=sorted(ALLOWED),retained=retained,unchanged_model_functions=len(original)-1,unchanged_model_artifacts=models,model_count=147,contracts=48,crate_count=1,process_worker_packet_store_unchanged=True,old_assertions_unchanged=True,result="PASS")
Path('.bboxy/runs/dream-integration-outcome-proof.json').write_text(json.dumps(result,indent=2)+"\n")
print(json.dumps(dict(result="PASS",retained_files=len(retained),unchanged_functions=len(original)-1,old_models_exact=145,new_models=147,contracts=48)))
