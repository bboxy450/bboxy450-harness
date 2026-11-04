#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Read-only byte/signature/catalogue proof for the delegated mechanical union."""
import hashlib, json, re, subprocess, tomllib, os
from pathlib import Path
BASE="9ac3c707a57a84db107cb26d21920956a42ebf03"
CANDIDATE="21bd689c070d122932381a5f49d04954adc39bda"
WP6="c98d185bc2182f3a37ceee936031081f8c7cb2bd"
KEY="9B07E849480C68328CA20684844ECC0587DE7AB0"
PAIRS=[("893551168ce40b7c33f7236c29ecac45dabb0aca","c1085e903b375966535bb35bb70eb3f1a87e3543"),("913db696f3ae1acd061d229c0e55acdc12a80b66","620f37b5d847f90682beb5a1b196efb86f4d8ac9"),("0c2706cad61dfdc775de265541d9a22ded906676","012ff149c2f1a70d9614c8611366984893376f02"),(WP6,"21bd689c070d122932381a5f49d04954adc39bda")]
def git(*args):return subprocess.check_output(["git",*args])
def blob(ref,path):return git("show",ref+":"+path)
def sha(data):return hashlib.sha256(data).hexdigest()
def doc(ref,path):return tomllib.loads(blob(ref,path).decode())
rows=[]
for original,new in PAIRS:
 for ref in [original,new]:
  result=subprocess.run(["git","verify-commit","--raw",ref],capture_output=True,text=True)
  identity=git("show","-s","--format=%an|%ae|%cn|%ce|%GF",ref).decode().strip()
  assert result.returncode==0 and "VALIDSIG "+KEY in result.stderr
  assert identity=="Boxy|bboxy450@gmail.com|Boxy|bboxy450@gmail.com|"+KEY
  rows.append(dict(commit=ref,verify_exit=0,identity=identity))
 assert original in git("show","-s","--format=%B",new).decode()
assert git("rev-parse","perf/dream-hashbrown").decode().strip()==BASE
assert git("rev-parse","dream/wp4").decode().strip()=="71b4adab4dd80018de6338ccbfa751ac78649848"
files=[]
exact_wp6=["src/dream/prompt.rs","src/dream/synth.rs","src/process.rs","src/store.rs","tests/cancel_process.rs","tests/dream_pipeline.rs","tests/support/dream.rs","tests/fixtures/dream/synth/tool-output.txt",".bboxy/reports/2026-11-04-dream-wp6-synthesis-r3.md",".bboxy/reports/2026-11-04-dream-wp6-synthesis-r3-resumed.md"]
for path in exact_wp6:
 actual=Path(path).read_bytes();assert actual==blob(WP6,path)
 files.append(dict(path=path,rule="exact_wp6",sha256=sha(actual)))
allowed=set(exact_wp6+["src/dream/mod.rs","tests/formal/models.rs","tests/formal_nonvacuity.rs","verification/corpus.toml","verification/contracts.toml","verification/coverage.toml"])
changes=git("diff","--name-only",BASE,CANDIDATE).decode().splitlines()
assert set(changes)==allowed
retained=[]
for path in git("ls-tree","-r","--name-only",BASE).decode().splitlines():
 if path in allowed or path==".bboxy/runs/formal-native.json":continue
 actual=os.readlink(path).encode() if Path(path).is_symlink() else Path(path).read_bytes()
 assert actual==blob(BASE,path),path
 retained.append(dict(path=path,sha256=sha(actual)))
expected=blob(WP6,"src/dream/mod.rs").replace(b"pub mod resolve;",b"pub mod resolve;\npub mod seal;")
assert Path("src/dream/mod.rs").read_bytes()==expected
p="tests/formal_nonvacuity.rs";delta=blob(WP6,p)[len(blob(PAIRS[2][0]+"^",p)):]
assert blob(WP6,p).startswith(blob(PAIRS[2][0]+"^",p))
assert Path(p).read_bytes()==blob(BASE,p)+delta
p="tests/formal/models.rs"
def functions(raw):
 text=raw.decode();result={}
 for match in re.finditer(r"(?m)^(?:pub )?fn ([A-Za-z0-9_]+)\b",text):
  end=re.search(r"(?m)^}",text[match.end():]);assert end
  result[match.group(1)]=text[match.start():match.end()+end.end()]
 return result
current=functions(Path(p).read_bytes());builders={}
for ref in [BASE,WP6]:
 for name,body in functions(blob(ref,p)).items():
  if name=="corpus":continue
  assert current[name]==body,name
  builders[name]=sha(body.encode())
p="verification/corpus.toml"
a={m["name"]:m for m in doc(BASE,p)["model"]};b={m["name"]:m for m in doc(WP6,p)["model"]}
for name in a.keys()&b.keys():assert a[name]==b[name]
now=tomllib.loads(Path(p).read_text())["model"];assert len(now)==145 and len({m["name"] for m in now})==145
assert {m["name"]:m for m in now}==a|b
contracts=tomllib.loads(Path("verification/contracts.toml").read_text())["contract"]
assert len(contracts)==47 and len({c["name"] for c in contracts})==47
base_contracts={c["name"]:c for c in doc(BASE,"verification/contracts.toml")["contract"]}
for contract in contracts:
 if contract["name"] in base_contracts:
  old=base_contracts[contract["name"]]
  for field in ["models","controls","property","checks"]:
   assert contract.get(field)==old.get(field),(contract["name"],field)
  if contract["name"] not in ["HarnessDreamLifecycle","HarnessProviderStream"]:
   assert contract==old,contract["name"]
cover=tomllib.loads(Path("verification/coverage.toml").read_text())["crate"];assert len(cover)==1
for crate in cover:
 for path in crate["sources"]:assert Path(path).is_file(),path
 for path in crate.get("planned_sources",[]):assert not Path(path).exists(),path
base_native=Path(".bboxy/runs/dream-integration-base-formal-native.json").read_bytes()
assert sha(base_native)=="d2b4d9b31de910313fbaefeb1c9e00efd1875605eb31b8fad27d5dd43b5786f8"
result=dict(source=CANDIDATE,base=BASE,wp6=WP6,signatures=rows,exact_payloads=files,retained_base_files=retained,model_function_sha256=builders,models=now,contracts=contracts,crate_inventory=cover,changed_paths=changes,base_native_sha256=sha(base_native),result="PASS")
Path(".bboxy/runs/dream-integration-proof.json").write_text(json.dumps(result,indent=2)+"\n")
print(json.dumps(dict(result="PASS",signature_count=len(rows),wp6_exact_files=len(files),retained_files=len(retained),exact_model_builders=len(builders),model_count=len(now),contracts=len(contracts),crate_count=len(cover))))
