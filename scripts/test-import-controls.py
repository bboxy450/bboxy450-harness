# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Synthetic-only importer controls; compilation or generic worker errors do not count as red."""
import hashlib,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'src/import/claude/engine.rs'
LOGS=ROOT/'target/import-controls'
CASES=[
 ('redaction','let record = self.options.redactor.import_value(record);','let record = record;','planted_shapes_and_registered_values_never_reach_envelopes_or_blobs','ImportSecretShapesRedacted'),
 ('unknown','self.metrics.unknown += 1;','panic!("deliberate unknown-record control");','unknown_malformed_and_truncated_input_never_panics_and_finishes_lawfully','ImportUnknownNeverPanics'),
]
def main():
 env={k:v for k,v in os.environ.items() if not k.startswith('BBOXY_')}
 LOGS.mkdir(parents=True,exist_ok=True)
 original=SOURCE.read_bytes();receipts=[]
 for name,old,new,test,assertion in CASES:
  text=original.decode();assert text.count(old)==1
  command=['cargo','test','--offline','--locked','--test','import_claude',test,'--','--exact','--nocapture']
  try:
   SOURCE.write_text(text.replace(old,new))
   with (LOGS/(name+'-red.log')).open('wb') as log:red=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  finally:SOURCE.write_bytes(original)
  assert SOURCE.read_bytes()==original
  output=(LOGS/(name+'-red.log')).read_text()
  assert red.returncode==101 and 'running 1 test' in output and 'test result: FAILED' in output and assertion in output,name+' did not fail at its named assertion'
  with (LOGS/(name+'-green.log')).open('wb') as log:green=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  assert green.returncode==0,name+' was not green after restoration'
  receipt=dict(name=name,command=command,assertion=assertion,red_exit=red.returncode,green_exit=green.returncode,restored_source_sha256=hashlib.sha256(original).hexdigest(),synthetic_only=True,operator_input=False)
  receipts.append(receipt);print(json.dumps(receipt),flush=True)
 (LOGS/'receipt.json').write_text(json.dumps(receipts,indent=2)+'\n')
if __name__=='__main__':main()
