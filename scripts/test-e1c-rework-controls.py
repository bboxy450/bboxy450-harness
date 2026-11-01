# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused PR #6 rejection controls; restore every production mutation."""
import fcntl
import argparse
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
def leader_only(source):
    start=source.index('async fn finish_child(');end=source.index('async fn supervise(',start)
    body=source[start:end];before='libc::kill(-(pid as i32),'
    if body.count(before)!=2:raise ValueError('group-signal mutation anchor drift')
    return source[:start]+body.replace(before,'libc::kill(pid as i32,')+source[end:]
def main(selected=None):
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
        ('acp-engine-writer','tests/formal/models.rs','formal_nonvacuity','acp_engine_emit_stays_enabled_with_a_full_writer','EngineNeverWaits',
         lambda s:once(s,'and(if mode == 3 {','and(if mode == 0 || mode == 3 {')),
        ('mcp-close-model','tests/formal/models.rs','formal_nonvacuity','mcp_close_joins_members_even_when_term_is_ignored','NoMemberAfterClose',
         lambda s:once(s,'if leader_only { Var(1) } else { Int(0) }','if true { Var(1) } else { Int(0) }')),
        ('admission-lag','src/frontends/acp.rs','lib','frontends::acp::tests::admission_lag_replays_the_durable_prompt_before_next_input','admission-lag hung without replay',
         lambda s:region(s,'async fn await_prompt_admission(','impl Face {', '''async fn await_prompt_admission(
    _gateway: &InProc,op_id:OpId,
    mut events:std::pin::Pin<Box<dyn futures_core::Stream<Item=Envelope>+Send>>,
    cancel:&CancellationToken,
) -> Option<Envelope> {
    use futures_util::StreamExt;
    loop {
        let event=tokio::select! {biased;_=cancel.cancelled()=>return None,event=events.next()=>event};
        let event=event?;
        if event.cause==Some(op_id) {return Some(event);}
    }
}
''')),
        ('leader-only','src/mcp.rs','mcp_client','close_kills_a_server_and_member_that_ignore_eof_and_term','MCP server group survived Close: orphaned member',leader_only),
        ('bboxy-scrub','src/mcp.rs','seat_process','cannot_inherit_bboxy_environment*',['profile MCP inherited BBOXY variables','ACP MCP inherited BBOXY variables'],
         lambda s:once(s,'scrub_bboxy_env(&mut command);','/* scrub removed by control */')),
        ('launcher-inherit','src/mcp.rs','seat_process','seat_launch::launcher_flags_deliver_context_tools_and_only_launcher_environment','launcher MCP lost BBOXY capability',
         lambda s:once(s,'if slot.origin!=Origin::Launcher {scrub_bboxy_env(&mut command);}','scrub_bboxy_env(&mut command);')),
        ('append-system','src/engine/mod.rs','lib','engine::hardening_tests::launch_context_is_in_the_first_scripted_request_stable_prefix','appended startup text absent from stable system prefix',
         lambda s:once(s,'if !self.append_system_prompt.is_empty() {','if false {')),
        ('missing-mcp-config','src/frontends/headless.rs','seat_process','seat_launch::unavailable_config_is_a_notice_and_the_turn_completes','unavailable MCP config did not produce a Notice',
         lambda s:once(s,'.push("launcher MCP configuration unavailable or invalid".into())','.extend(std::iter::empty::<String>())')),
    ]
    if selected:
        unknown=set(selected)-{row[0] for row in cases}
        if unknown:raise ValueError('unknown controls: '+str(sorted(unknown)))
        cases=[row for row in cases if row[0] in selected]
    OUT.mkdir(parents=True,exist_ok=True);records=[]
    for name,path,suite,test,expected,mutate in cases:
        file=ROOT/path;source=file.read_text()
        target=['--lib'] if suite=='lib' else ['--test',suite]
        selector=[test[:-1]] if test.endswith('*') else [test,'--','--exact']
        command=['cargo','test','--offline','--locked','--quiet','--features','acp,test-fixtures',*target,*selector]
        row={'name':name,'source':path,'source_sha256':hashlib.sha256(source.encode()).hexdigest(),'command':command,'red_assertion':expected}
        try:
            file.write_text(mutate(source))
            result=subprocess.run(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=600)
            (OUT/f'{name}-red.log').write_text('\n'.join(l.rstrip() for l in result.stdout.rstrip().splitlines())+'\n')
            row['red_exit']=result.returncode
            assertions=expected if isinstance(expected,list) else [expected]
            if result.returncode!=101 or any(assertion not in result.stdout for assertion in assertions):raise ValueError('mutation missed its behavioral witness: '+name)
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
        parser=argparse.ArgumentParser();parser.add_argument('--case',action='append');parser.add_argument('--out',type=Path,default=OUT)
        args=parser.parse_args();OUT=args.out
        raise SystemExit(main(args.case))
