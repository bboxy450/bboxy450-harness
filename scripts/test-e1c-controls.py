# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""E1c red/green process witnesses. Restore each scoped mutation even on failure."""
import fcntl
import os
import hashlib
import json
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'.bboxy/runs/e1c'
def main():
    OUT.mkdir(parents=True,exist_ok=True)
    cases=[
        ('timeout','src/mcp.rs','Duration::from_secs(self.slot.config.timeout_s)','Duration::from_secs(60)','timeout_is_labelled_cancelled_and_turn_continues'),
        ('crash','src/mcp.rs','_ => return ToolOutput::error(format!("mcp {}: server exited", self.slot.name)),','_ => std::future::pending::<rmcp::model::CallToolResult>().await,','crash_mid_call_is_failed_and_turn_continues'),
        ('startup','src/engine/mod.rs','let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;','while self.mcp.snapshot().0.is_empty() { tokio::time::sleep(std::time::Duration::from_millis(10)).await; }\n        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;','starting_hung_server_never_blocks_a_turn'),
    ]
    mcp=(ROOT/'src/mcp.rs').read_text()
    group=mcp[mcp.index('async fn finish_child('):mcp.index('async fn supervise(')]
    cases.append(('group_kill','src/mcp.rs',group,'async fn finish_child(rx: oneshot::Receiver<Child>) { if let Ok(mut child)=rx.await { let _=child.wait().await; } }\n','close_kills_a_server_and_member_that_ignore_eof_and_term'))
    profile=(ROOT/'src/provider/profile.rs').read_text()
    before=profile[profile.index('pub(crate) async fn resolve_command'):]
    baseline=subprocess.check_output(['git','show','origin/main:src/provider/profile.rs'],cwd=ROOT,text=True)
    after=baseline[baseline.index('struct Wake('):].replace('Arc::','std::sync::Arc::').replace('async fn resolve_command(','pub(crate) async fn resolve_command(')
    cases.append(('resolver','src/provider/profile.rs',before,after,'eof_cancels_a_pending_profile_factory_without_waiting_for_its_key_command'))
    records=[]
    for name,path,before,after,test in cases:
        path=ROOT/path
        source=path.read_text()
        if source.count(before)!=1:raise ValueError('control anchor drift: '+name)
        command=['cargo','test','--offline','--locked','--quiet','--features','acp,test-fixtures','--test','acp_face' if name=='resolver' else 'mcp_client',test,'--','--exact']
        row={'name':name,'source':str(path.relative_to(ROOT)),'source_sha256':hashlib.sha256(source.encode()).hexdigest(),'command':command,'mutation':after if name!='resolver' else 'restore the prior CommandRunner resolver body'}
        try:
            path.write_text(source.replace(before,after))
            result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=600)
            (OUT/f'{name}-red.log').write_text(result.stdout)
            row['red_exit']=result.returncode
            expected={'timeout':'mcp fixture: timed out after 1 s','resolver':'EOF waited for the key command','group_kill':'MCP server group survived Close'}.get(name,'MCP must not block turn')
            row['red_assertion']=expected
            if result.returncode!=101 or expected not in result.stdout:raise ValueError('control did not fail at its behavioral witness: '+name)
        finally:
            path.write_text(source)
        result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=600)
        (OUT/f'{name}-green.log').write_text(result.stdout)
        row['green_exit']=result.returncode
        records.append(row)
        (OUT/'process-controls.json').write_text(json.dumps(records,indent=2)+'\n')
        print(f'{name}: {row["red_exit"]} -> {row["green_exit"]}',flush=True)
        if result.returncode:raise ValueError('restored implementation did not pass: '+name)
    return 0
if __name__=='__main__':
    claim=ROOT/'target/e1c-controls.claim'
    claim.parent.mkdir(parents=True,exist_ok=True)
    with claim.open('a+') as handle:
        try:fcntl.flock(handle,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except BlockingIOError:raise SystemExit('another mutation pass owns this worktree; no wait')
        handle.seek(0);handle.truncate();handle.write(str(os.getpid())+'\n');handle.flush()
        raise SystemExit(main())
