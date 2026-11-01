# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Deliberately break one boundary at a time; restore source even on failure.
Run only in this Executor worktree, with no concurrent source-writing command.
Compiler failures never count as successful negative controls.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
RUNS=ROOT/'.bboxy/runs/hardening'
def replace_once(source,old,new):
    if source.count(old)!=1:raise ValueError('control needle is not unique')
    return source.replace(old,new,1)
def deny_allow(source):
    a=source.index('                    Decision::Deny { reason } => {');b=source.index('                    Decision::Ask',a)
    block=source[a:b];old='ApprovalDecision::Deny {\n                                reason: Some(reason),\n                            }'
    return source[:a]+replace_once(block,old,'ApprovalDecision::Allow')+source[b:]
def log_only_epoch(source):
    a=source.index('                read_epoch(&fork_root.join(session.to_string()))?');b=source.index('                    .checked_add(1)',a)
    return source[:a]+'                observed\n'+source[b:]
CASES=[
 ('item08-fence','src/engine/mod.rs',lambda s:replace_once(s,'.is_none_or(|r| r.generation != generation)','.is_none_or(|_| false)'),['cargo','test','--offline','--locked','--lib','a_b_a_is_fenced_by_birth_not_identity']),
 ('item09-deny','src/engine/mod.rs',deny_allow,['cargo','test','--offline','--locked','--test','policy_gate','deny_star_never_executes_the_counting_tool']),
 ('item09-stale','src/engine/mod.rs',lambda s:replace_once(s,'Op::ResolveApproval { approval, decision } => {','Op::ResolveApproval { approval, decision } => { let approval=self.approvals.keys().next().copied().unwrap_or(approval);'),['cargo','test','--offline','--locked','--test','policy_gate','old_turn_approval_cannot_resolve_the_current_turn']),
 ('item10-control','src/engine/mod.rs',lambda s:replace_once(s,'r.cause = Some(id);\n                    r.cancel.cancel();\n                    self.cancel_approvals(target).await?;','r.cause = Some(id);\n                    let _=r.cancel.is_cancelled();\n                    self.cancel_approvals(target).await?;'),['cargo','test','--offline','--locked','--test','gateway_ops','interrupt_when_full']),
 ('item14-old-reset','src/gateway/mod.rs',lambda s:replace_once(s,'req.after.is_some_and(|(_, s)| s > trusted)','req.after.is_some_and(|(e,s)|e!=hello.epoch&&s>hello.head)'),['cargo','test','--offline','--locked','--test','store_recovery','trusted_head_cursors']),
 ('item14-lost-epoch','src/store.rs',log_only_epoch,['cargo','test','--offline','--locked','--test','store_recovery','trusted_head_cursors']),
 ('item15-worker','src/tools.rs',lambda s:replace_once(s,'let result = blocking(move || file_tool(&name, &a, &c)).await;','let result = file_tool(&name, &a, &c);'),['cargo','test','--offline','--locked','--test','rt_budget','expensive_entries_hold_four_permits_off_the_io_workers']),
 ('item15-permits','src/runtime.rs',lambda s:replace_once(s,'Semaphore::new(4)','Semaphore::new(8)'),['cargo','test','--offline','--locked','--test','rt_budget','expensive_entries_hold_four_permits_off_the_io_workers']),
]
def run(label,command):
    r=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=120)
    (RUNS/(label+'.log')).write_text(r.stdout)
    return {'command':command,'exit_code':r.returncode,'test_assertion_failed':'test result: FAILED' in r.stdout}
def main():
    RUNS.mkdir(parents=True,exist_ok=True)
    selected=set(sys.argv[1:])
    if selected-set(c[0] for c in CASES):raise ValueError('unknown control')
    summary=RUNS/'fault-controls.json'
    rows=json.loads(summary.read_text()) if summary.exists() else []
    for label,path,mutate,command in CASES:
        if selected and label not in selected:continue
        p=ROOT/path;before=p.read_bytes();digest=hashlib.sha256(before).hexdigest()
        try:
            p.write_text(mutate(before.decode()));red=run(label+'-red',command)
            if red['exit_code']!=101 or not red['test_assertion_failed']:raise ValueError(label+' did not fail a running test')
        finally:p.write_bytes(before)
        if hashlib.sha256(p.read_bytes()).hexdigest()!=digest:raise ValueError('source restore failed')
        green=run(label+'-green',command)
        row={'item':label,'source':path,'restored_sha256':digest,'red':red,'green':green};rows=[r for r in rows if r['item']!=label]+[row]
        (RUNS/(label+'.json')).write_text(json.dumps(row,indent=2)+'\n');print(label,red['exit_code'],'->',green['exit_code'],flush=True)
        if green['exit_code']!=0:raise ValueError(label+' did not recover')
    (RUNS/'fault-controls.json').write_text(json.dumps(rows,indent=2)+'\n')
if __name__=='__main__':main()
