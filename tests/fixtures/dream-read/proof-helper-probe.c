// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Private Darwin capability witness; neither a product helper nor an authority source.
#define _DARWIN_C_SOURCE 1
#include <errno.h>
#include <fcntl.h>
#include <libproc.h>
#include <mach/mach.h>
#include <mach/task_info.h>
#include <mach/mach_time.h>
#include <signal.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/proc_info.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static pid_t retained_group=0,retained_child=0;
static void require(int ok,const char *label);
static void dispose_owned(void) {
    if (retained_group>0) {pid_t pid=retained_group;retained_group=0;kill(-pid,SIGKILL);while(waitpid(pid,NULL,0)<0&&errno==EINTR){}}
    if (retained_child>0) {pid_t pid=retained_child;retained_child=0;kill(pid,SIGKILL);while(waitpid(pid,NULL,0)<0&&errno==EINTR){}}
}

static uint64_t clock_ns(clockid_t id) {
    struct timespec t;
    require(clock_gettime(id,&t)==0&&t.tv_sec>=0&&t.tv_nsec>=0&&t.tv_nsec<1000000000,"known clock and units");
    __uint128_t raw=(__uint128_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;require(raw<=UINT64_MAX,"clock range");
    uint64_t now=(uint64_t)raw;static uint64_t last_monotonic=0;
    if(id==CLOCK_MONOTONIC){require(now>=last_monotonic,"nonreversed monotonic clock");last_monotonic=now;}
    return now;
}
static uint64_t ru_cpu(const struct rusage *r) {
    return ((uint64_t)r->ru_utime.tv_sec + (uint64_t)r->ru_stime.tv_sec) * 1000000000ULL
        + ((uint64_t)r->ru_utime.tv_usec + (uint64_t)r->ru_stime.tv_usec) * 1000ULL;
}
static uint64_t absolute_ns(uint64_t value) {
    mach_timebase_info_data_t timebase;require(mach_timebase_info(&timebase)==KERN_SUCCESS&&timebase.denom>0,"Mach timebase");
    __uint128_t ns=(__uint128_t)value*timebase.numer/timebase.denom;
    require(ns<=UINT64_MAX,"absolute CPU conversion bound");return (uint64_t)ns;
}
static void require(int ok, const char *label) {
    if (!ok) { fprintf(stderr, "probe refused: %s errno=%d\n", label, errno); exit(81); }
}
static void marker(const char *path) {
    int fd = open(path, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    require(fd >= 0 && write(fd, "executed\n", 9) == 9, "private marker");
    require(close(fd) == 0, "marker close");
}
struct image {
    pid_t pid, ppid, pgid;
    uint64_t birth, cpu, rss, waited, user, system, rusage_cpu, raw_cpu,raw_rusage_cpu,raw_waited,start_absolute,observed_at;
    uint32_t status,threads,runnable,flags;int32_t thread_state;
    uint64_t thread_user_raw,thread_system_raw;
    char path[PROC_PIDPATHINFO_MAXSIZE];
};
static int control_observe(pid_t pid,struct image *out) {
    struct proc_taskallinfo all;
    memset(out, 0, sizeof(*out));
    int result=proc_pidinfo(pid,PROC_PIDTASKALLINFO,0,&all,sizeof(all));
    if(result!=(int)sizeof(all)){fprintf(stderr,"observe_failed op=taskall pid=%d result=%d errno=%d\n",pid,result,errno);return 0;}
    out->observed_at=clock_ns(CLOCK_MONOTONIC);
    struct proc_bsdinfo b=all.pbsd;struct proc_taskinfo t=all.ptinfo;
    out->pid = pid; out->ppid = b.pbi_ppid; out->pgid = b.pbi_pgid;
    out->birth = (uint64_t)b.pbi_start_tvsec * 1000000ULL + b.pbi_start_tvusec;
    out->status = b.pbi_status;
    out->flags=b.pbi_flags;
    out->raw_cpu=t.pti_total_user+t.pti_total_system;
    out->cpu = absolute_ns(out->raw_cpu);
    out->user=absolute_ns(t.pti_total_user);out->system=absolute_ns(t.pti_total_system);
    out->threads=t.pti_threadnum;out->runnable=t.pti_numrunning;
    out->thread_state=-1;
    out->rss=t.pti_resident_size;
    return out->birth>0&&out->pgid>0;
}
static int observe(pid_t pid, struct image *out) {
    struct rusage_info_v1 usage;
    if(!control_observe(pid,out))return 0;
    int result=proc_pidpath(pid,out->path,sizeof(out->path));
    if(result<=0){fprintf(stderr,"observe_failed op=path pid=%d result=%d errno=%d\n",pid,result,errno);return 0;}
    result=proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage);
    if(result!=0){fprintf(stderr,"observe_failed op=rusage pid=%d result=%d errno=%d\n",pid,result,errno);return 0;}
    out->raw_rusage_cpu=usage.ri_user_time+usage.ri_system_time;out->rusage_cpu=absolute_ns(out->raw_rusage_cpu);
    if(out->threads==1) {
        uint64_t tids[2]={0};struct proc_threadinfo ti;
        int count=proc_pidinfo(pid,PROC_PIDLISTTHREADS,0,tids,sizeof(tids));
        if(count!=(int)sizeof(uint64_t)){fprintf(stderr,"observe_failed op=threads pid=%d result=%d errno=%d\n",pid,count,errno);return 0;}
        // PROC_PIDLISTTHREADS returns the thread handle consumed by
        // PROC_PIDTHREADINFO, not the distinct ID64 lookup namespace. The
        // initial main-thread handle may be zero before pthread bootstrap.
        count=proc_pidinfo(pid,PROC_PIDTHREADINFO,tids[0],&ti,sizeof(ti));
        if(count!=(int)sizeof(ti)){fprintf(stderr,"observe_failed op=threadinfo pid=%d result=%d errno=%d handle=%llu\n",pid,count,errno,(unsigned long long)tids[0]);return 0;}
        out->thread_state=ti.pth_run_state;out->thread_user_raw=ti.pth_user_time;out->thread_system_raw=ti.pth_system_time;
    }
    out->raw_waited=usage.ri_child_user_time+usage.ri_child_system_time;out->waited=absolute_ns(out->raw_waited);
    out->start_absolute=usage.ri_proc_start_abstime;
    return out->birth > 0 && out->pgid > 0;
}
static audit_token_t control_token(pid_t pid,uint64_t birth) {
    struct image before,after;require(control_observe(pid,&before)&&before.birth==birth,"control token pre-identity");
    mach_port_name_t task=MACH_PORT_NULL;require(task_name_for_pid(mach_task_self(),pid,&task)==KERN_SUCCESS,"control task-name right");
    audit_token_t token;mach_msg_type_number_t count=TASK_AUDIT_TOKEN_COUNT;
    kern_return_t result=task_info(task,TASK_AUDIT_TOKEN,(task_info_t)&token,&count);mach_port_deallocate(mach_task_self(),task);
    require(result==KERN_SUCCESS&&count==TASK_AUDIT_TOKEN_COUNT&&token.val[5]==(unsigned)pid&&token.val[7]>0&&control_observe(pid,&after)&&after.birth==birth,"control token actual birth");
    return token;
}
static struct image control_stop_ack(pid_t pid,uint64_t birth) {
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000ULL,cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct image current;
    for (;;) {
        require(control_observe(pid,&current)&&current.birth==birth&&current.threads==1&&current.runnable<=1,"bounded control stop identity");
        if(current.status==4&&current.runnable==0)return current;
        uint64_t used=clock_ns(CLOCK_THREAD_CPUTIME_ID)-cpu;
        if(clock_ns(CLOCK_MONOTONIC)>=until||used>1000000) {
            fprintf(stderr,"control_stop_unknown pid=%d status=%u runnable=%u threads=%u flags=%u cpu_ns=%llu user_ns=%llu system_ns=%llu controller_ns=%llu\n",pid,current.status,current.runnable,current.threads,current.flags,
                (unsigned long long)current.cpu,(unsigned long long)current.user,(unsigned long long)current.system,(unsigned long long)used);
            require(0,"bounded actual control stop acknowledgement");
        }
    }
}
static audit_token_t capture_token(pid_t pid,uint64_t birth) {
    struct image before,after;require(observe(pid,&before)&&before.birth==birth,"token pre-identity");
    mach_port_name_t task=MACH_PORT_NULL;require(task_name_for_pid(mach_task_self(),pid,&task)==KERN_SUCCESS,"own task-name right");
    audit_token_t token;mach_msg_type_number_t count=TASK_AUDIT_TOKEN_COUNT;
    kern_return_t result=task_info(task,TASK_AUDIT_TOKEN,(task_info_t)&token,&count);
    mach_port_deallocate(mach_task_self(),task);
    require(result==KERN_SUCCESS&&count==TASK_AUDIT_TOKEN_COUNT&&observe(pid,&after)&&after.birth==birth,
        "kernel audit token bound to exact birth");return token;
}
static struct image confirmed_stop(pid_t pid,uint64_t birth,const char *absent_marker,int initial) {
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000ULL;struct image current;
    uint64_t cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    for (;;) {
        require(observe(pid,&current)&&current.birth==birth,"stop confirmation identity");
        require(current.threads==1&&current.runnable<=1,"single-threaded helper profile");
        if(initial)require(current.user==0,"no initial target user CPU before admission");
        if(absent_marker)require(access(absent_marker,F_OK)==-1&&errno==ENOENT,"no target marker before admission");
        if(current.status==4&&current.runnable==0) {
            return current;
        }
        if(clock_ns(CLOCK_MONOTONIC)>=until) {
            fprintf(stderr,"stop_ack pid=%d ppid=%d pgid=%d flags=%u path=%s status=%u threads=%u runnable=%u thread_state=%d user_ns=%llu system_ns=%llu thread_user_raw=%llu thread_system_raw=%llu\n",
                pid,current.ppid,current.pgid,current.flags,current.path,current.status,current.threads,current.runnable,current.thread_state,(unsigned long long)current.user,(unsigned long long)current.system,
                (unsigned long long)current.thread_user_raw,(unsigned long long)current.thread_system_raw);
            audit_token_t token=capture_token(pid,birth);
            fprintf(stderr,"stop_ack_current_token pid=%u version=%u\n",token.val[5],token.val[7]);
            require(0,"bounded actual stop acknowledgement");
        }
        require(clock_ns(CLOCK_THREAD_CPUTIME_ID)-cpu_start<=1000000,"ordinary actual stop observation <=1ms CPU");
        if(initial)usleep(100);
    }
}
// Decision65489 fixed-slot arithmetic, independently exercised below against
// retained native identities. Required identity/census work is never post-seal.
struct window_ledger {
    uint64_t operation,sequence,t0,t_close,cpu_start,cpu_close,end[2],actual_cpu,pipe_actual,pipe_charged;
    uint64_t windows,controller,charge;
    unsigned slots,unknown_mask,ack_mask,identity_mask;
    int roster_closed,source_complete,exec_pending,required_after_close,bad_clock,retired_open;
};
enum ledger_result { LEDGER_OK,LEDGER_CLOCK,LEDGER_IDENTITY,LEDGER_ROSTER,LEDGER_PIPE,LEDGER_COVERAGE,LEDGER_LIMIT };
static enum ledger_result window_charge(struct window_ledger *g) {
    g->windows=0;g->controller=0;g->charge=0;
    if(g->bad_clock||g->t0==0||g->t_close<g->t0||g->cpu_close<g->cpu_start)return LEDGER_CLOCK;
    if(g->operation==0||g->sequence==0||g->slots==0||g->slots>2||g->ack_mask!=((1u<<g->slots)-1)
        ||g->identity_mask!=g->ack_mask||g->exec_pending||g->retired_open)return LEDGER_IDENTITY;
    if(!g->roster_closed||!g->source_complete||g->required_after_close)return LEDGER_ROSTER;
    if(g->pipe_charged!=g->pipe_actual)return LEDGER_PIPE;
    for(unsigned i=0;i<g->slots;i++) {
        if(g->end[i]<g->t0||g->end[i]>g->t_close)return LEDGER_CLOCK;
        if((g->unknown_mask&(1u<<i))&&g->end[i]!=g->t_close)return LEDGER_ROSTER;
        uint64_t n=g->end[i]-g->t0;if(g->windows>UINT64_MAX-n)return LEDGER_CLOCK;g->windows+=n;
    }
    uint64_t raw=g->cpu_close-g->cpu_start;if(raw>UINT64_MAX-g->pipe_charged)return LEDGER_CLOCK;
    g->controller=raw+g->pipe_charged;
    uint64_t helper=g->windows>g->actual_cpu?g->windows:g->actual_cpu;
    if(helper>UINT64_MAX-g->controller)return LEDGER_CLOCK;g->charge=helper+g->controller;
    if(g->actual_cpu>g->windows)return LEDGER_COVERAGE;
    return g->charge<=1000000?LEDGER_OK:LEDGER_LIMIT;
}
static int additive_debt(uint64_t prior,uint64_t now,uint64_t cost,uint64_t *result) {
    uint64_t n=cost>1000000?cost:1000000;if(n>UINT64_MAX/9)return 0;
    uint64_t base=prior>now?prior:now;n*=9;if(base>UINT64_MAX-n)return 0;*result=base+n;return 1;
}
static void pay_window_debt(uint64_t *deadline,uint64_t cost) {
    require(additive_debt(*deadline,clock_ns(CLOCK_MONOTONIC),cost,deadline),"additive ordinary debt clock");
    for (;;) {uint64_t now=clock_ns(CLOCK_MONOTONIC);if(now>=*deadline)return;uint64_t left=*deadline-now;
        usleep((useconds_t)((left>1000000?1000000:left)+999)/1000);}
}
static unsigned ledger_controls(void) {
    struct window_ledger good={.operation=1,.sequence=1,.t0=100,.t_close=200,.cpu_start=10,.cpu_close=30,
        .end={130,160},.actual_cpu=80,.slots=2,.ack_mask=3,.identity_mask=3,.roster_closed=1,.source_complete=1};
    require(window_charge(&good)==LEDGER_OK&&good.windows==90&&good.controller==20&&good.charge==110,"unequal slot window golden");
    struct window_ledger bad;unsigned count=0;
#define REJECT(label,edit,expected) do {bad=good;edit;require(window_charge(&bad)==expected,label);count++;} while(0)
    REJECT("backdated stop endpoint refuses",bad.end[0]=99,LEDGER_CLOCK);
    REJECT("unknown birth cannot inherit endpoint",bad.unknown_mask=2,LEDGER_ROSTER);
    REJECT("old exec endpoint cannot close new task",bad.exec_pending=1,LEDGER_IDENTITY);
    REJECT("blocked or zero-delta is not stop ack",bad.ack_mask=1,LEDGER_IDENTITY);
    REJECT("missing current audit identity refuses",bad.identity_mask=1,LEDGER_IDENTITY);
    REJECT("one absent census is not closed roster",bad.roster_closed=0,LEDGER_ROSTER);
    REJECT("unaccounted transient or unsupported source basis refuses",bad.source_complete=0,LEDGER_ROSTER);
    REJECT("required controller cannot move after fake seal",bad.required_after_close=1,LEDGER_ROSTER);
    REJECT("retired open slot remains coverage",bad.retired_open=1,LEDGER_IDENTITY);
    REJECT("unexplained actual CPU above W refuses",bad.actual_cpu=91,LEDGER_COVERAGE);
    REJECT("omitted pipe CPU refuses",bad.pipe_actual=1,LEDGER_PIPE);
    REJECT("double pipe CPU refuses",bad.pipe_actual=1;bad.pipe_charged=2,LEDGER_PIPE);
    REJECT("unknown or reversed clock refuses",bad.bad_clock=1,LEDGER_CLOCK);
    REJECT("original sequence cannot be absent",bad.sequence=0,LEDGER_IDENTITY);
    REJECT("inclusive ordinary1ms gate unchanged",bad.cpu_close=1000010,LEDGER_LIMIT);
#undef REJECT
    uint64_t debt=0;require(additive_debt(27000001,1,1,&debt)&&debt==36000001,"debt is additive and minimum9ms");count++;
    require(debt!=9000001,"debt reset mutant rejected");count++;
    // Descendant roll-up advances an already charged lifetime floor; adding
    // direct+rolled totals would count the same descendant a second time.
    uint64_t prior=7,own=7,rolled=9;require((own>rolled?own:rolled)-prior==2&&own+rolled-prior!=2,"double roll-up mutant rejected");count++;
    return count;
}
static struct image acknowledged_member(pid_t pid,const struct image *original,const audit_token_t *expected,const char *path) {
    (void)control_stop_ack(pid,original->birth);
    struct image current;require(observe(pid,&current)&&current.pid==original->pid&&current.ppid==original->ppid&&current.pgid==original->pgid
        &&current.birth==original->birth&&current.start_absolute==original->start_absolute&&current.threads==1&&current.status==4&&current.runnable==0
        &&strcmp(current.path,path)==0,"identity/image/start-bound stopped member");
    audit_token_t actual=control_token(pid,original->birth);require(memcmp(&actual,expected,sizeof(actual))==0,"original current audit incarnation");
    struct image final;require(control_observe(pid,&final)&&final.birth==current.birth&&final.ppid==current.ppid&&final.pgid==current.pgid
        &&final.status==4&&final.threads==1&&final.runnable==0,"stopped continuity after mandatory identity calls");
    current.observed_at=clock_ns(CLOCK_MONOTONIC);return current;
}
static void complete_known_roster(pid_t leader,pid_t child,const struct image *a,const struct image *b,
    const audit_token_t *ta,const audit_token_t *tb,const char *path,struct image *end_a,struct image *end_b) {
    int ids[3]={0};int n=proc_listchildpids(leader,ids,sizeof(ids));require(n==1&&ids[0]==child,"stopped creator exact child edge");
    memset(ids,0,sizeof(ids));require(proc_listchildpids(child,ids,sizeof(ids))==0,"stopped no-fork child closed edge");
    memset(ids,0,sizeof(ids));n=proc_listpgrppids(leader,ids,sizeof(ids));
    require(n==2&&((ids[0]==leader&&ids[1]==child)||(ids[0]==child&&ids[1]==leader)),"complete same-grant exact group roster");
    *end_a=acknowledged_member(leader,a,ta,path);*end_b=acknowledged_member(child,b,tb,path);
}
struct native_window_evidence {struct window_ledger grant;uint64_t debt,late_cpu,post_cpu,start[2];uint32_t version[2];unsigned controls;};
static struct native_window_evidence native_member_windows(pid_t leader,pid_t child,const char *path) {
    struct native_window_evidence evidence;memset(&evidence,0,sizeof(evidence));evidence.controls=ledger_controls();
    struct image a,b;require(observe(leader,&a)&&observe(child,&b),"retained native member baselines");
    audit_token_t ta=control_token(leader,a.birth),tb=control_token(child,b.birth);
    evidence.start[0]=a.start_absolute;evidence.start[1]=b.start_absolute;evidence.version[0]=ta.val[7];evidence.version[1]=tb.val[7];
    require(a.status==4&&a.runnable==0&&b.status==4&&b.runnable==0,"closed attachment before member-window grant");
    uint64_t before=a.rusage_cpu+b.rusage_cpu;struct window_ledger *g=&evidence.grant;
    g->operation=a.start_absolute;g->sequence=1;g->slots=2;g->source_complete=1;
    g->cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);g->t0=clock_ns(CLOCK_MONOTONIC);
    require(kill(-leader,SIGCONT)==0,"member-window original CONT delivery");
    while(clock_ns(CLOCK_MONOTONIC)-g->t0<50000){}
    require(kill(-leader,SIGSTOP)==0,"member-window original STOP request");
    struct image end_a=acknowledged_member(leader,&a,&ta,path);g->end[0]=clock_ns(CLOCK_MONOTONIC);
    uint64_t delayed=clock_ns(CLOCK_MONOTONIC);while(clock_ns(CLOCK_MONOTONIC)-delayed<50000){}
    struct image end_b=acknowledged_member(child,&b,&tb,path);g->end[1]=clock_ns(CLOCK_MONOTONIC);
    g->ack_mask=3;g->identity_mask=3;
    complete_known_roster(leader,child,&a,&b,&ta,&tb,path,&end_a,&end_b);g->roster_closed=1;
    g->t_close=clock_ns(CLOCK_MONOTONIC);g->cpu_close=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(end_a.rusage_cpu+end_b.rusage_cpu>=before,"monotonic observed active helper CPU");
    g->actual_cpu=end_a.rusage_cpu+end_b.rusage_cpu-before;
    enum ledger_result verdict=window_charge(g);
    fprintf(stderr,"member_window op=%llu seq=%llu t0=%llu endpoint0=%llu endpoint1=%llu tclose=%llu W=%llu A=%llu C=%llu Q=%llu closed=%d verdict=%d\n",
        (unsigned long long)g->operation,(unsigned long long)g->sequence,(unsigned long long)g->t0,(unsigned long long)g->end[0],(unsigned long long)g->end[1],
        (unsigned long long)g->t_close,(unsigned long long)g->windows,(unsigned long long)g->actual_cpu,(unsigned long long)g->controller,(unsigned long long)g->charge,g->roster_closed,verdict);
    fprintf(stderr,"member_identity slot=0 helper_sequence=1 pid=%d ppid=%d pgid=%d birth=%llu start_abstime=%llu audit_version=%u image_phase=canonical_probe_c\n",a.pid,a.ppid,a.pgid,(unsigned long long)a.birth,(unsigned long long)a.start_absolute,ta.val[7]);
    fprintf(stderr,"member_identity slot=1 helper_sequence=2 pid=%d ppid=%d pgid=%d birth=%llu start_abstime=%llu audit_version=%u image_phase=canonical_probe_c\n",b.pid,b.ppid,b.pgid,(unsigned long long)b.birth,(unsigned long long)b.start_absolute,tb.val[7]);
    pay_window_debt(&evidence.debt,g->charge);
    require(verdict==LEDGER_OK&&g->end[1]>g->end[0],"real unequal-stop complete member-window ordinary grant");
    uint64_t post=g->cpu_close;
    struct image held_a=acknowledged_member(leader,&a,&ta,path),held_b=acknowledged_member(child,&b,&tb,path);
    complete_known_roster(leader,child,&a,&b,&ta,&tb,path,&held_a,&held_b);
    require(held_a.user==end_a.user&&held_b.user==end_b.user&&held_a.cpu>=end_a.cpu&&held_b.cpu>=end_b.cpu,"stable user-work and explained late-kernel deltas");
    evidence.late_cpu=held_a.cpu-end_a.cpu+held_b.cpu-end_b.cpu;evidence.post_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID)-post;
    pay_window_debt(&evidence.debt,evidence.post_cpu+evidence.late_cpu);
    require(evidence.post_cpu<=1000000&&evidence.late_cpu<=1000000-evidence.post_cpu,"inclusive post-closure ordinary1ms gate");
    return evidence;
}
static pid_t suspended(const char *program, char *const argv[], char *const envp[], int own_group,
    uint64_t *cpu, uint64_t *wall) {
    posix_spawnattr_t attr; posix_spawn_file_actions_t actions;
    require(posix_spawnattr_init(&attr) == 0, "spawn attr init");
    short flags = POSIX_SPAWN_START_SUSPENDED | POSIX_SPAWN_CLOEXEC_DEFAULT;
    if (own_group) flags |= POSIX_SPAWN_SETPGROUP;
    require(posix_spawnattr_setflags(&attr, flags) == 0, "exact suspended flags");
    if (own_group) require(posix_spawnattr_setpgroup(&attr, 0) == 0, "set own group");
    require(posix_spawn_file_actions_init(&actions) == 0, "spawn actions init");
    require(posix_spawn_file_actions_addopen(&actions, 0, "/dev/null", O_RDONLY, 0) == 0,
        "private null stdin");
    require(posix_spawn_file_actions_adddup2(&actions, 1, 1) == 0
        && posix_spawn_file_actions_adddup2(&actions, 2, 2) == 0, "retained output fds");
    pid_t pid = -1;
    uint64_t c = clock_ns(CLOCK_THREAD_CPUTIME_ID), w = clock_ns(CLOCK_MONOTONIC);
    int result = posix_spawn(&pid, program, &actions, &attr, argv, envp);
    uint64_t wc = clock_ns(CLOCK_MONOTONIC), cc = clock_ns(CLOCK_THREAD_CPUTIME_ID);
    *cpu = cc - c; *wall = wc - w;
    // Retain any positive returned identity before even an error verdict can
    // reject the native effect; failed application never implies no effect.
    if(pid>0){if (own_group) retained_group=pid;else retained_child=pid;}
    if (result != 0 || pid <= 0) { errno = result; require(0, "suspended creation"); }
    posix_spawn_file_actions_destroy(&actions); posix_spawnattr_destroy(&attr);
    if (*cpu > 2000000 || *wall > 25000000) {
        dispose_owned();
        require(0, "native creation 2ms CPU/25ms wall");
    }
    return pid;
}
static int exited(pid_t pid) {
    siginfo_t info; memset(&info, 0, sizeof(info));
    require(waitid(P_PID, pid, &info, WEXITED | WNOHANG | WNOWAIT) == 0,
        "leader nonreaping observation");
    return info.si_pid == pid && (info.si_code == CLD_EXITED
        || info.si_code == CLD_KILLED || info.si_code == CLD_DUMPED);
}
static void joined_group(pid_t pid, struct rusage *usage) {
    // Signal while the still-unreaped leader pins the exact group generation.
    kill(-pid, SIGKILL);
    int status = 0;
    pid_t joined=wait4(pid,&status,0,usage);if(joined==pid)retained_group=0;
    require(joined == pid && WIFEXITED(status) && WEXITSTATUS(status) == 0,"actual leader wait4");
    uint64_t until = clock_ns(CLOCK_MONOTONIC) + 1000000000ULL;
    while (kill(-pid, 0) == 0 || errno != ESRCH) {
        require(clock_ns(CLOCK_MONOTONIC) < until, "actual group settlement"); usleep(1000);
    }
    require(waitpid(pid, &status, WNOHANG) == -1 && errno == ECHILD, "leader already reaped");
}
static void cpu_child(const char *path) {
    marker(path);
    uint64_t until = clock_ns(CLOCK_PROCESS_CPUTIME_ID) + 12000000ULL;
    volatile uint64_t v = 1;
    while (clock_ns(CLOCK_PROCESS_CPUTIME_ID) < until) for (int i = 0; i < 256; i++) v = v * 33 + i;
    _exit(v == 0 ? 1 : 0);
}
static int token_capability(const char *self,const char *root) {
    char first[4096],second[4096];snprintf(first,sizeof(first),"%s/audit-first-marker",root);
    snprintf(second,sizeof(second),"%s/audit-replacement-marker",root);
    char *args[]={(char *)self,"cpu-child",first,NULL};char *env[]={"PATH=/usr/bin:/bin","LANG=C",NULL};
    uint64_t cpu=0,wall=0;pid_t old=suspended(self,args,env,1,&cpu,&wall);struct image a,b;
    require(observe(old,&a)&&a.status==4,"audit first suspended identity");a=confirmed_stop(old,a.birth,first,1);audit_token_t old_token=capture_token(old,a.birth);
    require(proc_signal_with_audittoken(&old_token,SIGKILL)==0,"fenced first kill");
    struct rusage usage;int status=0;pid_t joined=wait4(old,&status,0,&usage);if(joined==old)retained_group=0;
    require(joined==old&&WIFSIGNALED(status)&&WTERMSIG(status)==SIGKILL,"fenced target actual reap");
    require(access(first,F_OK)==-1&&errno==ENOENT,"audit target never ran");
    args[2]=second;pid_t replacement=suspended(self,args,env,1,&cpu,&wall);
    require(observe(replacement,&b)&&b.status==4,"audit replacement suspended identity");
    b=confirmed_stop(replacement,b.birth,second,1);
    errno=0;int stale=proc_signal_with_audittoken(&old_token,SIGKILL);int stale_error=stale>0?stale:errno;
    fprintf(stderr,"stale_audit result=%d errno=%d old_pid=%d replacement=%d old_pidversion=%u\n",stale,stale_error,old,replacement,old_token.val[7]);
    require(stale!=0&&stale_error==ESRCH,"retired audit token cannot signal a replacement");
    struct image after;require(observe(replacement,&after)&&after.birth==b.birth&&after.cpu==b.cpu&&after.status==4,
        "replacement unaffected by retired token");
    audit_token_t current=capture_token(replacement,b.birth);
    require(old_token.val[7]!=current.val[7],"distinct observed kernel PID versions");
    audit_token_t reused=old_token;reused.val[5]=(unsigned int)replacement;
    errno=0;int stale_version=proc_signal_with_audittoken(&reused,SIGKILL);int version_error=stale_version>0?stale_version:errno;
    fprintf(stderr,"stale_pid_version result=%d errno=%d old_version=%u replacement_pid=%d replacement_version=%u\n",stale_version,version_error,old_token.val[7],replacement,current.val[7]);
    require(stale_version!=0&&(version_error==ESRCH||version_error==EINVAL),"stale version cannot signal actual replacement PID");
    require(observe(replacement,&after)&&after.birth==b.birth&&after.cpu==b.cpu&&after.status==4,
        "replacement survives old-token/new-PID causal control");
    require(proc_signal_with_audittoken(&current,SIGKILL)==0,"fenced replacement cleanup");
    joined=wait4(replacement,&status,0,&usage);if(joined==replacement)retained_group=0;
    require(joined==replacement&&WIFSIGNALED(status)&&WTERMSIG(status)==SIGKILL,"replacement actual reap");
    require(access(second,F_OK)==-1&&errno==ENOENT,"replacement never ran");
    return version_error;
}
static void leader(const char *self, const char *first, const char *child_marker, const char *report, const char *sentinel) {
    errno=0;require(fcntl(atoi(sentinel),F_GETFD)==-1&&errno==EBADF,"CLOEXEC_DEFAULT closes unlisted inherited fd");
    marker(first);
    char *args[] = {(char *)self, "cpu-child", (char *)child_marker, NULL};
    char *env[] = {"PATH=/usr/bin:/bin", "LANG=C", NULL};
    uint64_t cpu = 0, wall = 0;
    pid_t child = suspended(self, args, env, 0, &cpu, &wall);
    FILE *out = fopen(report, "wx"); require(out != NULL, "child report create");
    fprintf(out, "%d\n", child); require(fflush(out) == 0, "child identity flush");
    raise(SIGSTOP); // Parent observes both identities while both are stopped.
    struct rusage usage; int status;
    pid_t joined=wait4(child,&status,0,&usage);if(joined==child)retained_child=0;
    require(joined == child && WIFEXITED(status) && WEXITSTATUS(status) == 0, "waited descendant");
    fprintf(out, "%llu\n", (unsigned long long)ru_cpu(&usage));
    require(fclose(out) == 0, "child report close"); _exit(0);
}
static int native_capability(const char *self, const char *root) {
    char first[4096], second[4096], report[4096];
    snprintf(first, sizeof(first), "%s/leader-marker", root);
    snprintf(second, sizeof(second), "%s/child-marker", root);
    snprintf(report, sizeof(report), "%s/child-report", root);
    char sentinel_path[4096],sentinel_arg[32];snprintf(sentinel_path,sizeof(sentinel_path),"%s/fd-sentinel",root);
    int sentinel=open(sentinel_path,O_RDWR|O_CREAT|O_EXCL,0600);require(sentinel>=3,"private inherited-fd sentinel");
    require(fcntl(sentinel,F_SETFD,0)==0,"sentinel deliberately inheritable");snprintf(sentinel_arg,sizeof(sentinel_arg),"%d",sentinel);
    char *args[] = {(char *)self, "leader", first, second, report, sentinel_arg, NULL};
    char *env[] = {"PATH=/usr/bin:/bin", "LANG=C", NULL};
    uint64_t create_cpu = 0, create_wall = 0;
    pid_t pid = suspended(self, args, env, 1, &create_cpu, &create_wall);
    close(sentinel);
    struct image before, after, child;
    require(observe(pid, &before), "created identity");
    require(before.ppid == getpid() && before.pgid == pid && before.status == 4
        && strcmp(before.path, self) == 0 && access(first, F_OK) == -1 && errno == ENOENT,
        "created suspended before first marker");
    uint64_t initial_cpu=before.cpu,confirm_at=clock_ns(CLOCK_MONOTONIC);
    before=confirmed_stop(pid,before.birth,first,1);
    uint64_t confirm_wall=clock_ns(CLOCK_MONOTONIC)-confirm_at;
    uint64_t bootstrap_cpu=before.cpu-initial_cpu;
    fprintf(stderr,"native_bootstrap initial_cpu_ns=%llu confirmed_cpu_ns=%llu reconciliation_cpu_ns=%llu stop_confirm_wall_ns=%llu threads=%u runnable=%u\n",
        (unsigned long long)initial_cpu,(unsigned long long)before.cpu,(unsigned long long)bootstrap_cpu,(unsigned long long)confirm_wall,before.threads,before.runnable);
    usleep(20000);
    int held_observed=observe(pid,&after);
    fprintf(stderr,"held_before pid=%d birth=%llu status=%u threads=%u runnable=%u user=%llu system=%llu cpu=%llu rusage_cpu=%llu\n",pid,(unsigned long long)before.birth,before.status,before.threads,before.runnable,(unsigned long long)before.user,(unsigned long long)before.system,(unsigned long long)before.cpu,(unsigned long long)before.rusage_cpu);
    fprintf(stderr,"held_after observed=%d pid=%d birth=%llu status=%u threads=%u runnable=%u user=%llu system=%llu cpu=%llu rusage_cpu=%llu\n",held_observed,pid,(unsigned long long)after.birth,after.status,after.threads,after.runnable,(unsigned long long)after.user,(unsigned long long)after.system,(unsigned long long)after.cpu,(unsigned long long)after.rusage_cpu);
    require(held_observed && after.birth == before.birth && after.user == before.user&&after.cpu>=before.cpu&&after.status==4&&after.threads==1&&after.runnable==0
        && access(first, F_OK) == -1 && errno == ENOENT, "no pre-admission target user work");
    uint64_t held_kernel_cpu=after.cpu-before.cpu;require(held_kernel_cpu<=1000000,"initial stopped kernel reconciliation bound");
    if(held_kernel_cpu)usleep((useconds_t)((9*held_kernel_cpu+999)/1000));
    require(kill(-pid, SIGCONT) == 0, "first resume");
    uint64_t until = clock_ns(CLOCK_MONOTONIC) + 2000000000ULL;
    pid_t child_pid = 0;
    while (!child_pid) {
        FILE *f = fopen(report, "r"); if (f) { fscanf(f, "%d", &child_pid); fclose(f); }
        require(clock_ns(CLOCK_MONOTONIC) < until, "child identity readiness"); usleep(100);
    }
    do {
        if (observe(pid,&after)&&after.status==4&&observe(child_pid,&child)&&child.status==4) break;
        require(clock_ns(CLOCK_MONOTONIC)<until,"two-member stopped visibility");usleep(100);
    } while (1);
    require(observe(pid, &after) && after.status == 4 && after.birth == before.birth
        && observe(child_pid, &child) && child.status == 4 && child.ppid == pid
        && child.pgid == pid && strcmp(child.path, self) == 0 && access(second, F_OK) == -1,
        "two suspended exact group members");
    int members[3] = {0};
    int child_count=proc_listchildpids(pid,members,sizeof(members));
    fprintf(stderr,"exact_children return=%d leader=%d expected_child=%d ids=%d,%d,%d\n",child_count,pid,child_pid,members[0],members[1],members[2]);
    require(child_count == 1
        && members[0] == child_pid, "bounded exact child relationship");
    memset(members,0,sizeof(members));int group_count=proc_listpgrppids(pid,members,sizeof(members));
    fprintf(stderr,"exact_group return=%d leader=%d child=%d ids=%d,%d,%d\n",group_count,pid,child_pid,members[0],members[1],members[2]);
    require(group_count==2&&((members[0]==pid&&members[1]==child_pid)||(members[1]==pid&&members[0]==child_pid)),"bounded exact group relationship");
    uint64_t peak_rss = after.rss + child.rss;
    require(kill(-pid, SIGCONT) == 0, "group resume");
    until=clock_ns(CLOCK_MONOTONIC)+2000000000ULL;
    while(access(second,F_OK)!=0) {
        require(clock_ns(CLOCK_MONOTONIC)<until,"actual running child marker readiness");usleep(50);
    }
    audit_token_t active_token=capture_token(child_pid,child.birth);
    require(proc_signal_with_audittoken(&active_token,SIGSTOP)==0,"audit STOP receipt on actual running own child");
    require(kill(-pid,SIGSTOP)==0,"one exact known-roster group STOP");
    after=confirmed_stop(pid,before.birth,NULL,0);child=confirmed_stop(child_pid,child.birth,NULL,0);
    uint64_t stopped_user=child.user;usleep(20000);struct image held_child;
    require(observe(child_pid,&held_child)&&held_child.birth==child.birth&&held_child.status==4&&held_child.threads==1&&held_child.runnable==0&&held_child.user==stopped_user,
        "actual running own child STOP application and held user quiescence");
    struct native_window_evidence windows=native_member_windows(pid,child_pid,self);
    require(kill(-pid,SIGCONT)==0,"resume after actual stop application witness");
    uint64_t child_cpu = 0,child_raw_cpu=0;
    until = clock_ns(CLOCK_MONOTONIC) + 2000000000ULL;
    while (!exited(pid)) {
        struct image c, p;
        if (observe(child_pid, &c)) { if (c.cpu > child_cpu) child_cpu = c.cpu;if(c.raw_cpu>child_raw_cpu)child_raw_cpu=c.raw_cpu; }
        if (observe(pid, &p) && observe(child_pid, &c) && p.rss + c.rss > peak_rss) peak_rss = p.rss + c.rss;
        require(clock_ns(CLOCK_MONOTONIC) < until, "native completion"); usleep(100);
    }
    struct rusage_info_v1 raw_leader_usage;int leader_usage_result=proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&raw_leader_usage);
    mach_timebase_info_data_t timebase;require(mach_timebase_info(&timebase)==KERN_SUCCESS,"Mach timebase sample");
    fprintf(stderr,"cpu_units timebase_numer=%u timebase_denom=%u sampled_child_raw=%llu sampled_child_converted_ns=%llu leader_rusage_result=%d leader_child_raw=%llu leader_child_converted_ns=%llu\n",timebase.numer,timebase.denom,(unsigned long long)child_raw_cpu,(unsigned long long)child_cpu,leader_usage_result,
        (unsigned long long)(leader_usage_result==0?raw_leader_usage.ri_child_user_time+raw_leader_usage.ri_child_system_time:0),
        (unsigned long long)(leader_usage_result==0?absolute_ns(raw_leader_usage.ri_child_user_time+raw_leader_usage.ri_child_system_time):0));
    struct rusage usage; joined_group(pid, &usage);
    FILE *f = fopen(report, "r"); uint64_t waited_cpu = 0; pid_t reported = 0;
    require(f && fscanf(f, "%d\n%llu", &reported, (unsigned long long *)&waited_cpu) == 2
        && reported == child_pid, "actual descendant wait4 report"); fclose(f);
    require(waited_cpu >= 12000000 && child_cpu > 0 && ru_cpu(&usage) >= waited_cpu,
        "leader wait4 descendant CPU rollup");
    int stale_error=token_capability(self,root);
    printf("{\"created_suspended\":true,\"cloexec_default\":true,\"audit_token_signals\":true,\"running_child_stop_applied\":true,\"stale_pid_version_control\":true,\"mach_timebase_numer\":%u,\"mach_timebase_denom\":%u,\"sampled_child_cpu_raw\":%llu,\"initial_native_cpu_ns\":%llu,\"confirmed_native_cpu_ns\":%llu,\"native_bootstrap_reconciliation_ns\":%llu,\"stop_confirm_wall_ns\":%llu,\"held_user_cpu_delta_ns\":0,\"held_kernel_cpu_ns\":%llu,\"held_kernel_paid_debt_ns\":%llu,\"threads\":1,\"confirmed_runnable\":0,\"stale_audit_error\":%d,\"leader\":%d,\"child\":%d,\"pgid\":%d,\"leader_birth\":%llu,\"child_birth\":%llu,\"members\":2,\"sampled_child_cpu_ns\":%llu,\"waited_child_cpu_ns\":%llu,\"leader_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true,\"member_windows\":{\"basis\":\"canonical_private_c_program_only\",\"operation\":%llu,\"sequence\":%llu,\"t0_ns\":%llu,\"endpoint_ns\":[%llu,%llu],\"tclose_ns\":%llu,\"W_ns\":%llu,\"A_ns\":%llu,\"C_ns\":%llu,\"Q_ns\":%llu,\"roster_closed\":true,\"late_kernel_ns\":%llu,\"post_cpu_ns\":%llu,\"debt_deadline_ns\":%llu,\"causal_controls\":%u}}\n",
        timebase.numer,timebase.denom,(unsigned long long)child_raw_cpu,(unsigned long long)initial_cpu,(unsigned long long)before.cpu,(unsigned long long)bootstrap_cpu,(unsigned long long)confirm_wall,
        (unsigned long long)held_kernel_cpu,(unsigned long long)(9*held_kernel_cpu),stale_error,pid, child_pid, pid, (unsigned long long)before.birth, (unsigned long long)child.birth,
        (unsigned long long)child_cpu, (unsigned long long)waited_cpu, (unsigned long long)ru_cpu(&usage),
        (unsigned long long)peak_rss, (unsigned long long)create_cpu, (unsigned long long)create_wall,
        (unsigned long long)windows.grant.operation,(unsigned long long)windows.grant.sequence,(unsigned long long)windows.grant.t0,
        (unsigned long long)windows.grant.end[0],(unsigned long long)windows.grant.end[1],(unsigned long long)windows.grant.t_close,
        (unsigned long long)windows.grant.windows,(unsigned long long)windows.grant.actual_cpu,(unsigned long long)windows.grant.controller,
        (unsigned long long)windows.grant.charge,(unsigned long long)windows.late_cpu,(unsigned long long)windows.post_cpu,(unsigned long long)windows.debt,windows.controls);
    return 0;
}
static int git_capability(char **argv) {
    (void)argv;
    // Decision65489 and Oracle01M6PXGPD8714R5DQCV7VEW37E: the exact current
    // Git/GPG pair has no approved whole-command process/thread lifetime basis.
    // Censuses and Trace2 alone cannot close its reserved unknown slot. Do not
    // execute another CONT, infer a true flag, or mint an availability receipt.
    printf("{\"profile\":\"read-helper-native/1\",\"availability\":\"verification_unproven\",\"whole_command_lifetime_coverage\":false,\"basis\":\"missing_supported_exact_tool_os_lifetime_capability\"}\n");
    fprintf(stderr,"probe unavailable: verification_unproven missing supported exact Git/GPG whole-command process/thread lifetime basis\n");
    return 81;
}
int main(int argc,char **argv) {
    require(atexit(dispose_owned)==0,"owned cleanup registration");
    require(argc>=2,"mode required");
    if (!strcmp(argv[1],"cpu-child")) {require(argc==3,"child argc");cpu_child(argv[2]);}
    if (!strcmp(argv[1],"leader")) {require(argc==6,"leader argc");leader(argv[0],argv[2],argv[3],argv[4],argv[5]);}
    if (!strcmp(argv[1],"native")) {require(argc==3,"native argc");return native_capability(argv[0],argv[2]);}
    if (!strcmp(argv[1],"ledger-controls")) {require(argc==2,"ledger argc");printf("{\"profile\":\"read-helper-native/1\",\"causal_controls\":%u,\"tool_availability\":\"verification_unproven\"}\n",ledger_controls());return 0;}
    if (!strcmp(argv[1],"git")) {require(argc==10,"Git argc");return git_capability(argv);}
    require(0,"unknown mode");return 81;
}
