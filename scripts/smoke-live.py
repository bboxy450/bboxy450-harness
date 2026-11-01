# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""At most three provider requests. Persist only a redacted classification."""
import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
def main():
    runs=ROOT/'.bboxy/runs';runs.mkdir(parents=True,exist_ok=True)
    path=runs/('smoke-live-'+datetime.date.today().isoformat()+'.json')
    previous=json.loads(path.read_text()) if path.exists() else None
    used=previous.get('requests_total',previous.get('requests',0)) if previous else 0
    env=os.environ.copy();env['HARNESS_SMOKE_REQUEST_BUDGET']='3'
    faces={}
    for face,command in [('headless','smoke-live'),('acp','smoke-live-acp')]:
        result=subprocess.run(['cargo','run','--offline','--locked','--quiet','--features','acp','--',command],cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:record=json.loads(result.stdout)
        except (ValueError,TypeError):
            record={'outcome':'FAIL','reason':'smoke runner failed','redacted':True,'requests':0}
            sys.stderr.write(result.stderr)
        record['exit_code']=result.returncode
        faces[face]=record
    outcome=next((label for label in ['FAIL','UNAVAILABLE','SKIP'] if any(r['outcome']==label for r in faces.values())),'PASS')
    record={'faces':faces,'outcome':outcome,'reason':'; '.join(f"{name}: {r['outcome']} ({r['reason']})" for name,r in faces.items()),'requests':sum(r.get('requests',0) for r in faces.values()),'redacted':True}
    code=1 if outcome=='FAIL' else 75 if outcome=='UNAVAILABLE' else 0
    record['requests_total']=used+record.get('requests',0)
    record['prior_attempts']=(previous.get('prior_attempts',[])+[{k:v for k,v in previous.items() if k!='prior_attempts'}]) if previous else []
    record['exit_code']=code
    record['at']=datetime.datetime.now().astimezone().isoformat()
    record['source']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    runs=ROOT/'.bboxy/runs';runs.mkdir(parents=True,exist_ok=True)
    path=runs/('smoke-live-'+datetime.date.today().isoformat()+'.json')
    path.write_text(json.dumps(record,indent=2)+'\n')
    print('smoke-live: '+record['outcome']+'; '+record['reason'])
    return code
if __name__=='__main__':sys.exit(main())
