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
def owner_wait(source,replacement):
    return once(source,'self.retire_for_reap();\n        let result = self.child.wait().await;',replacement+'\n        let result = self.child.wait().await;')
def leader_only(source):
    return owner_wait(source,'let _=self.child.start_kill();\n        self.group.reaped();')
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
        ('group-kill','src/process.rs','mcp_client','close_kills_a_server_and_member_that_ignore_eof_and_term','MCP server group survived Close',
         lambda s:owner_wait(s,'/* group fence removed by control */')),
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
        ('mcp-exited-term','src/mcp.rs','mcp_client','mcp_finish_exited_leader_skips_term_and_grace_but_kills_members','already-exited MCP cleanup sent TERM',
         lambda s:once(s,'    // WNOWAIT retains the leader', '    child.send(libc::SIGTERM);\n    // WNOWAIT retains the leader')),
        ('mcp-exited-grace','src/mcp.rs','mcp_client','mcp_finish_exited_leader_skips_term_and_grace_but_kills_members','already-exited MCP cleanup slept through grace',
         lambda s:once(s,'    // WNOWAIT retains the leader', '    tokio::time::sleep(Duration::from_secs(1)).await;\n    // WNOWAIT retains the leader')),
        ('pty-spawn-fds','tests/seat_process.rs','seat_process','non_pty_spawn_does_not_keep_a_sibling_master_alive','non-PTY child retained a sibling master',
         lambda s:once(s,'portable_pty::unix::close_random_fds();','/* descriptor fence removed */')),
        ('pty-hangup-poll','src/frontends/headless.rs','lib','frontends::headless::tty_tests::physical_eof_is_distinct_from_cooked_control_d','physical PTY EOF was classified as cooked Ctrl-D',
         lambda s:once(s,'// Darwin omits POLLHUP when no events are requested.\n        events: libc::POLLIN,','// Darwin omits POLLHUP when no events are requested.\n        events: 0,')),
        ('pty-writer-drop','tests/support/seat_shutdown.rs','seat_process','seat_shutdown::writer_drop_is_not_cooked_eof','writer drop injected terminal input',
         lambda s:region(s,'fn terminal_writer(','fn wait_for(','fn terminal_writer(master:&dyn portable_pty::MasterPty)->Box<dyn std::io::Write+Send> {master.take_writer().unwrap()}\n')),
        ('reap-lease','src/process.rs','lib','process::ownership_tests::echild_after_owned_reap_never_leaves_a_signal_target','ECHILD observed a PID released while its group was registered',
         lambda s:owner_wait(s,'/* reap lease removed */')),
        ('mcp-reap-ownership','src/process.rs','mcp_client','mcp_finish_*',['already-fenced cleanup signalled a foreign pgid after reap','failed-handoff cleanup signalled a foreign pgid after reap'],
         lambda s:owner_wait(s,'let _=self.child.start_kill();')),
        ('sigchld-admission','src/process.rs','lib','process::ownership_tests::automatic_sigchld_reaping_is_refused_before_spawn','automatic SIGCHLD reaping was admitted',
         lambda s:region(s,'fn check_sigchld(','impl OwnedChild {','fn check_sigchld(_action:&libc::sigaction)->io::Result<()> {Ok(())}\n')),
        ('reap-lease-model','tests/formal/models.rs','formal_nonvacuity','reap_identity_is_retired_and_both_mcp_notifications_complete','NoSignalToEmptyGroup',
         lambda s:once(s,'if mode == 5 { Var(9) } else { Int(0) }','if mode == 0 || mode == 5 { Var(9) } else { Int(0) }')),
        ('seat-hangup','src/process.rs','seat_process','seat_shutdown::hup_kills_owned_groups_before_supervisor_escalation_and_resume_recovers','owned group survived supervisor escalation',
         lambda s:region(s,'pub(crate) fn kill_owned_groups()','extern "C" fn shutdown_signal','pub(crate) fn kill_owned_groups() {}\n')),
        ('seat-hangup-model','tests/formal/models.rs','formal_nonvacuity','synchronous_hangup_kill_precedes_the_supervisors_unjoinable_exit','NoOwnedMemberAfterSupervisorKill',
         lambda s:once(s,'if mode == 4 { Var(1) } else { Int(0) }','if mode == 0 || mode == 4 { Var(1) } else { Int(0) }')),
        ('leader-only','src/process.rs','mcp_client','close_kills_a_server_and_member_that_ignore_eof_and_term','MCP server group survived Close: orphaned member',leader_only),
        ('bboxy-indirection','src/mcp.rs','seat_process','seat_launch::profile_and_acp_refuse_bboxy_environment_indirection','BBOXY reference lacked typed configuration refusal',
         lambda s:once(s,'origin != Origin::Launcher\n        && env','false\n        && env')),
        ('resolver-namespace','src/process.rs','seat_process','seat_launch::command_resolver_cannot_copy_seat_environment_into_mcp_aliases','command resolver leaked BBOXY_SOCK into an MCP alias',
         lambda s:once(s,'name.as_encoded_bytes().starts_with(b"BBOXY_")','name.as_encoded_bytes() == b"BBOXY_CAPABILITY"')),
        ('bboxy-scrub','src/mcp.rs','seat_process','cannot_inherit_bboxy_environment*',['profile MCP inherited BBOXY variables','ACP MCP inherited BBOXY variables'],
         lambda s:once(s,'scrub_bboxy_env(&mut command);','/* scrub removed by control */')),
        ('launcher-inherit','src/mcp.rs','seat_process','seat_launch::launcher_flags_deliver_context_tools_and_only_launcher_environment','launcher MCP lost its capability file',
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
