# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused PR #6 rejection controls; restore every production mutation."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'.bboxy/runs/e1c-rework'
def region(source, start, end, replacement):
    a=source.index(start);b=source.index(end,a)
    return source[:a]+replacement+source[b:]
def once(source,before,after):
    if source.count(before)!=1:raise ValueError('mutation anchor drift: '+before)
    return source.replace(before,after)
def main():
    cases=[
        ('backpressure','src/frontends/acp.rs','acp_face','slow_reader_resumes_without_losing_the_turn','slow reader lost the ACP connection',
         lambda s:region(s,'    async fn emit(','    async fn response(','''    async fn emit(&self,value:Value) {
        let Some(value)=bound(self.config.redactor.value(value)) else {return;};
        if self.output.try_send(value).is_err(){self.cancel.cancel();}
    }
''')),
        ('ordering','src/frontends/acp.rs','acp_face','back_to_back_prompt_and_cancel_preserve_session_order','cancel overtook prompt admission',
         lambda s:region(s,'    fn route(','    async fn emit(','''    fn route(self:&Arc<Self>,request:Value) {
        let face=self.clone();self.spawn(async move {face.dispatch(request).await;});
    }
''')),
        ('content','src/frontends/acp.rs','acp_face','scripted_client_full_turn_and_permission_keep_the_dispatcher_live','final tool card lost the diff',
         lambda s:region(s,'    fn content(','    fn suffix(','''    fn content(&mut self,_parent:ItemId,_item:ItemId,value:Value)->Value {value}
''')),
        ('group-kill','src/mcp.rs','mcp_client','close_kills_a_server_and_member_that_ignore_eof_and_term','MCP server group survived Close',
         lambda s:region(s,'async fn finish_child(','async fn supervise(','''async fn finish_child(rx:oneshot::Receiver<Child>) {
    if let Ok(mut child)=rx.await {let _=child.wait().await;}
}
''')),
        ('prefix-length','src/context/mod.rs','context_auto','thirty_turns_trim_then_summarize_and_share_the_stable_prefix','reduced cache prefix contains only the system prompt',
         lambda s:once(s,'let prefix = 1 + messages.len() + history::to_wire(&stable).len();','let prefix = 1;')),
        ('prefix-drift','src/context/mod.rs','context_auto','thirty_turns_trim_then_summarize_and_share_the_stable_prefix','cache prefix changed without a reduction',
         lambda s:once(s,'"content":summary','"content":format!("{summary} [{}]",log.last().unwrap().at.0)')),
        ('trim-only','src/engine/mod.rs','context_auto','trimming_alone_fits_without_a_following_summary','summary followed a sufficient trim',
         lambda s:once(s,'if auto::fits(','if false && auto::fits(')),
    ]
    OUT.mkdir(parents=True,exist_ok=True);records=[]
    for name,path,suite,test,expected,mutate in cases:
        file=ROOT/path;source=file.read_text()
        command=['cargo','test','--offline','--locked','--quiet','--features','acp,test-fixtures','--test',suite,test,'--','--exact']
        row={'name':name,'source':path,'source_sha256':hashlib.sha256(source.encode()).hexdigest(),'command':command,'red_assertion':expected}
        try:
            file.write_text(mutate(source))
            result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=600)
            (OUT/f'{name}-red.log').write_text('\n'.join(l.rstrip() for l in result.stdout.rstrip().splitlines())+'\n')
            row['red_exit']=result.returncode
            if result.returncode!=101 or expected not in result.stdout:raise ValueError('mutation missed its behavioral witness: '+name)
        finally:file.write_text(source)
        result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=600)
        (OUT/f'{name}-green.log').write_text('\n'.join(l.rstrip() for l in result.stdout.rstrip().splitlines())+'\n')
        row['green_exit']=result.returncode;records.append(row)
        (OUT/'controls.json').write_text(json.dumps(records,indent=2)+'\n')
        print(f'{name}: {row["red_exit"]} -> {row["green_exit"]}',flush=True)
        if result.returncode:raise ValueError('restored implementation failed: '+name)
    return 0
if __name__=='__main__':
    claim=ROOT/'target/e1c-controls.claim';claim.parent.mkdir(parents=True,exist_ok=True)
    with claim.open('a+') as f:
        try:fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except BlockingIOError:raise SystemExit('another mutation pass owns this worktree')
        f.seek(0);f.truncate();f.write(str(os.getpid())+'\n');f.flush()
        raise SystemExit(main())
