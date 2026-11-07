// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Private Darwin capability witness; neither a product helper nor an authority source.
#define _DARWIN_C_SOURCE 1
#include <CommonCrypto/CommonDigest.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <libproc.h>
#include <mach/mach.h>
#include <mach/task_info.h>
#include <mach/mach_time.h>
#include <mach/mach_vm.h>
#include <stdbool.h>
#include <stddef.h>
#include <mach-o/dyld_images.h>
#include <mach-o/loader.h>
#include <signal.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/proc_info.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/sysctl.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static pid_t retained_group=0,retained_child=0;
static void (*dispose_private_engine)(void)=NULL;
static void require(int ok,const char *label);
static void dispose_owned(void) {
    // Private ownership has its own bounded, persisted quarantine path. Never
    // follow that path with the generic blocking numeric-PGID fallback.
    if(dispose_private_engine){dispose_private_engine();return;}
    if (retained_group>0) {pid_t pid=retained_group;retained_group=0;kill(-pid,SIGKILL);while(waitpid(pid,NULL,0)<0&&errno==EINTR){}}
    if (retained_child>0) {pid_t pid=retained_child;retained_child=0;kill(pid,SIGKILL);while(waitpid(pid,NULL,0)<0&&errno==EINTR){}}
}

static int known_clock_ns(clockid_t id,uint64_t *out) {
    struct timespec t;
    if(clock_gettime(id,&t)!=0||t.tv_sec<0||t.tv_nsec<0||t.tv_nsec>=1000000000)return 0;
    __uint128_t raw=(__uint128_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;if(raw>UINT64_MAX)return 0;
    uint64_t now=(uint64_t)raw;static uint64_t last_monotonic=0;
    if(id==CLOCK_MONOTONIC){if(now<last_monotonic)return 0;last_monotonic=now;}
    *out=now;return 1;
}
static uint64_t clock_ns(clockid_t id) {
    uint64_t now=0;require(known_clock_ns(id,&now),"known nonreversed clock and units");return now;
}
static uint64_t ru_cpu(const struct rusage *r) {
    return ((uint64_t)r->ru_utime.tv_sec + (uint64_t)r->ru_stime.tv_sec) * 1000000000ULL
        + ((uint64_t)r->ru_utime.tv_usec + (uint64_t)r->ru_stime.tv_usec) * 1000ULL;
}
static int known_absolute_ns(uint64_t value,uint64_t *out) {
    mach_timebase_info_data_t timebase;if(mach_timebase_info(&timebase)!=KERN_SUCCESS||!timebase.numer||!timebase.denom)return 0;
    __uint128_t ns=(__uint128_t)value*timebase.numer/timebase.denom;
    if(ns>UINT64_MAX)return 0;*out=(uint64_t)ns;return 1;
}
static uint64_t absolute_ns(uint64_t value) {
    uint64_t out=0;require(known_absolute_ns(value,&out),"known Mach CPU conversion bound");return out;
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
    uid_t uid;int32_t nice;
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
    if(!known_clock_ns(CLOCK_MONOTONIC,&out->observed_at))return 0;
    struct proc_bsdinfo b=all.pbsd;struct proc_taskinfo t=all.ptinfo;
    out->pid = pid; out->ppid = b.pbi_ppid; out->pgid = b.pbi_pgid;
    out->uid=b.pbi_uid;out->nice=b.pbi_nice;
    out->birth = (uint64_t)b.pbi_start_tvsec * 1000000ULL + b.pbi_start_tvusec;
    out->status = b.pbi_status;
    out->flags=b.pbi_flags;
    if(UINT64_MAX-t.pti_total_user<t.pti_total_system)return 0;
    out->raw_cpu=t.pti_total_user+t.pti_total_system;
    if(!known_absolute_ns(out->raw_cpu,&out->cpu)||!known_absolute_ns(t.pti_total_user,&out->user)||!known_absolute_ns(t.pti_total_system,&out->system))return 0;
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
static int known_control_token(pid_t pid,uint64_t birth,audit_token_t *out) {
    struct image before,after;if(!control_observe(pid,&before)||before.birth!=birth)return 0;
    mach_port_name_t task=MACH_PORT_NULL;int named=task_name_for_pid(mach_task_self(),pid,&task);
    if(named!=KERN_SUCCESS||task==MACH_PORT_NULL){if(task!=MACH_PORT_NULL)mach_port_deallocate(mach_task_self(),task);return 0;}
    audit_token_t token;mach_msg_type_number_t count=TASK_AUDIT_TOKEN_COUNT;
    kern_return_t result=task_info(task,TASK_AUDIT_TOKEN,(task_info_t)&token,&count);mach_port_deallocate(mach_task_self(),task);
    if(result!=KERN_SUCCESS||count!=TASK_AUDIT_TOKEN_COUNT||token.val[5]!=(unsigned)pid||!token.val[7]||!control_observe(pid,&after)||after.birth!=birth)return 0;
    *out=token;return 1;
}
static audit_token_t control_token(pid_t pid,uint64_t birth) {
    audit_token_t token;require(known_control_token(pid,birth,&token),"control token actual birth");return token;
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
enum source_qualified_counter_kind { COUNTER_SOURCE_NONE,COUNTER_SOURCE_POST_ACK_RI,COUNTER_SOURCE_OWN_WAIT4,COUNTER_SOURCE_LEADER_ROLLUP };
struct window_ledger {
    uint64_t operation,sequence,t0,t_close,cpu_start,cpu_close,end[2],actual_cpu,pipe_actual,pipe_charged;
    uint64_t windows,controller,charge;
    unsigned slots,unknown_mask,ack_mask,identity_mask;
    int roster_closed,source_complete,exec_pending,required_after_close,bad_clock,retired_open;
    // Decision66236: these are proof facts, never a caller-selected allowance.
    unsigned measured,baseline_mask,counter_mask,post_ack_mask,wait4_mask,counter_identity_mask,conversion_mask;
    uint64_t slot_cpu[2],baseline_ticks[2],endpoint_ticks[2],endpoint_wait4_ns[2],baseline_read[2],counter_read[2];
    uint32_t baseline_version[2],counter_version[2],timebase_numer,timebase_denom;
    unsigned counter_kind[2],rollup_mask,rollup_unique,rollup_child_retired,force_window;
    uint64_t rollup_child_exit_ns,rollup_child_zero_ns,rollup_leader_ack_ns,rollup_read_ns,live_child_cpu_ns,rollup_child_cpu_ns;
    uint64_t wait4_subtree_ns;
};
static int engine_scaled_ticks(uint64_t,uint32_t,uint32_t,int,uint64_t *);
enum ledger_result { LEDGER_OK,LEDGER_CLOCK,LEDGER_IDENTITY,LEDGER_ROSTER,LEDGER_PIPE,LEDGER_COVERAGE,LEDGER_LIMIT };
static enum ledger_result window_amounts(struct window_ledger *g,int measured) {
    g->windows=0;g->controller=0;g->charge=0;g->measured=0;
    if(g->slots==0||g->slots>2||g->t0==0||g->t0==UINT64_MAX||g->t_close==UINT64_MAX
        ||g->cpu_start==UINT64_MAX||g->cpu_close==UINT64_MAX||g->actual_cpu==UINT64_MAX||g->cpu_close<g->cpu_start)return LEDGER_CLOCK;
    for(unsigned i=0;i<g->slots;i++) {
        if(g->end[i]==UINT64_MAX||g->end[i]<g->t0||g->end[i]>g->t_close)return LEDGER_CLOCK;
        uint64_t n=g->end[i]-g->t0;if(g->windows>UINT64_MAX-n)return LEDGER_CLOCK;g->windows+=n;
    }
    uint64_t raw=g->cpu_close-g->cpu_start;if(raw>UINT64_MAX-g->pipe_charged)return LEDGER_CLOCK;
    g->controller=raw+g->pipe_charged;
    uint64_t helper=measured?g->actual_cpu:(g->windows>g->actual_cpu?g->windows:g->actual_cpu);
    if(helper>UINT64_MAX-g->controller)return LEDGER_CLOCK;g->charge=helper+g->controller;
    g->measured=(unsigned)measured;
    return g->charge<=1000000&&g->windows<=25000000?LEDGER_OK:LEDGER_LIMIT;
}
static enum ledger_result window_charge(struct window_ledger *g) {
    g->windows=0;g->controller=0;g->charge=0;g->measured=0;
    if(g->bad_clock||g->t0==0||g->t_close<g->t0||g->cpu_close<g->cpu_start)return LEDGER_CLOCK;
    if(g->operation==0||g->sequence==0||g->slots==0||g->slots>2||g->ack_mask!=((1u<<g->slots)-1)
        ||g->identity_mask!=g->ack_mask||g->exec_pending||g->retired_open)return LEDGER_IDENTITY;
    if(!g->roster_closed||!g->source_complete||g->required_after_close)return LEDGER_ROSTER;
    if(g->pipe_charged!=g->pipe_actual)return LEDGER_PIPE;
    for(unsigned i=0;i<g->slots;i++)if((g->unknown_mask&(1u<<i))&&g->end[i]!=g->t_close)return LEDGER_ROSTER;
    enum ledger_result amounts=window_amounts(g,0);
    if(amounts==LEDGER_CLOCK)return amounts;
    unsigned all=(1u<<g->slots)-1;g->measured=0;
    int measured=!g->force_window&&g->unknown_mask==0&&g->baseline_mask==all&&g->counter_mask==all
        &&g->counter_identity_mask==all&&g->conversion_mask==all
        &&((g->post_ack_mask|g->wait4_mask|g->rollup_mask)&all)==all&&g->timebase_numer&&g->timebase_denom;
    if(g->rollup_mask) {
        if(g->slots!=2||g->rollup_mask!=2||!g->rollup_unique||!g->rollup_child_retired
            ||!g->rollup_child_exit_ns||!g->rollup_child_zero_ns||!g->rollup_leader_ack_ns
            ||g->rollup_read_ns<g->rollup_child_exit_ns||g->rollup_read_ns<g->rollup_child_zero_ns
            ||g->rollup_read_ns<g->rollup_leader_ack_ns||g->rollup_read_ns>g->t_close
            ||g->counter_read[1]!=g->rollup_read_ns)measured=0;
        if(g->live_child_cpu_ns||g->rollup_child_cpu_ns!=g->slot_cpu[1])return LEDGER_COVERAGE;
    }
    uint64_t measured_sum=0;
    for(unsigned i=0;i<g->slots;i++) {
        uint64_t wall=g->end[i]-g->t0;
        if((g->counter_mask&(1u<<i))&&g->slot_cpu[i]>wall)return LEDGER_COVERAGE;
        if(!g->baseline_read[i]||g->baseline_read[i]>g->t0||g->counter_read[i]<g->end[i]
            ||g->counter_read[i]>g->t_close||g->baseline_version[i]!=g->counter_version[i])measured=0;
        if((g->counter_mask&(1u<<i))&&UINT64_MAX-measured_sum<g->slot_cpu[i])return LEDGER_CLOCK;
        if(g->counter_mask&(1u<<i))measured_sum+=g->slot_cpu[i];
        if(!(g->wait4_mask&(1u<<i))&&g->endpoint_ticks[i]<g->baseline_ticks[i])measured=0;
        if((g->post_ack_mask&(1u<<i))&&g->counter_kind[i]!=COUNTER_SOURCE_POST_ACK_RI)measured=0;
        if((g->wait4_mask&(1u<<i))&&g->counter_kind[i]!=COUNTER_SOURCE_OWN_WAIT4)measured=0;
        if((g->rollup_mask&(1u<<i))&&g->counter_kind[i]!=COUNTER_SOURCE_LEADER_ROLLUP)measured=0;
    }
    if(measured&&measured_sum!=g->actual_cpu)return LEDGER_COVERAGE;
    if(g->actual_cpu>g->windows)return LEDGER_COVERAGE;
    return window_amounts(g,measured);
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
    struct window_ledger measured={.operation=1,.sequence=1,.t0=100,.t_close=1200200,.cpu_start=10,.cpu_close=300010,
        .end={1200100,0},.actual_cpu=200000,.slots=1,.ack_mask=1,.identity_mask=1,.roster_closed=1,.source_complete=1,
        .baseline_mask=1,.counter_mask=1,.post_ack_mask=1,.counter_identity_mask=1,.conversion_mask=1,
        .slot_cpu={200000,0},.baseline_ticks={10,0},.endpoint_ticks={200010,0},.baseline_read={90,0},.counter_read={1200150,0},
        .baseline_version={1,0},.counter_version={1,0},.timebase_numer=1,.timebase_denom=1,.counter_kind={COUNTER_SOURCE_POST_ACK_RI,0}};
    require(window_charge(&measured)==LEDGER_OK&&measured.measured&&measured.charge==500000,"closed post-ACK measured CPU replaces wall proxy");count++;
    bad=measured;bad.post_ack_mask=0;require(window_charge(&bad)==LEDGER_LIMIT&&!bad.measured,"missing post-ACK counter retains window charge");count++;
    bad=measured;bad.counter_read[0]=200;require(window_charge(&bad)==LEDGER_LIMIT&&!bad.measured,"pre-ACK read cannot certify measured charge");count++;
    bad=measured;bad.unknown_mask=1;bad.end[0]=bad.t_close;require(window_charge(&bad)==LEDGER_LIMIT&&!bad.measured,"unknown slot keeps full window basis");count++;
    bad=good;bad.counter_mask=3;bad.slot_cpu[0]=40;bad.slot_cpu[1]=40;require(window_charge(&bad)==LEDGER_COVERAGE,"per-slot A over W cannot hide in aggregate window");count++;
    bad=measured;bad.end[0]=30000100;bad.t_close=30000300;bad.counter_read[0]=30000200;
    require(window_charge(&bad)==LEDGER_LIMIT&&bad.measured&&bad.charge==500000,"measured CPU never waives hard twenty-five millisecond wall bound");count++;
    struct window_ledger rollup=measured;rollup.slots=2;rollup.end[1]=1000100;
    rollup.ack_mask=rollup.identity_mask=rollup.baseline_mask=rollup.counter_mask=rollup.counter_identity_mask=rollup.conversion_mask=3;
    rollup.rollup_mask=2;rollup.post_ack_mask=1;rollup.rollup_unique=rollup.rollup_child_retired=1;
    rollup.counter_kind[1]=COUNTER_SOURCE_LEADER_ROLLUP;rollup.slot_cpu[1]=100000;rollup.actual_cpu=300000;rollup.rollup_child_cpu_ns=100000;
    rollup.baseline_read[1]=90;rollup.counter_read[1]=rollup.rollup_read_ns=1200150;
    rollup.baseline_ticks[1]=10;rollup.endpoint_ticks[1]=100010;rollup.baseline_version[1]=rollup.counter_version[1]=2;
    rollup.rollup_child_exit_ns=1000100;rollup.rollup_child_zero_ns=1100100;rollup.rollup_leader_ack_ns=1200100;
    require(window_charge(&rollup)==LEDGER_OK&&rollup.measured,"unique retired-child rollup has a distinct conditional source");
    bad=rollup;bad.rollup_unique=0;require(window_charge(&bad)==LEDGER_LIMIT&&!bad.measured,"mixed child rollup cannot certify measured basis");count++;
    bad=rollup;bad.rollup_read_ns=bad.counter_read[1]=1000000;require(window_charge(&bad)==LEDGER_LIMIT&&!bad.measured,"pre-exit rollup cannot acquire later exit proof");count++;
    bad=rollup;bad.live_child_cpu_ns=100000;require(window_charge(&bad)==LEDGER_COVERAGE,"live child and leader rollup cannot both fund the same CPU");count++;
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
    struct window_ledger *g=&evidence.grant;
    g->operation=a.start_absolute;g->sequence=1;g->slots=2;g->source_complete=1;
    mach_timebase_info_data_t scale;require(mach_timebase_info(&scale)==KERN_SUCCESS&&scale.numer&&scale.denom,"member-window explicit timebase");
    g->timebase_numer=scale.numer;g->timebase_denom=scale.denom;
    g->baseline_ticks[0]=a.raw_rusage_cpu;g->baseline_ticks[1]=b.raw_rusage_cpu;
    g->baseline_read[0]=g->baseline_read[1]=clock_ns(CLOCK_MONOTONIC);
    g->baseline_version[0]=ta.val[7];g->baseline_version[1]=tb.val[7];g->baseline_mask=3;
    g->cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);g->t0=clock_ns(CLOCK_MONOTONIC);
    require(kill(-leader,SIGCONT)==0,"member-window original CONT delivery");
    while(clock_ns(CLOCK_MONOTONIC)-g->t0<50000){}
    require(kill(-leader,SIGSTOP)==0,"member-window original STOP request");
    struct image end_a=acknowledged_member(leader,&a,&ta,path);g->end[0]=clock_ns(CLOCK_MONOTONIC);
    uint64_t delayed=clock_ns(CLOCK_MONOTONIC);while(clock_ns(CLOCK_MONOTONIC)-delayed<50000){}
    struct image end_b=acknowledged_member(child,&b,&tb,path);g->end[1]=clock_ns(CLOCK_MONOTONIC);
    g->ack_mask=3;g->identity_mask=3;
    complete_known_roster(leader,child,&a,&b,&ta,&tb,path,&end_a,&end_b);g->roster_closed=1;
    g->endpoint_ticks[0]=end_a.raw_rusage_cpu;g->endpoint_ticks[1]=end_b.raw_rusage_cpu;
    g->counter_read[0]=end_a.observed_at;g->counter_read[1]=end_b.observed_at;
    g->counter_version[0]=ta.val[7];g->counter_version[1]=tb.val[7];
    g->counter_mask=g->post_ack_mask=g->counter_identity_mask=g->conversion_mask=3;
    g->counter_kind[0]=g->counter_kind[1]=COUNTER_SOURCE_POST_ACK_RI;
    for(unsigned i=0;i<2;i++)require(g->endpoint_ticks[i]>=g->baseline_ticks[i]
        &&engine_scaled_ticks(g->endpoint_ticks[i]-g->baseline_ticks[i],scale.numer,scale.denom,1,&g->slot_cpu[i]),"monotonic per-slot explicit converted active CPU");
    require(UINT64_MAX-g->slot_cpu[0]>=g->slot_cpu[1],"checked nonduplicated native slot CPU sum");g->actual_cpu=g->slot_cpu[0]+g->slot_cpu[1];
    g->live_child_cpu_ns=g->slot_cpu[1];
    g->t_close=clock_ns(CLOCK_MONOTONIC);g->cpu_close=clock_ns(CLOCK_THREAD_CPUTIME_ID);
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
struct prepared_spawn_outcome {
    pid_t pid;int attempted,error,cpu_known,wall_known;
    uint64_t cpu_start,cpu_end,wall_start,wall_end,cpu,wall;
};
static void prepared_spawn(const char *program,char *const argv[],char *const envp[],
    const posix_spawn_file_actions_t *actions,const posix_spawnattr_t *attr,int own_group,struct prepared_spawn_outcome *out) {
    memset(out,0,sizeof(*out));out->cpu=out->wall=UINT64_MAX;
    out->cpu_start=out->cpu_end=out->wall_start=out->wall_end=UINT64_MAX;
    if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&out->cpu_start)||!known_clock_ns(CLOCK_MONOTONIC,&out->wall_start))return;
    out->attempted=1;out->pid=-1;
    out->error=posix_spawn(&out->pid,program,actions,attr,argv,envp);
    // Retain the actual effect BEFORE either ending clock or any verdict.
    if(out->pid>0){if(own_group)retained_group=out->pid;else retained_child=out->pid;}
    int wall=known_clock_ns(CLOCK_MONOTONIC,&out->wall_end),cpu=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&out->cpu_end);
    out->wall_known=wall&&out->wall_end>=out->wall_start;out->cpu_known=cpu&&out->cpu_end>=out->cpu_start;
    if(out->wall_known)out->wall=out->wall_end-out->wall_start;
    if(out->cpu_known)out->cpu=out->cpu_end-out->cpu_start;
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
    struct prepared_spawn_outcome out;prepared_spawn(program,argv,envp,&actions,&attr,own_group,&out);
    pid_t pid=out.pid;*cpu=out.cpu;*wall=out.wall;
    require(out.attempted&&out.cpu_known&&out.wall_known,"known prepared creation clock");
    if (out.error != 0 || pid <= 0) { errno = out.error; require(0, "suspended creation"); }
    posix_spawn_file_actions_destroy(&actions); posix_spawnattr_destroy(&attr);
    if (*cpu > 2000000 || *wall > 25000000) {
        dispose_owned();
        require(0, "native creation 2ms CPU/25ms wall");
    }
    return pid;
}
static int known_exited(pid_t pid,int *done) {
    siginfo_t info; memset(&info, 0, sizeof(info));
    if(waitid(P_PID,pid,&info,WEXITED|WNOHANG|WNOWAIT)!=0)return 0;
    *done=info.si_pid==pid&&(info.si_code==CLD_EXITED||info.si_code==CLD_KILLED||info.si_code==CLD_DUMPED);return 1;
}
static int exited(pid_t pid) {
    int done=0;require(known_exited(pid,&done),"leader nonreaping observation");return done;
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
    printf("{\"created_suspended\":true,\"cloexec_default\":true,\"audit_token_signals\":true,\"running_child_stop_applied\":true,\"stale_pid_version_control\":true,\"mach_timebase_numer\":%u,\"mach_timebase_denom\":%u,\"sampled_child_cpu_raw\":%llu,\"initial_native_cpu_ns\":%llu,\"confirmed_native_cpu_ns\":%llu,\"native_bootstrap_reconciliation_ns\":%llu,\"stop_confirm_wall_ns\":%llu,\"held_user_cpu_delta_ns\":0,\"held_kernel_cpu_ns\":%llu,\"held_kernel_paid_debt_ns\":%llu,\"threads\":1,\"confirmed_runnable\":0,\"stale_audit_error\":%d,\"leader\":%d,\"child\":%d,\"pgid\":%d,\"leader_birth\":%llu,\"child_birth\":%llu,\"members\":2,\"sampled_child_cpu_ns\":%llu,\"waited_child_cpu_ns\":%llu,\"leader_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true,\"member_windows\":{\"basis\":\"canonical_private_c_program_only\",\"charge_basis\":\"%s\",\"slot_A_ns\":[%llu,%llu],\"unknown_mask\":%u,\"counter_mask\":%u,\"post_ack_mask\":%u,\"wait4_mask\":%u,\"baseline_mask\":%u,\"counter_identity_mask\":%u,\"conversion_mask\":%u,\"operation\":%llu,\"sequence\":%llu,\"t0_ns\":%llu,\"endpoint_ns\":[%llu,%llu],\"tclose_ns\":%llu,\"W_ns\":%llu,\"A_ns\":%llu,\"C_ns\":%llu,\"Q_ns\":%llu,\"roster_closed\":true,\"late_kernel_ns\":%llu,\"post_cpu_ns\":%llu,\"debt_deadline_ns\":%llu,\"causal_controls\":%u}}\n",
        timebase.numer,timebase.denom,(unsigned long long)child_raw_cpu,(unsigned long long)initial_cpu,(unsigned long long)before.cpu,(unsigned long long)bootstrap_cpu,(unsigned long long)confirm_wall,
        (unsigned long long)held_kernel_cpu,(unsigned long long)(9*held_kernel_cpu),stale_error,pid, child_pid, pid, (unsigned long long)before.birth, (unsigned long long)child.birth,
        (unsigned long long)child_cpu, (unsigned long long)waited_cpu, (unsigned long long)ru_cpu(&usage),
        (unsigned long long)peak_rss, (unsigned long long)create_cpu, (unsigned long long)create_wall,
        windows.grant.measured?"measured":"window",(unsigned long long)windows.grant.slot_cpu[0],(unsigned long long)windows.grant.slot_cpu[1],
        windows.grant.unknown_mask,windows.grant.counter_mask,windows.grant.post_ack_mask,windows.grant.wait4_mask,windows.grant.baseline_mask,windows.grant.counter_identity_mask,windows.grant.conversion_mask,
        (unsigned long long)windows.grant.operation,(unsigned long long)windows.grant.sequence,(unsigned long long)windows.grant.t0,
        (unsigned long long)windows.grant.end[0],(unsigned long long)windows.grant.end[1],(unsigned long long)windows.grant.t_close,
        (unsigned long long)windows.grant.windows,(unsigned long long)windows.grant.actual_cpu,(unsigned long long)windows.grant.controller,
        (unsigned long long)windows.grant.charge,(unsigned long long)windows.late_cpu,(unsigned long long)windows.post_cpu,(unsigned long long)windows.debt,windows.controls);
    return 0;
}

// Private-candidate outer schema from interfaces/dream-proof-tool-qualification
// ETag 98a3023e2a24233959de04bb573a73274ff9980f4da80eb91bb4a89cd1fcaa7e.
// This decoder grants no runtime authority. Missing actual source/closure
// evidence stays unbound, before a helper creation or original claim.
#define PRIVATE_INPUT_MAX 16384u
#define PRIVATE_EVIDENCE_MAX 131072u
struct private_dir {char path[4097];uint64_t dev,ino;uint32_t uid;int fd;};
struct private_file {char path[4097],sha[65];uint64_t dev,ino,len;int64_t mtime_sec;uint32_t mtime_nsec;unsigned root;};
struct private_input {
    char run[27],nonce[27],arch[129],os_build[129],source[65],exe[65],case_name[32];
    uint64_t incarnation,deadline;
    struct private_dir prefix,repo,output;
    struct private_file source_lock,build,review,closure,config,commands,git,gpg,manifest,keys;
    char valid_commit[41],corrupt_commit[41];
};
// Fixed raw-copy, result, parser/path/hash/stat frames share this declared
// peak metadata reservation. Opaque libc/native allocation stays RSS coverage.
#define PRIVATE_METADATA_CAPACITY (sizeof(struct private_input)+PRIVATE_INPUT_MAX+1u+8192u+32768u)
_Static_assert(PRIVATE_METADATA_CAPACITY<=PRIVATE_EVIDENCE_MAX,"private fixed metadata capacity");
struct json_reader {const unsigned char *p,*end;unsigned depth,limit;};
struct private_metrics {
    uint64_t entry,deadline,ordinary_max,debt,debt_paid,rest_until,rest_poll_cpu,metadata,open_cpu,last_unit,close_max,rss_peak;
    // Closed units belong to this same owner BEFORE their CPU epoch advances.
    // A missing scheduling clock cannot erase their exact normalized/raw cost.
    uint64_t retained_cost_ns,retained_raw_ns,retained_units,retained_unknown_units,closed_units,booked_units;
    uint64_t retained_adjustment_cost_ns,retained_adjustments,booked_adjustments;
    int rss_known;unsigned custody_hold,retained_overflow;pid_t original_parent;
};
static const char *private_error="input_invalid",*private_detail="strict private input decoding failed";
static struct private_metrics private_cost;
static int private_has_retained_cost(void) {
    return private_cost.retained_units||private_cost.retained_adjustments||private_cost.retained_adjustment_cost_ns;
}
static int private_failed=0;
static const char *const private_error_codes[]={"cancelled","clock_unknown","coverage_unknown","deadline","input_invalid","resource_limit","review_unbound","settlement_unknown","signature_unexpected","source_changed","other"};
struct private_error_observation {uint64_t count;char first[161],last[161];unsigned truncated;};
static struct private_error_observation private_errors[11];
static uint64_t private_error_count;
static unsigned private_errors_overflow;
struct private_terminal_observation {
    unsigned control_known,control_ok,started,cost_ok,cost_unknown,tail_complete,diagnostics_ok;
    unsigned persistence_attempted,persist_open,persist_write,persist_fsync,persist_close,persist_dir_fsync,persist_ok,persist_unknown,stdout_complete;
    unsigned format_paid,persistence_paid,write_paid;
    int persistence_errno,stdout_errno;
    uint64_t format_start,format_end,format_inclusive,write_start,write_end,write_inclusive;
};
static struct private_terminal_observation private_terminal;
static void private_reset_diagnostics(void) {
    memset(private_errors,0,sizeof(private_errors));private_error_count=0;private_errors_overflow=0;
    memset(&private_terminal,0,sizeof(private_terminal));private_terminal.cost_ok=private_terminal.diagnostics_ok=1;
}
static void private_record_error(const char *code,const char *detail) {
    unsigned index=0;while(index<10&&strcmp(code,private_error_codes[index]))index++;
    struct private_error_observation *row=&private_errors[index];
    if(index==10)private_errors_overflow=1;
    if(private_error_count==UINT64_MAX||row->count==UINT64_MAX)private_errors_overflow=1;
    else {private_error_count++;row->count++;}
    if(!detail)detail="";size_t n=strnlen(detail,sizeof(row->last));
    if(n>=sizeof(row->last)){n=sizeof(row->last)-1;row->truncated=1;}
    if(row->count==1){memcpy(row->first,detail,n);row->first[n]=0;}
    memcpy(row->last,detail,n);row->last[n]=0;
}
static int private_fail(const char *code,const char *detail) {
    private_record_error(code,detail);
    if(!private_failed){private_error=code;private_detail=detail;private_failed=1;}return 0;
}
static int private_unit_outcome(int ok) {
    if(private_terminal.started&&!ok) {
        private_terminal.cost_ok=0;
        if(private_cost.last_unit==UINT64_MAX||private_cost.open_cpu==UINT64_MAX||private_has_retained_cost()
            ||private_cost.retained_overflow||private_cost.debt_paid!=private_cost.debt)private_terminal.cost_unknown=1;
    }
    return ok;
}
static void (*private_rest_control)(void)=NULL;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
// Interrupted observations only; no test hook is present in the probe build.
static unsigned source_known_payment_interrupts,source_known_payment_fail_nth,source_known_schedule_interrupts,source_known_schedule_fail_nth;
#endif
static int private_retain_closed_cost(uint64_t raw_cost) {
    uint64_t cost=raw_cost>1000000?raw_cost:1000000;
    if(raw_cost==UINT64_MAX||cost>UINT64_MAX/9) {
        private_fail("clock_unknown","unknown or overflowing closed-unit cost");
        cost=3000000; // Canonical finite mandatory-settlement debt; raw cost stays unknown.
    }
    if(private_cost.retained_overflow||UINT64_MAX-private_cost.retained_cost_ns<cost
        ||private_cost.retained_units==UINT64_MAX||private_cost.closed_units==UINT64_MAX
        ||(raw_cost==UINT64_MAX?private_cost.retained_unknown_units==UINT64_MAX:UINT64_MAX-private_cost.retained_raw_ns<raw_cost)) {
        private_cost.retained_overflow=1;
        return private_fail("clock_unknown","retained closed-unit cost overflow; original owner stays held");
    }
    private_cost.retained_cost_ns+=cost;private_cost.retained_units++;private_cost.closed_units++;
    if(raw_cost==UINT64_MAX)private_cost.retained_unknown_units++;
    else private_cost.retained_raw_ns+=raw_cost;
    return 1;
}
static int private_retain_debt_adjustment(uint64_t extra_cost) {
    // Arrears of ALREADY closed grants. This adds their missing normalized
    // rest cost exactly once; it creates no new work/native class or floor.
    // The revocation observation/formatter remains a separate ordinary unit.
    if(!extra_cost)return 1;
    if(extra_cost==UINT64_MAX||private_cost.retained_overflow
        ||UINT64_MAX-private_cost.retained_adjustment_cost_ns<extra_cost||private_cost.retained_adjustments==UINT64_MAX) {
        private_cost.retained_overflow=1;
        return private_fail("clock_unknown","fallback debt adjustment cannot be retained");
    }
    private_cost.retained_adjustment_cost_ns+=extra_cost;private_cost.retained_adjustments++;return 1;
}
static int private_book_retained_cost(void) {
    if(private_cost.retained_overflow)return private_fail("clock_unknown","retained closed-unit cost is unrepresentable");
    if(!private_has_retained_cost())return 1;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_schedule_interrupts){source_known_schedule_interrupts--;return private_fail("clock_unknown","source-known interrupted debt scheduling");}
    if(source_known_schedule_fail_nth&&!--source_known_schedule_fail_nth)return private_fail("clock_unknown","source-known interrupted selected debt scheduling");
#endif
    uint64_t cost=private_cost.retained_cost_ns;
    if(UINT64_MAX-cost<private_cost.retained_adjustment_cost_ns)return private_fail("clock_unknown","retained work and fallback debt sum overflow");
    cost+=private_cost.retained_adjustment_cost_ns;
    uint64_t now;
    if(!known_clock_ns(CLOCK_MONOTONIC,&now))return private_fail("clock_unknown","shared debt clock unavailable");
    uint64_t base=private_cost.rest_until>now?private_cost.rest_until:now;
    if(cost>UINT64_MAX/9||UINT64_MAX-base<9*cost
        ||UINT64_MAX-private_cost.debt<9*cost||UINT64_MAX-private_cost.booked_units<private_cost.retained_units
        ||UINT64_MAX-private_cost.booked_adjustments<private_cost.retained_adjustments)
        return private_fail("clock_unknown","shared additive debt overflow");
    uint64_t deadline=base+9*cost;
    // Moving retained -> booked is atomic in this single controller. Failed
    // payment leaves the booked debt intact; recovery never books it twice.
    private_cost.rest_until=deadline;private_cost.debt+=9*cost;private_cost.booked_units+=private_cost.retained_units;
    private_cost.booked_adjustments+=private_cost.retained_adjustments;
    private_cost.retained_cost_ns=private_cost.retained_raw_ns=private_cost.retained_units=private_cost.retained_unknown_units=0;
    private_cost.retained_adjustment_cost_ns=private_cost.retained_adjustments=0;
    return 1;
}
static int private_schedule_cost(uint64_t raw_cost) {
    if(!private_retain_closed_cost(raw_cost))return 0;
    if(!private_book_retained_cost())return 0;
    return 1;
}
static int private_pay_pending(void) {
    if(!private_book_retained_cost())return 0;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_payment_interrupts){source_known_payment_interrupts--;return private_fail("clock_unknown","source-known interrupted debt payment");}
    if(source_known_payment_fail_nth&&!--source_known_payment_fail_nth)return private_fail("clock_unknown","source-known interrupted selected debt payment");
#endif
    uint64_t pay_begin;
    if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&pay_begin))return private_fail("clock_unknown","rest payment beginning clock unavailable");
    if(!private_cost.open_cpu)private_cost.open_cpu=pay_begin;
    for(;;) {
        uint64_t now;
        if(!known_clock_ns(CLOCK_MONOTONIC,&now))return private_fail("clock_unknown","shared rest clock unavailable");
        if(now>=private_cost.rest_until)break;
        if(private_rest_control)private_rest_control();
        uint64_t left=private_cost.rest_until-now;
        usleep((useconds_t)(((left>1000000?1000000:left)+999)/1000));
    }
    private_cost.debt_paid=private_cost.debt;
    uint64_t ended;
    if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&ended)||ended<pay_begin) {
        private_cost.open_cpu=UINT64_MAX;return private_fail("clock_unknown","next ordinary beginning clock unavailable");
    }
    if(UINT64_MAX-private_cost.rest_poll_cpu<ended-pay_begin)return private_fail("clock_unknown","rest polling CPU sum overflow");
    private_cost.rest_poll_cpu+=ended-pay_begin;
    // Keep the already-open CPU epoch. Payment polling and control callbacks
    // therefore enter the next measured unit; resetting it to `ended` would
    // silently discard that actual application work. Final observer overhead
    // remains an explicit measurement-cutoff limitation, never tool approval.
    return 1;
}
static int private_end_unit(uint64_t at) {
    if(private_cost.open_cpu)at=private_cost.open_cpu;
    uint64_t end,used=UINT64_MAX;
    if(known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&end)&&at!=UINT64_MAX&&end>=at)used=end-at;
    else private_fail("clock_unknown","unavailable or reversed ordinary clock");
    private_cost.last_unit=used;
    if(used>private_cost.ordinary_max)private_cost.ordinary_max=used;
    int scheduled=private_schedule_cost(used);
    private_cost.open_cpu=used==UINT64_MAX?UINT64_MAX:end;
    if(!scheduled||!private_pay_pending())return private_unit_outcome(0);
    if(used==UINT64_MAX)return private_unit_outcome(0);
    if(used>1000000)return private_unit_outcome(private_fail("resource_limit","ordinary validation exceeds one millisecond"));
    if(!private_cost.custody_hold&&clock_ns(CLOCK_MONOTONIC)>=private_cost.deadline)return private_unit_outcome(private_fail("deadline","original private test deadline expired"));
    return 1;
}
static int private_utf8(const unsigned char *p,size_t n) {
    size_t i=0;
    while(i<n) {
        unsigned a=p[i++];
        if(a<0x80)continue;
        unsigned need,min,cp;
        if(a>=0xc2&&a<=0xdf){need=1;min=0x80;cp=a&31;}
        else if(a>=0xe0&&a<=0xef){need=2;min=0x800;cp=a&15;}
        else if(a>=0xf0&&a<=0xf4){need=3;min=0x10000;cp=a&7;}
        else return 0;
        if(need>n-i)return 0;
        for(unsigned k=0;k<need;k++){unsigned b=p[i++];if((b&0xc0)!=0x80)return 0;cp=(cp<<6)|(b&63);}
        if(cp<min||cp>0x10ffff||(cp>=0xd800&&cp<=0xdfff))return 0;
    }
    return 1;
}
static int jchar(struct json_reader *r,unsigned c) {if(r->p==r->end||*r->p!=c)return 0;r->p++;return 1;}
static int jstring(struct json_reader *r,char *out,size_t cap) {
    if(!jchar(r,'"'))return 0;
    size_t n=0;
    while(r->p<r->end) {
        unsigned c=*r->p++;
        if(c=='"'){out[n]=0;return 1;}
        if(c<32)return 0;
        if(c=='\\') {
            if(r->p==r->end)return 0;
            unsigned e=*r->p++;
            if(e=='"'||e=='\\')c=e;
            else if(e=='b')c=8;else if(e=='f')c=12;else if(e=='n')c=10;else if(e=='r')c=13;else if(e=='t')c=9;
            else if(e=='u') {
                if(r->end-r->p<4||r->p[0]!='0'||r->p[1]!='0')return 0;
                unsigned x=0;
                for(unsigned i=2;i<4;i++){unsigned d=r->p[i];if(d>='0'&&d<='9')d-='0';else if(d>='a'&&d<='f')d=d-'a'+10;else return 0;x=(x<<4)|d;}
                r->p+=4;
                if(x==0||x>=32||x==8||x==9||x==10||x==12||x==13)return 0;
                c=x;
            } else return 0;
        }
        if(c==0||n+1>=cap)return 0;
        out[n++]=(char)c;
    }
    return 0;
}
static int jkey(struct json_reader *r,const char *key,int first) {
    char actual[64];return (first||jchar(r,','))&&jstring(r,actual,sizeof(actual))&&!strcmp(actual,key)&&jchar(r,':');
}
static int jobject(struct json_reader *r) {if(r->depth>=(r->limit?r->limit:6)||!jchar(r,'{'))return 0;r->depth++;return 1;}
static int jclose(struct json_reader *r) {if(!r->depth||!jchar(r,'}'))return 0;r->depth--;return 1;}
static int ju64(struct json_reader *r,uint64_t *out) {
    if(r->p==r->end||*r->p<'0'||*r->p>'9')return 0;
    const unsigned char *begin=r->p;uint64_t value=0;unsigned first=*r->p;
    do {unsigned d=*r->p++-'0';if(value>(UINT64_MAX-d)/10)return 0;value=value*10+d;}
    while(r->p<r->end&&*r->p>='0'&&*r->p<='9');
    if(first=='0'&&r->p-begin!=1)return 0;
    *out=value;return 1;
}
static int ji64(struct json_reader *r,int64_t *out) {
    int negative=r->p<r->end&&*r->p=='-';if(negative)r->p++;
    uint64_t v;if(!ju64(r,&v)||(negative&&v==0))return 0;
    if(!negative){if(v>INT64_MAX)return 0;*out=(int64_t)v;}
    else {if(v>(uint64_t)INT64_MAX+1)return 0;*out=v==(uint64_t)INT64_MAX+1?INT64_MIN:-(int64_t)v;}
    return 1;
}
static int j32(struct json_reader *r,uint32_t *out) {uint64_t v;if(!ju64(r,&v)||v>UINT32_MAX)return 0;*out=(uint32_t)v;return 1;}
static int hex_value(const char *s,size_t n) {if(strnlen(s,n+1)!=n)return 0;for(size_t i=0;i<n;i++)if(!((s[i]>='0'&&s[i]<='9')||(s[i]>='a'&&s[i]<='f')))return 0;return 1;}
static int julid(const char *s) {if(strlen(s)!=26||s[0]<'0'||s[0]>'7')return 0;for(unsigned i=0;i<26;i++)if(!strchr("0123456789ABCDEFGHJKMNPQRSTVWXYZ",s[i]))return 0;return 1;}
static int jhex(struct json_reader *r,char out[65]) {return jstring(r,out,65)&&hex_value(out,64);}
static int jdir(struct json_reader *r,struct private_dir *d) {
    d->fd=-1;
    return jobject(r)&&jkey(r,"dev",1)&&ju64(r,&d->dev)&&jkey(r,"ino",0)&&ju64(r,&d->ino)
        &&jkey(r,"path",0)&&jstring(r,d->path,sizeof(d->path))&&jkey(r,"uid",0)&&j32(r,&d->uid)&&jclose(r);
}
static int jfile(struct json_reader *r,struct private_file *f) {
    char root[10];
    if(!(jobject(r)&&jkey(r,"dev",1)&&ju64(r,&f->dev)&&jkey(r,"ino",0)&&ju64(r,&f->ino)
        &&jkey(r,"len",0)&&ju64(r,&f->len)&&jkey(r,"mtime_nsec",0)&&j32(r,&f->mtime_nsec)
        &&f->mtime_nsec<1000000000&&jkey(r,"mtime_sec",0)&&ji64(r,&f->mtime_sec)
        &&jkey(r,"path",0)&&jstring(r,f->path,sizeof(f->path))&&jkey(r,"root",0)&&jstring(r,root,sizeof(root))
        &&jkey(r,"sha256",0)&&jhex(r,f->sha)&&jclose(r)))return 0;
    if(!strcmp(root,"candidate"))f->root=0;else if(!strcmp(root,"fixture"))f->root=1;else return 0;
    return 1;
}
static int private_canonical(const unsigned char *bytes,size_t len);
static int private_decode(const unsigned char *bytes,size_t len,struct private_input *v) {
    if(!len||len>PRIVATE_INPUT_MAX||!private_utf8(bytes,len))return 0;
    memset(v,0,sizeof(*v));v->prefix.fd=v->repo.fd=v->output.fd=-1;
    struct json_reader r={bytes,bytes+len,0,6};char kind[32];
    if(!(jobject(&r)&&jkey(&r,"arch",1)&&jstring(&r,v->arch,sizeof(v->arch))
        &&jkey(&r,"build",0)&&jfile(&r,&v->build)&&jkey(&r,"case",0)&&jstring(&r,v->case_name,sizeof(v->case_name))
        &&jkey(&r,"closure",0)&&jfile(&r,&v->closure)&&jkey(&r,"commands",0)&&jfile(&r,&v->commands)
        &&jkey(&r,"config",0)&&jfile(&r,&v->config)&&jkey(&r,"deadline_ns",0)&&ju64(&r,&v->deadline)
        &&jkey(&r,"fixture",0)&&jobject(&r)&&jkey(&r,"corrupt_commit",1)&&jstring(&r,v->corrupt_commit,sizeof(v->corrupt_commit))
        &&jkey(&r,"manifest",0)&&jfile(&r,&v->manifest)&&jkey(&r,"public_keys",0)&&jfile(&r,&v->keys)
        &&jkey(&r,"repo",0)&&jdir(&r,&v->repo)&&jkey(&r,"valid_commit",0)&&jstring(&r,v->valid_commit,sizeof(v->valid_commit))&&jclose(&r)
        &&jkey(&r,"git",0)&&jfile(&r,&v->git)&&jkey(&r,"gpg",0)&&jfile(&r,&v->gpg)
        &&jkey(&r,"incarnation",0)&&ju64(&r,&v->incarnation)&&v->incarnation>0
        &&jkey(&r,"kind",0)&&jstring(&r,kind,sizeof(kind))&&!strcmp(kind,"private_candidate_test")
        &&jkey(&r,"nonce",0)&&jstring(&r,v->nonce,sizeof(v->nonce))&&julid(v->nonce)
        &&jkey(&r,"os_build",0)&&jstring(&r,v->os_build,sizeof(v->os_build))
        &&jkey(&r,"output",0)&&jdir(&r,&v->output)&&jkey(&r,"prefix",0)&&jdir(&r,&v->prefix)
        &&jkey(&r,"probe_exe_sha256",0)&&jhex(&r,v->exe)&&jkey(&r,"probe_source_sha256",0)&&jhex(&r,v->source)
        &&jkey(&r,"run_id",0)&&jstring(&r,v->run,sizeof(v->run))&&julid(v->run)
        &&jkey(&r,"source_lock",0)&&jfile(&r,&v->source_lock)&&jkey(&r,"source_review",0)&&jfile(&r,&v->review)
        &&jclose(&r)&&r.p==r.end&&r.depth==0))return 0;
    const char *cases[]={"positive","corrupt_proof","pause_resume","cancel","stale_incarnation","clock_failure","debt_overlap","late_stop","reap_delay"};
    int found=0;for(unsigned i=0;i<sizeof(cases)/sizeof(cases[0]);i++)if(!strcmp(v->case_name,cases[i]))found=1;
    return found&&hex_value(v->valid_commit,40)&&hex_value(v->corrupt_commit,40)&&strcmp(v->valid_commit,v->corrupt_commit);
}
static int private_path(const char *path,int absolute) {
    size_t len=strnlen(path,4097);if(!len||len>4096||((path[0]=='/')!=absolute))return 0;
    const char *p=path+(absolute?1:0);if(!*p)return 0;
    while(*p) {const char *end=strchr(p,'/');size_t n=end?(size_t)(end-p):strlen(p);
        if(!n||n>255||(n==1&&p[0]=='.')||(n==2&&p[0]=='.'&&p[1]=='.'))return 0;
        for(size_t i=0;i<n;i++)if((unsigned char)p[i]<32||p[i]=='\\')return 0;
        if(!end)break;p=end+1;if(!*p)return 0;
    }
    return 1;
}
static int private_close(int fd);
static int private_open_walk(int root,const char *path,int absolute,int final_flags) {
    uint64_t begin=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int path_valid=private_path(path,absolute);
    int dir=path_valid?(absolute?open("/",O_RDONLY|O_DIRECTORY|O_CLOEXEC):dup(root)):-1;
    if(!private_end_unit(begin)){private_close(dir);return -1;}
    if(dir<0)return -1;
    const char *p=path+(absolute?1:0);
    while(*p) {
        uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        const char *end=strchr(p,'/');size_t n=end?(size_t)(end-p):strlen(p);char name[256];memcpy(name,p,n);name[n]=0;
        int next=openat(dir,name,(end?O_RDONLY|O_DIRECTORY:final_flags)|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
        close(dir);dir=next;
        if(!private_end_unit(at)){private_close(dir);return -1;}
        if(dir<0)return -1;if(!end)return dir;p=end+1;
    }
    private_close(dir);return -1;
}
static int private_dir_open(struct private_dir *d) {
    int fd=private_open_walk(-1,d->path,1,O_RDONLY|O_DIRECTORY);if(fd<0)return private_fail("source_changed","owned root custody unavailable");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    struct stat st;int result=fstat(fd,&st);
    int valid=result==0&&S_ISDIR(st.st_mode)&&(st.st_mode&0777)==0700&&st.st_uid==geteuid()
        &&(uint64_t)st.st_dev==d->dev&&(uint64_t)st.st_ino==d->ino&&st.st_uid==d->uid;
    // private_open_walk already closed its own bounded units. This final
    // metadata observation gets a separate fresh ordinary unit.
    int paid=private_end_unit(at);
    if(!valid||!paid){private_close(fd);return private_fail("source_changed","owned root identity or mode changed");}
    d->fd=fd;return 1;
}
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
static int private_hash_fd(int fd,uint64_t expected_len,char digest[65]) {
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    CC_SHA256_CTX hash;int initialized=CC_SHA256_Init(&hash);
    if(!private_end_unit(at))return 0;
    if(!initialized)return private_fail("input_invalid","SHA256 initialization failed");
    unsigned char bytes[4096];uint64_t total=0;int eof=0;
    while(!eof) {
        at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int ok=1;
        for(unsigned i=0;i<64;i++) {
            ssize_t n=read(fd,bytes,sizeof(bytes));
            if(n<0){private_fail("source_changed","bound file read failed");ok=0;break;}
            if((uint64_t)n>expected_len-total){private_fail("source_changed","bound file length grew");ok=0;break;}
            total+=(uint64_t)n;
            if(n&&!CC_SHA256_Update(&hash,bytes,(CC_LONG)n)){private_fail("input_invalid","SHA256 update failed");ok=0;break;}
            if(!n){eof=1;break;}
            uint64_t now=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            if(now<at){private_fail("clock_unknown","reversed hash clock");ok=0;break;}
            if(now-at>=400000)break;
        }
        int paid=private_end_unit(at);if(!paid||!ok)return 0;
    }
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    unsigned char value[CC_SHA256_DIGEST_LENGTH];int finalized=CC_SHA256_Final(value,&hash);
    if(finalized)for(unsigned i=0;i<sizeof(value);i++)snprintf(digest+2*i,3,"%02x",value[i]);
    int paid=private_end_unit(at);if(!paid)return 0;
    if(!finalized||total!=expected_len)return private_fail("source_changed","bound file length or hash finalization changed");
    return 1;
}
static void private_hash(const void *bytes,size_t len,char digest[65]) {
    unsigned char value[CC_SHA256_DIGEST_LENGTH];CC_SHA256(bytes,(CC_LONG)len,value);
    for(unsigned i=0;i<sizeof(value);i++)snprintf(digest+2*i,3,"%02x",value[i]);
}
#pragma clang diagnostic pop
static int private_close_observed(int fd,int *actual_closed) {
    *actual_closed=fd<0;if(fd<0)return 1;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int result=close(fd);
    *actual_closed=result==0;
    if(result!=0)private_fail("source_changed","owned validation descriptor close failed");
    uint64_t end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    if(end<at)return private_unit_outcome(private_fail("clock_unknown","reversed descriptor-close clock"));
    if(end-at>private_cost.close_max)private_cost.close_max=end-at;
    return private_end_unit(at);
}
static int private_close(int fd) {int actual_closed;int paid=private_close_observed(fd,&actual_closed);return paid&&actual_closed;}
static int private_root_current(const struct private_dir *root) {
    int current=private_open_walk(-1,root->path,1,O_RDONLY|O_DIRECTORY);if(current<0)return private_fail("source_changed","exact root path changed");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat held,now;
    int same=fstat(root->fd,&held)==0&&fstat(current,&now)==0&&S_ISDIR(now.st_mode)&&now.st_uid==root->uid&&(now.st_mode&0777)==0700
        &&(uint64_t)now.st_dev==root->dev&&(uint64_t)now.st_ino==root->ino&&held.st_dev==now.st_dev&&held.st_ino==now.st_ino;
    int paid=private_end_unit(at);int closed=private_close(current);
    if(!paid||!closed)return 0;if(!same)return private_fail("source_changed","held root and exact path differ");return 1;
}
static int private_same_file(const struct stat *a,const struct stat *b) {
    return a->st_dev==b->st_dev&&a->st_ino==b->st_ino&&a->st_uid==b->st_uid&&a->st_gid==b->st_gid
        &&a->st_mode==b->st_mode&&a->st_nlink==b->st_nlink&&a->st_size==b->st_size
        &&a->st_mtimespec.tv_sec==b->st_mtimespec.tv_sec&&a->st_mtimespec.tv_nsec==b->st_mtimespec.tv_nsec
        &&a->st_ctimespec.tv_sec==b->st_ctimespec.tv_sec&&a->st_ctimespec.tv_nsec==b->st_ctimespec.tv_nsec;
}
static int private_named_current(int fd,int root,const char *path,int absolute,const struct stat *original) {
    int current=private_open_walk(root,path,absolute,O_RDONLY);
    if(current<0)return private_fail("source_changed","original descriptor pathname changed");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat held,named;
    int same=fstat(fd,&held)==0&&fstat(current,&named)==0&&private_same_file(original,&held)&&private_same_file(&held,&named);
    int paid=private_end_unit(at),closed=private_close(current);
    if(!paid||!closed)return 0;
    if(!same)return private_fail("source_changed","original hashed stamp descriptor and pathname differ");return 1;
}
static __attribute__((unused)) int private_file_check(const struct private_input *v,const struct private_file *f,int executable,uint64_t limit) {
    if(f->len>limit||(executable&&!f->len))return private_fail("input_invalid","bound file allocation limit");
    int root=f->root?v->repo.fd:v->prefix.fd;
    int fd=private_open_walk(root,f->path,0,O_RDONLY);if(fd<0)return private_fail("source_changed","bound relative file custody unavailable");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat before,after;int ok=fstat(fd,&before)==0;
    ok=ok&&S_ISREG(before.st_mode)&&before.st_uid==geteuid()&&before.st_nlink==1&&(before.st_mode&0022)==0&&(before.st_mode&06000)==0
        &&(executable||(before.st_mode&077)==0)
        &&(!executable||(before.st_mode&0100)!=0)&&(uint64_t)before.st_dev==f->dev&&(uint64_t)before.st_ino==f->ino
        &&before.st_size>=0&&(uint64_t)before.st_size==f->len&&before.st_mtimespec.tv_sec==f->mtime_sec&&(uint32_t)before.st_mtimespec.tv_nsec==f->mtime_nsec;
    if(!private_end_unit(at)||!ok){private_close(fd);return private_fail("source_changed","bound file identity or mode changed");}
    char hash[65];ok=private_hash_fd(fd,f->len,hash);
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);ok=ok&&fstat(fd,&after)==0&&before.st_dev==after.st_dev&&before.st_ino==after.st_ino
        &&before.st_size==after.st_size&&before.st_mtimespec.tv_sec==after.st_mtimespec.tv_sec&&before.st_mtimespec.tv_nsec==after.st_mtimespec.tv_nsec
        &&before.st_ctimespec.tv_sec==after.st_ctimespec.tv_sec&&before.st_ctimespec.tv_nsec==after.st_ctimespec.tv_nsec;
    if(!private_end_unit(at)){private_close(fd);return 0;}
    if(!ok||strcmp(hash,f->sha)){private_close(fd);return private_fail("source_changed","bound file bytes changed");}
    const struct private_dir *bound=f->root?&v->repo:&v->prefix;
    if(!private_root_current(bound)){private_close(fd);return 0;}
    int current=private_open_walk(bound->fd,f->path,0,O_RDONLY);
    if(current<0){private_close(fd);return private_fail("source_changed","bound pathname changed after hashing");}
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat named,held;
    ok=fstat(fd,&held)==0&&fstat(current,&named)==0&&private_same_file(&before,&after)&&private_same_file(&after,&held)
        &&private_same_file(&held,&named)&&held.st_nlink==1&&named.st_nlink==1;
    int paid=private_end_unit(at);int closed_current=private_close(current),closed_held=private_close(fd);
    if(!paid||!closed_current||!closed_held)return 0;
    if(!ok)return private_fail("source_changed","hashed descriptor and exact pathname differ");return 1;
}
// Borrowed spans refer to immutable retained Basis/SourceLock bytes. `path`
// includes its JSON quote/escape bytes; token equality is byte equality.
struct private_slice {const unsigned char *data;size_t len;};
struct private_ref {uint64_t len;char sha[65];struct private_slice raw,path;};
static int (*private_ref_hook)(const struct private_ref *)=NULL;
static unsigned private_ref_root=0;
static const char *const private_source_names[7] __attribute__((unused))={"git","gnupg","libgpg-error","libgcrypt","libassuan","npth","libksba"};
struct private_source_info {unsigned patch_count;char patch_sha[8][65];uint64_t patch_len[8];struct private_ref canonical_source;char bootstrap_sha[65];};
struct private_patch_span {unsigned source_index,patch_index,patch_mask;struct private_slice path;char before[65],after[65];};
struct private_patch_info {unsigned count;struct private_patch_span spans[128];struct private_slice raw;};
static __attribute__((unused)) int jarray(struct json_reader *r) {if(r->depth>=(r->limit?r->limit:6)||!jchar(r,'['))return 0;r->depth++;return 1;}
static __attribute__((unused)) int jarray_close(struct json_reader *r) {if(!r->depth||!jchar(r,']'))return 0;r->depth--;return 1;}
static int private_relative_path(const char *path) {
    if(!*path||*path=='/'||strchr(path,'\r')||strchr(path,'\n'))return 0;
    const char *p=path;while(*p){const char *e=strchr(p,'/');size_t n=e?(size_t)(e-p):strlen(p);
        if(!n||(n==1&&p[0]=='.')||(n==2&&p[0]=='.'&&p[1]=='.'))return 0;
        if(!e)return 1;p=e+1;if(!*p)return 0;
    }return 0;
}
static __attribute__((unused)) int jref(struct json_reader *r,struct private_ref *ref) {
    const unsigned char *start=r->p;char path[4097];
    if(!jobject(r)||!jkey(r,"len",1)||!ju64(r,&ref->len)||!jkey(r,"path",0))return 0;
    ref->path.data=r->p;
    if(!jstring(r,path,sizeof(path))||!private_relative_path(path))return 0;
    ref->path.len=(size_t)(r->p-ref->path.data);
    if(!jkey(r,"sha256",0)||!jhex(r,ref->sha)||!jclose(r))return 0;
    ref->raw.data=start;ref->raw.len=(size_t)(r->p-start);return private_ref_hook?private_ref_hook(ref):1;
}
static __attribute__((unused)) int private_inner_checkpoint(void) {
    uint64_t now=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    if(now<private_cost.open_cpu)return private_fail("clock_unknown","reversed inner validation clock");
    if(now-private_cost.open_cpu>=400000)return private_end_unit(private_cost.open_cpu);
    if(clock_ns(CLOCK_MONOTONIC)>=private_cost.deadline)return private_fail("deadline","original inner validation deadline expired");return 1;
}
// Canonical depth is value/key depth, with the root at zero. Scalar leaves
// count; the older reader's container counter is only a secondary stack bound.
static int private_canonical_value(struct json_reader *r,unsigned depth) {
    if(depth>10||r->p==r->end)return 0;
    unsigned c=*r->p;
    if(c=='{'||c=='[') {
        r->p++;unsigned count=0;char prior[4097]={0};
        while(r->p<r->end&&*r->p!=(c=='{'?'}':']')) {
            if(count>=64||(count&&!jchar(r,',')))return 0;
            if(c=='{') {char key[4097];if(depth==10||!jstring(r,key,sizeof(key)))return 0;
                for(size_t i=0;key[i];i++)if((unsigned char)key[i]>127)return 0;
                if((count&&strcmp(prior,key)>=0)||!jchar(r,':'))return 0;memcpy(prior,key,strlen(key)+1);
            }
            if(!private_canonical_value(r,depth+1)||!private_inner_checkpoint())return 0;count++;
        }
        return jchar(r,c=='{'?'}':']');
    }
    if(c=='"'){char value[4097];return jstring(r,value,sizeof(value));}
    const char *literal=c=='t'?"true":c=='f'?"false":c=='n'?"null":NULL;
    if(literal){size_t n=strlen(literal);if((size_t)(r->end-r->p)<n||memcmp(r->p,literal,n))return 0;r->p+=n;return 1;}
    if(c=='-'){int64_t v;return ji64(r,&v);}uint64_t v;return ju64(r,&v);
}
static int private_canonical(const unsigned char *bytes,size_t len) {
    if(!len||!private_utf8(bytes,len))return 0;struct json_reader r={bytes,bytes+len,0,10};
    return private_canonical_value(&r,0)&&r.p==r.end;
}
// Fixed scalar registry reconciles every jref in SourceLock/Basis, including
// offline provenance. It has no path resolver, file descriptor or IO effect.
struct private_ref_binding {struct private_ref ref;unsigned root;};
static struct private_ref_binding private_ref_bindings[256];
static unsigned private_ref_binding_count;
static int private_register_ref(const struct private_ref *ref) {
    for(unsigned i=0;i<private_ref_binding_count;i++) {
        const struct private_ref_binding *bound=&private_ref_bindings[i];
        if(bound->root==private_ref_root&&bound->ref.path.len==ref->path.len&&!memcmp(bound->ref.path.data,ref->path.data,ref->path.len))
            return bound->ref.len==ref->len&&!strcmp(bound->ref.sha,ref->sha);
        if(!private_inner_checkpoint())return 0;
    }
    if(private_ref_binding_count>=256)return 0;
    private_ref_bindings[private_ref_binding_count++]=(struct private_ref_binding){*ref,private_ref_root};return 1;
}
// BEGIN SOURCE_LOCK_VALIDATOR -- proof_review owns only this marked section.
// Adapted from bboxy450-harness proof-tool sources.lock.json / patches.lock.json.
// Origin: https://github.com/bboxy450/bboxy450-harness
// Retain origin/accurate reuse credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

// Fixed source pins copied from the builder's authenticated source-lock profile:
// interfaces/dream-proof-tool-qualification, revision98a3023e. They bind a
// conditional private source-review premise; they do not grant runtime approval.
struct private_source_pin {const char *version,*url,*archive,*sig_url,*signature,*key;uint64_t key_len;};
static const struct private_source_pin private_source_pins[7]={
    {"2.54.0","https://www.kernel.org/pub/software/scm/git/git-2.54.0.tar.xz","f689162364c10de79ef89aa8dbf48731eb057e34edbbd20aca510ce0154681a3","https://www.kernel.org/pub/software/scm/git/git-2.54.0.tar.sign","1083f5c8eafe129da815da3f78cbc5865bdaec29be0d3969c104d3b003801775","2a3477639ac9bff6c814a679983408c70e5320402a38d41a74b2df0b206e9bfc",51376ULL},
    {"2.5.24","https://gnupg.org/ftp/gcrypt/gnupg/gnupg-2.5.24.tar.bz2","bf149d01a2b9fcc0e4589b8ae8697d3d5c557ea48ed95a3fa55dd3b1187e6039","https://gnupg.org/ftp/gcrypt/gnupg/gnupg-2.5.24.tar.bz2.sig","6cc9dad0a8ad9a27e842c06daf696c94db5c83237933687e773c02b04857fb1a","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL},
    {"1.61","https://gnupg.org/ftp/gcrypt/libgpg-error/libgpg-error-1.61.tar.bz2","7a85413f2bc354f4f8aa832b718af122e48965e9e0eb9012ee659c13c6385c93","https://gnupg.org/ftp/gcrypt/libgpg-error/libgpg-error-1.61.tar.bz2.sig","9b20f1e6046aaa3fe16fcd65587fd0fa5e34ceb4830f8eadc856c3e80ad57ad3","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL},
    {"1.12.4","https://gnupg.org/ftp/gcrypt/libgcrypt/libgcrypt-1.12.4.tar.bz2","d77f68f48879510e79a2f65977ccc68981781ea0923e5bdffac2a193ea3d660e","https://gnupg.org/ftp/gcrypt/libgcrypt/libgcrypt-1.12.4.tar.bz2.sig","8be53f19f43ffa0297aceb520be717de539d1bdad45b5538a57fd95610b1e2db","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL},
    {"3.0.2","https://gnupg.org/ftp/gcrypt/libassuan/libassuan-3.0.2.tar.bz2","d2931cdad266e633510f9970e1a2f346055e351bb19f9b78912475b8074c36f6","https://gnupg.org/ftp/gcrypt/libassuan/libassuan-3.0.2.tar.bz2.sig","5aa3c5cea6f42bcb96be9e1dd922b33ccb05e2f0565e93674e85ddcf8cd78e86","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL},
    {"1.8","https://gnupg.org/ftp/gcrypt/npth/npth-1.8.tar.bz2","8bd24b4f23a3065d6e5b26e98aba9ce783ea4fd781069c1b35d149694e90ca3e","https://gnupg.org/ftp/gcrypt/npth/npth-1.8.tar.bz2.sig","eb9314cf3867a438cb0fdd371125d44c57ba66086f152199ad5f40a9c9679acb","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL},
    {"1.8.1","https://gnupg.org/ftp/gcrypt/libksba/libksba-1.8.1.tar.bz2","c2f84393011827219ae117131dba8e7684c2bed0961eed11b0642c2acba440b5","https://gnupg.org/ftp/gcrypt/libksba/libksba-1.8.1.tar.bz2.sig","af99f325f81eb395d740bf48dc8e89903b141c10f69244e46e4caa1caec0577e","6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375",6466ULL}
};
static const char *const private_release_roots[2]={"6DAA6E64A76D2840571B4902528897B826403ADA","AC8E115BF73E2D8D47FA9908E98E9B2D19C6C8BD"};
static const char private_git_root[]="96E07AF25771955980DAD10020D04E5A713660A7";
static const char private_git_subkey[]="E1F036B1FEE7221FC778ECEFB0B5E88696AFE6CB";
static const char private_bootstrap_sha[]="21cb9adcad96b7be404014ff6e9fb837160a2135455c217dec9efe196cf0661a";
static int private_source_expect(struct json_reader *r,const char *expected) {
    char actual[4097];
    if(!jstring(r,actual,sizeof(actual)))return private_fail("input_invalid","source tuple string syntax");
    if(strcmp(actual,expected))return private_fail("review_unbound","frozen source tuple differs");
    return 1;
}
static int private_source_fingerprints(struct json_reader *r,unsigned source,int subkey) {
    if(!jarray(r))return 0;
    unsigned count=source?2u:1u;
    for(unsigned i=0;i<count;i++) {
        if(i&&!jchar(r,','))return 0;
        const char *expected=source?private_release_roots[i]:(subkey?private_git_subkey:private_git_root);
        if(!private_source_expect(r,expected))return 0;
    }
    return jarray_close(r);
}
int private_source_lock_validate(const unsigned char *bytes,size_t len,const struct private_input *outer,struct private_source_info *info) {
    if(!len||len>32768||!private_utf8(bytes,len))return private_fail("input_invalid","source lock encoding or size");
    memset(info,0,sizeof(*info));struct json_reader r={bytes,bytes+len,0,10};
    if(!jobject(&r)||!jkey(&r,"backports",1)||!jarray(&r)||!jarray_close(&r)
        ||!jkey(&r,"bootstrap_verifier",0)||!jobject(&r)||!jkey(&r,"absolute_path",1)
        ||!private_source_expect(&r,"/opt/homebrew/Cellar/gnupg/2.5.24_1/bin/gpg")
        ||!jkey(&r,"sha256",0)||!jhex(&r,info->bootstrap_sha)||!jclose(&r))
        return private_fail("input_invalid","strict source lock bootstrap fields");
    if(strcmp(info->bootstrap_sha,private_bootstrap_sha))return private_fail("review_unbound","independent bootstrap verifier differs");
    char probe[65];
    if(!jkey(&r,"canonical_q2",0)||!jobject(&r)||!jkey(&r,"sha256",1)||!jhex(&r,probe)
        ||!jkey(&r,"source",0)||!jref(&r,&info->canonical_source)||!jclose(&r))
        return private_fail("input_invalid","strict canonical source binding");
    if(!info->canonical_source.len||strcmp(probe,outer->source)||strcmp(info->canonical_source.sha,outer->source))
        return private_fail("review_unbound","external frozen probe source binding differs");
    if(!jkey(&r,"format",0)||!private_source_expect(&r,"bboxy.dream.proof-tool-sources/1")
        ||!jkey(&r,"profile_patches",0)||!jarray(&r))return private_fail("input_invalid","strict source lock profile fields");
    while(r.p<r.end&&*r.p!=']') {
        unsigned i=info->patch_count;
        if(i>=8||(i&&!jchar(&r,',')))return private_fail("input_invalid","source profile patch array bound");
        struct private_ref ref;
        if(!jref(&r,&ref))return private_fail("input_invalid","source profile patch reference");
        memcpy(info->patch_sha[i],ref.sha,sizeof(ref.sha));info->patch_len[i]=ref.len;info->patch_count++;
        if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(&r)||!jkey(&r,"sources",0)||!jarray(&r))return private_fail("input_invalid","strict source array fields");
    for(unsigned i=0;i<7;i++) {
        const struct private_source_pin *pin=&private_source_pins[i];struct private_ref key;
        if((i&&!jchar(&r,','))||!jobject(&r)||!jkey(&r,"archive_sha256",1)||!private_source_expect(&r,pin->archive)
            ||!jkey(&r,"archive_url",0)||!private_source_expect(&r,pin->url)
            ||!jkey(&r,"compression",0)||!private_source_expect(&r,i?"bz2":"xz")
            ||!jkey(&r,"name",0)||!private_source_expect(&r,private_source_names[i])
            ||!jkey(&r,"public_key",0)||!jref(&r,&key)||!jkey(&r,"roots",0)||!private_source_fingerprints(&r,i,0)
            ||!jkey(&r,"signature_sha256",0)||!private_source_expect(&r,pin->signature)
            ||!jkey(&r,"signature_url",0)||!private_source_expect(&r,pin->sig_url)
            ||!jkey(&r,"signed_bytes",0)||!private_source_expect(&r,i?"archive":"uncompressed_tar")
            ||!jkey(&r,"signing_subkeys",0)||!private_source_fingerprints(&r,i,1)
            ||!jkey(&r,"version",0)||!private_source_expect(&r,pin->version)||!jclose(&r))
            return private_fail("input_invalid","strict seven-source row");
        if(key.len!=pin->key_len||strcmp(key.sha,pin->key))return private_fail("review_unbound","pinned public release key differs");
        if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(&r)||!jclose(&r)||r.p!=r.end||r.depth)return private_fail("input_invalid","source lock has missing extra or noncanonical fields");
    return private_inner_checkpoint();
}
int private_auth_array(struct json_reader *r,const struct private_source_info *info) {
    if(strcmp(info->bootstrap_sha,private_bootstrap_sha))return private_fail("review_unbound","source authentication lacks its pinned independent verifier");
    if(!jarray(r))return private_fail("input_invalid","source authentication array");
    for(unsigned i=0;i<7;i++) {
        char archive[65],status_sha[65],signature[65],primary[41],subkey[41],verifier[65];int64_t exit_code;
        struct private_ref status;const struct private_source_pin *pin=&private_source_pins[i];
        if((i&&!jchar(r,','))||!jobject(r)||!jkey(r,"archive_sha256",1)||!jhex(r,archive)
            ||!jkey(r,"auth_exit",0)||!ji64(r,&exit_code)||exit_code<INT32_MIN||exit_code>INT32_MAX
            ||!jkey(r,"auth_status_sha256",0)||!jhex(r,status_sha)
            ||!jkey(r,"name",0)||!private_source_expect(r,private_source_names[i])
            ||!jkey(r,"signature_sha256",0)||!jhex(r,signature)
            ||!jkey(r,"signer_primary",0)||!jstring(r,primary,sizeof(primary))
            ||!jkey(r,"signer_subkey",0)||!jstring(r,subkey,sizeof(subkey))
            ||!jkey(r,"status",0)||!jref(r,&status)||!jkey(r,"verifier_sha256",0)||!jhex(r,verifier)||!jclose(r))
            return private_fail("input_invalid","strict ordered authentication row");
        int signer=i?(!strcmp(primary,subkey)&&(!strcmp(primary,private_release_roots[0])||!strcmp(primary,private_release_roots[1])))
            :(!strcmp(primary,private_git_root)&&!strcmp(subkey,private_git_subkey));
        if(strcmp(archive,pin->archive)||strcmp(signature,pin->signature)||strcmp(verifier,info->bootstrap_sha)
            ||exit_code!=0||!status.len||strcmp(status_sha,status.sha)||!signer)
            return private_fail("review_unbound","source authentication tuple is not the pinned independent result");
        if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(r))return private_fail("input_invalid","source authentication count must be seven");
    return 1;
}
int private_generated_array(struct json_reader *r) {
    if(!jarray(r))return private_fail("input_invalid","generated tuple array");
    for(unsigned i=0;i<7;i++) {
        struct private_ref config,summary;
        if((i&&!jchar(r,','))||!jobject(r)||!jkey(r,"config",1)||!jref(r,&config)
            ||!jkey(r,"source",0)||!private_source_expect(r,private_source_names[i])
            ||!jkey(r,"summary",0)||!jref(r,&summary)||!jclose(r))
            return private_fail("input_invalid","strict ordered generated tuple");
        if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(r))return private_fail("input_invalid","generated tuple count must be seven");
    return 1;
}
static int private_source_versioned(const char *source) {
    char expected[257];
    for(unsigned i=0;i<7;i++) {
        int n=snprintf(expected,sizeof(expected),"%s-%s",private_source_names[i],private_source_pins[i].version);
        if(n>0&&(size_t)n<sizeof(expected)&&!strcmp(source,expected))return (int)i;
    }
    return -1;
}
static int private_source_same_slice(struct private_slice a,struct private_slice b) {
    return a.len==b.len&&!memcmp(a.data,b.data,a.len);
}
int private_patch_array(struct json_reader *r,const struct private_source_info *info,const struct private_ref *patch_lock,struct private_patch_info *patches) {
    const unsigned char *start=r->p;memset(patches,0,sizeof(*patches));
    if(info->patch_count>8)return private_fail("input_invalid","selected patch count exceeds source profile bound");
    if(!jarray(r))return private_fail("input_invalid","patch lock array");
    for(unsigned index=0;index<info->patch_count;index++) {
        struct private_patch_span files[16];unsigned count=0;char patch[4097],sha[65],source[257];
        if((index&&!jchar(r,','))||!jobject(r)||!jkey(r,"files",1)||!jarray(r))
            return private_fail("input_invalid","strict patch row");
        while(r->p<r->end&&*r->p!=']') {
            if(count>=16)return private_fail("input_invalid","patch file array bound");
            struct private_patch_span *file=&files[count];char path[4097];
            if((count&&!jchar(r,','))||!jobject(r)||!jkey(r,"after",1)||!jhex(r,file->after)
                ||!jkey(r,"before",0)||!jhex(r,file->before)||!jkey(r,"path",0))
                return private_fail("input_invalid","patch file array bound or syntax");
            file->path.data=r->p;
            if(!jstring(r,path,sizeof(path))||!private_path(path,0))return private_fail("input_invalid","safe patch source path");
            file->path.len=(size_t)(r->p-file->path.data);
            if(!jclose(r))return private_fail("input_invalid","strict patch file row");
            for(unsigned j=0;j<count;j++)if(private_source_same_slice(files[j].path,file->path))
                return private_fail("review_unbound","duplicate path within one patch");
            count++;if(!private_inner_checkpoint())return 0;
        }
        if(!jarray_close(r)||!jkey(r,"patch",0)||!jstring(r,patch,sizeof(patch))||!private_path(patch,0)
            ||!jkey(r,"sha256",0)||!jhex(r,sha)||!jkey(r,"source",0)||!jstring(r,source,sizeof(source))||!jclose(r))
            return private_fail("input_invalid","strict existing patches.lock row shape");
        int source_index=private_source_versioned(source);
        if(source_index<0||strcmp(sha,info->patch_sha[index]))return private_fail("review_unbound","selected patch source hash or order differs");
        for(unsigned j=0;j<count;j++) {
            struct private_patch_span *prior=NULL;
            for(unsigned k=0;k<patches->count;k++)if(patches->spans[k].source_index==(unsigned)source_index
                &&private_source_same_slice(patches->spans[k].path,files[j].path)){prior=&patches->spans[k];break;}
            if(prior) {
                if(prior->patch_index==index||strcmp(prior->after,files[j].before))
                    return private_fail("review_unbound","postimage patch chain does not bind its exact preimage");
                memcpy(prior->after,files[j].after,sizeof(prior->after));prior->patch_index=index;prior->patch_mask|=1u<<index;
            } else {
                if(patches->count>=128)return private_fail("input_invalid","patch postimage map bound");
                files[j].source_index=(unsigned)source_index;files[j].patch_index=index;files[j].patch_mask=1u<<index;
                patches->spans[patches->count++]=files[j];
            }
        }
        if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(r))return private_fail("input_invalid","patch count differs from source lock");
    patches->raw.data=start;patches->raw.len=(size_t)(r->p-start);char digest[65];
    private_hash(start,patches->raw.len,digest);
    if(patch_lock->len!=patches->raw.len||strcmp(patch_lock->sha,digest))
        return private_fail("review_unbound","patch_lock reference does not bind exact canonical patch array");
    return private_inner_checkpoint();
}

// END SOURCE_LOCK_VALIDATOR

// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Prepared canonical-C section only. It is not another executable/controller.
// Inner ICD98a3023e, streaming borrowed representations and fixed profile65126.
struct private_platform {
    char machine[9],release[129],os_build[129],cache_uuid[33];
};
struct private_abs {struct private_ref ref;uint64_t dev,ino;int64_t mtime_sec;uint32_t mtime_nsec;};
struct private_macho {uint32_t cpu,subtype;char uuid[33];struct private_slice rpaths,loads;unsigned rpath_count,load_count;};
struct private_node {struct private_slice id,install_name;unsigned kind,source,nullable;struct private_ref file,member_index;struct private_macho macho;char platform_uuid[33];};
struct private_load {struct private_slice name,target;unsigned mode,cache;};
struct private_link_map {unsigned tool,count;struct private_ref file;char output_sha[65];struct private_slice output,inputs;};
struct private_fixture {
    struct private_ref files[32],keys,payload,valid_signature,corrupt_signature;unsigned file_count;
    char ancestor[41],valid[41],corrupt[41],blob[41],signer[41];
    struct private_slice origin,policy_branch,record;
};
struct private_build {
    char id[27],commit[41],source_sha[65],reported_target[257],sdk_version[257];uint64_t incarnation;
    struct private_ref source_lock,patch_lock,recipe_file,recipe_commands,recipe_env,compiler_version,linker_version,sdk_settings;
    struct private_abs compiler,linker,tools[2];struct private_slice sdk_path;
    struct private_platform platform;struct private_patch_info patches;
};
struct private_basis {
    struct private_slice raw,build_raw,closure_raw,commands_raw,config_raw,fixture_raw;
    struct private_source_info source;struct private_build build;
    struct private_platform platform;struct private_fixture fixture;
    struct private_ref gpg_conf,common_conf;
    struct private_node nodes[64];struct private_link_map links[64];
    unsigned node_count,image_count,link_count,edge_count,source_coverage,command_coverage;
};
static int private_text(struct json_reader *r,struct private_slice *span,size_t max,int nonempty) {
    char text[4097];const unsigned char *start=r->p;
    if(max>4096||!jstring(r,text,max+1)||(nonempty&&!text[0]))return 0;
    span->data=start;span->len=(size_t)(r->p-start);return 1;
}
static int private_token_text(struct private_slice token,char *out,size_t cap) {
    struct json_reader r={token.data,token.data+token.len,0,10};
    return jstring(&r,out,cap)&&r.p==r.end;
}
static int private_token_same(struct private_slice a,struct private_slice b) {
    return a.len==b.len&&memcmp(a.data,b.data,a.len)==0;
}
static int private_literal(struct json_reader *r,const char *expected) {
    char text[4097];size_t n=strlen(expected);
    return n<=4096&&jstring(r,text,n+1)&&!strcmp(text,expected);
}
static int private_sha40(struct json_reader *r,char out[41]) {return jstring(r,out,41)&&hex_value(out,40);}
static int private_uuid(struct json_reader *r,char out[33]) {return jstring(r,out,33)&&hex_value(out,32)&&strcmp(out,"00000000000000000000000000000000");}
static int private_fingerprint(struct json_reader *r,char out[41]) {
    if(!jstring(r,out,41)||strlen(out)!=40)return 0;
    for(unsigned i=0;i<40;i++)if(!((out[i]>='0'&&out[i]<='9')||(out[i]>='A'&&out[i]<='F')))return 0;return 1;
}
static int private_source_index(struct json_reader *r,unsigned *out) {
    char name[129];if(!jstring(r,name,sizeof(name)))return 0;
    for(unsigned i=0;i<7;i++)if(!strcmp(name,private_source_names[i])){*out=i;return 1;}return 0;
}
static int private_decimal_version(const char *s,unsigned min_dots,unsigned max_dots) {
    unsigned dots=0,digits=0;for(;*s;s++){if(*s=='.'){if(!digits)return 0;dots++;digits=0;}else if(*s>='0'&&*s<='9')digits++;else return 0;}
    return digits&&dots>=min_dots&&dots<=max_dots;
}
static int private_os_build(const char *s) {
    size_t n=strlen(s);if(n<4||s[0]<'0'||s[0]>'9'||s[1]<'0'||s[1]>'9'||s[2]<'A'||s[2]>'Z')return 0;
    unsigned digits=0;size_t i=3;while(s[i]>='0'&&s[i]<='9'){digits++;i++;}
    return digits>=1&&digits<=6&&(!s[i]||(s[i]>='a'&&s[i]<='z'&&!s[i+1]));
}
static int private_platform_read(struct json_reader *r,struct private_platform *p) {
    return jobject(r)&&jkey(r,"arch",1)&&private_literal(r,"aarch64")
        &&jkey(r,"darwin_machine",0)&&jstring(r,p->machine,sizeof(p->machine))&&(!strcmp(p->machine,"arm64")||!strcmp(p->machine,"aarch64"))
        &&jkey(r,"os_build",0)&&jstring(r,p->os_build,sizeof(p->os_build))&&private_os_build(p->os_build)
        &&jkey(r,"premise",0)&&private_literal(r,"trusted_platform_boundary/1")
        &&jkey(r,"release",0)&&jstring(r,p->release,sizeof(p->release))&&private_decimal_version(p->release,1,2)
        &&jkey(r,"shared_cache_uuid",0)&&private_uuid(r,p->cache_uuid)&&jclose(r);
}
static int private_platform_same(const struct private_platform *a,const struct private_platform *b) {
    return !strcmp(a->machine,b->machine)&&!strcmp(a->release,b->release)&&!strcmp(a->os_build,b->os_build)&&!strcmp(a->cache_uuid,b->cache_uuid);
}
static int private_abs_read(struct json_reader *r,struct private_abs *a) {
    const unsigned char *start=r->p;char path[4097];
    if(!(jobject(r)&&jkey(r,"dev",1)&&ju64(r,&a->dev)&&jkey(r,"ino",0)&&ju64(r,&a->ino)
        &&jkey(r,"len",0)&&ju64(r,&a->ref.len)&&jkey(r,"mtime_nsec",0)&&j32(r,&a->mtime_nsec)&&a->mtime_nsec<1000000000
        &&jkey(r,"mtime_sec",0)&&ji64(r,&a->mtime_sec)&&jkey(r,"path",0)))return 0;
    a->ref.path.data=r->p;if(!jstring(r,path,sizeof(path))||!private_path(path,1))return 0;
    a->ref.path.len=(size_t)(r->p-a->ref.path.data);
    if(!jkey(r,"sha256",0)||!jhex(r,a->ref.sha)||!jclose(r))return 0;
    a->ref.raw.data=start;a->ref.raw.len=(size_t)(r->p-start);return 1;
}
static int private_string_array(struct json_reader *r,struct private_slice *raw,unsigned *count,const char *const *fixed,unsigned fixed_count) {
    const unsigned char *start=r->p;unsigned n=0;if(!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {
        if(n>=64||(n&&!jchar(r,',')))return 0;
        struct private_slice s;if(fixed){if(n>=fixed_count||!private_literal(r,fixed[n]))return 0;}else if(!private_text(r,&s,4096,1))return 0;
        n++;if(!private_inner_checkpoint())return 0;
    }
    if(!jarray_close(r)||(fixed&&n!=fixed_count))return 0;
    raw->data=start;raw->len=(size_t)(r->p-start);*count=n;return 1;
}
struct private_command_form {const char *id,*tool,*input;unsigned members,argc;const char *argv[5];};
static const struct private_command_form private_forms[16]={
    {"root_bare","git","none",1,2,{"rev-parse","--is-bare-repository"}},
    {"root_top","git","none",1,2,{"rev-parse","--show-toplevel"}},
    {"policy_main","git","none",1,3,{"rev-parse","--verify","{policy_branch}"}},
    {"origin_main","git","none",1,3,{"rev-parse","--verify","refs/remotes/origin/main"}},
    {"object_type","git","none",1,3,{"cat-file","-t","{commit}"}},
    {"ancestry","git","none",1,4,{"merge-base","--is-ancestor","{ancestor}","{commit}"}},
    {"tree_record","git","none",1,5,{"ls-tree","-z","{commit}","--","{record}"}},
    {"show_record","git","none",1,2,{"show","{commit}:{record}"}},
    {"last_touch","git","none",1,5,{"rev-list","-1","{commit}","--","{record}"}},
    {"blob_pin","git","none",1,2,{"rev-parse","{commit}:{record}"}},
    {"tree_pin","git","none",1,2,{"rev-parse","{commit}^{tree}"}},
    {"origin_url","git","none",1,3,{"config","--get","remote.origin.url"}},
    {"signature","git","none",2,4,{"log","-1","--format=%G?%x00%GF%x00%GP","{commit}"}},
    {"show_keys","gpg","public_keys",1,3,{"--with-colons","--show-keys","-"}},
    {"import_keys","gpg","public_keys",1,2,{"--import","-"}},
    {"verify_child","gpg","commit_payload",1,5,{"--keyid-format=long","--status-fd=1","--verify","{git_signature_temp}","-"}}
};
static const char *const private_git_prefix[]={"--no-pager","--no-optional-locks","-c","core.fsmonitor=false","-c","core.hooksPath=/dev/null","-c","credential.helper=","-c","protocol.allow=never","-c","gpg.format=openpgp","-c","gpg.program={gpg}","-c","gpg.openpgp.program={gpg}"};
static const char *const private_gpg_prefix[]={"--no-options","--no-autostart","--batch","--no-tty"};
static const char *const private_env_names[]={"GIT_CONFIG_NOSYSTEM","GIT_CONFIG_GLOBAL","GIT_TERMINAL_PROMPT","GIT_ASKPASS","SSH_ASKPASS","LANG","LC_ALL","PATH","HOME","GNUPGHOME","TMPDIR"};
static const char *const private_env_values[]={"1","/dev/null","0","/usr/bin/false","/usr/bin/false","C","C","/usr/bin:/bin","{home}","{home}","{scratch}"};
static int private_commands_read(struct json_reader *r) {
    struct private_slice raw;unsigned count;
    if(!jobject(r)||!jkey(r,"forms",1)||!jarray(r))return 0;
    for(unsigned i=0;i<16;i++) {
        uint64_t members;const struct private_command_form *f=&private_forms[i];
        if((i&&!jchar(r,','))||!jobject(r)||!jkey(r,"argv",1)||!private_string_array(r,&raw,&count,f->argv,f->argc)
            ||!jkey(r,"id",0)||!private_literal(r,f->id)||!jkey(r,"members",0)||!ju64(r,&members)||members!=f->members
            ||!jkey(r,"stdin",0)||!private_literal(r,f->input)||!jkey(r,"tool",0)||!private_literal(r,f->tool)||!jclose(r)||!private_inner_checkpoint())return 0;
    }
    return jarray_close(r)&&jkey(r,"git_prefix",0)&&private_string_array(r,&raw,&count,private_git_prefix,16)
        &&jkey(r,"gpg_prefix",0)&&private_string_array(r,&raw,&count,private_gpg_prefix,4)
        &&jkey(r,"profile",0)&&private_literal(r,"65126")&&jclose(r);
}
static int private_config_read(struct json_reader *r,struct private_basis *b) {
    if(!jobject(r)||!jkey(r,"common_conf",1)||!jref(r,&b->common_conf)||!jkey(r,"env",0)||!jarray(r))return 0;
    for(unsigned i=0;i<11;i++)if((i&&!jchar(r,','))||!jobject(r)||!jkey(r,"name",1)||!private_literal(r,private_env_names[i])
        ||!jkey(r,"value",0)||!private_literal(r,private_env_values[i])||!jclose(r))return 0;
    if(!jarray_close(r)||!jkey(r,"gpg_conf",0)||!jref(r,&b->gpg_conf)||!jclose(r))return 0;
    const char *config="batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n";
    char empty_sha[65],config_sha[65];private_hash("",0,empty_sha);private_hash(config,strlen(config),config_sha);
    return !b->common_conf.len&&!strcmp(b->common_conf.sha,empty_sha)&&b->gpg_conf.len==strlen(config)&&!strcmp(b->gpg_conf.sha,config_sha);
}
static int private_ref_outer(const struct private_ref *ref,const struct private_file *outer) {
    char path[4097];return private_token_text(ref->path,path,sizeof(path))&&!strcmp(path,outer->path)&&ref->len==outer->len&&!strcmp(ref->sha,outer->sha);
}
static int private_abs_outer(const struct private_abs *a,const struct private_input *outer,unsigned tool) {
    const struct private_file *f=tool?&outer->gpg:&outer->git;const char *relative=tool?"bin/gpg":"bin/git";
    char path[4097],expected[4097];int n=snprintf(expected,sizeof(expected),"%s/%s",outer->prefix.path,relative);
    return !f->root&&!strcmp(f->path,relative)&&n>0&&(size_t)n<sizeof(expected)&&private_token_text(a->ref.path,path,sizeof(path))&&!strcmp(path,expected)
        &&a->ref.len==f->len&&!strcmp(a->ref.sha,f->sha)&&a->dev==f->dev&&a->ino==f->ino&&a->mtime_sec==f->mtime_sec&&a->mtime_nsec==f->mtime_nsec;
}
static int private_build_read(struct json_reader *r,struct private_basis *b,const struct private_input *outer) {
    struct private_build *v=&b->build;char target[257],canonical_name[263],sdk_path[4097];
    if(!(jobject(r)&&jkey(r,"auth",1)&&private_auth_array(r,&b->source)&&jkey(r,"build_id",0)&&jstring(r,v->id,sizeof(v->id))&&julid(v->id)
        &&jkey(r,"builder",0)&&jobject(r)&&jkey(r,"commit",1)&&private_sha40(r,v->commit)&&jkey(r,"source_sha256",0)&&jhex(r,v->source_sha)&&jclose(r)
        &&jkey(r,"compiler",0)&&jobject(r)&&jkey(r,"image",1)&&private_abs_read(r,&v->compiler)
        &&jkey(r,"reported_target",0)&&jstring(r,v->reported_target,sizeof(v->reported_target))
        &&jkey(r,"target",0)&&private_literal(r,"aarch64-apple-darwin")&&jkey(r,"version",0)&&jref(r,&v->compiler_version)&&jclose(r)
        &&jkey(r,"generated",0)&&private_generated_array(r)&&jkey(r,"incarnation",0)&&ju64(r,&v->incarnation)&&v->incarnation>0
        &&jkey(r,"linker",0)&&jobject(r)&&jkey(r,"image",1)&&private_abs_read(r,&v->linker)&&jkey(r,"version",0)&&jref(r,&v->linker_version)&&jclose(r)
        &&jkey(r,"patch_lock",0)&&jref(r,&v->patch_lock)&&jkey(r,"patches",0)&&private_patch_array(r,&b->source,&v->patch_lock,&v->patches)
        &&jkey(r,"platform",0)&&private_platform_read(r,&v->platform)&&jkey(r,"recipe",0)&&jobject(r)
        &&jkey(r,"commands",1)&&jref(r,&v->recipe_commands)&&jkey(r,"env",0)&&jref(r,&v->recipe_env)&&jkey(r,"file",0)&&jref(r,&v->recipe_file)&&jclose(r)
        &&jkey(r,"sdk",0)&&jobject(r)&&jkey(r,"canonical_name",1)&&jstring(r,canonical_name,sizeof(canonical_name))&&jkey(r,"path",0)))return 0;
    const unsigned char *start=r->p;if(!jstring(r,sdk_path,sizeof(sdk_path))||!private_path(sdk_path,1))return 0;
    v->sdk_path=(struct private_slice){start,(size_t)(r->p-start)};
    if(!(jkey(r,"settings",0)&&jref(r,&v->sdk_settings)&&jkey(r,"version",0)&&jstring(r,v->sdk_version,sizeof(v->sdk_version))&&private_decimal_version(v->sdk_version,1,2)&&jclose(r)
        &&jkey(r,"source_lock",0)&&jref(r,&v->source_lock)&&jkey(r,"tools",0)&&jobject(r)&&jkey(r,"git",1)&&private_abs_read(r,&v->tools[0])
        &&jkey(r,"gpg",0)&&private_abs_read(r,&v->tools[1])&&jclose(r)&&jclose(r)))return 0;
    int n=snprintf(target,sizeof(target),"macosx%s",v->sdk_version);
    if(n<=0||(size_t)n>=sizeof(target)||strcmp(target,canonical_name)||strcmp(v->recipe_file.sha,v->source_sha)||!private_ref_outer(&v->source_lock,&outer->source_lock)
        ||!private_abs_outer(&v->tools[0],outer,0)||!private_abs_outer(&v->tools[1],outer,1))return 0;
    const char *release=NULL;
    if(!strncmp(v->reported_target,"arm64-apple-darwin",18))release=v->reported_target+18;
    else if(!strncmp(v->reported_target,"aarch64-apple-darwin",20))release=v->reported_target+20;
    return release&&private_decimal_version(release,0,2)&&!strcmp(release,v->platform.release)&&!strcmp(v->platform.os_build,outer->os_build);
}
static int private_no_git_component(const char *path) {
    const char *p=path;while(*p){const char *end=strchr(p,'/');size_t n=end?(size_t)(end-p):strlen(p);if(n==4&&!memcmp(p,".git",4))return 0;if(!end)break;p=end+1;}return 1;
}
static int private_fixture_inventory_has(const struct private_fixture *f,const struct private_ref *ref) {
    for(unsigned i=0;i<f->file_count;i++)if(private_token_same(f->files[i].raw,ref->raw))return 1;return 0;
}
static int private_fixture_read_raw(struct json_reader *r,struct private_basis *b,const struct private_input *outer) {
    struct private_fixture *f=&b->fixture;struct private_dir repo;char prior[4097]={0},path[4097];
    if(!(jobject(r)&&jkey(r,"ancestor",1)&&private_sha40(r,f->ancestor)&&jkey(r,"corrupt_commit",0)&&private_sha40(r,f->corrupt)
        &&jkey(r,"corrupt_signature",0)&&jref(r,&f->corrupt_signature)&&jkey(r,"files",0)&&jarray(r)))return 0;
    f->file_count=0;
    while(r->p<r->end&&*r->p!=']') {
        unsigned i=f->file_count;if(i>=32||(i&&!jchar(r,','))||!jref(r,&f->files[i])||!private_token_text(f->files[i].path,path,sizeof(path))||(i&&strcmp(prior,path)>=0))return 0;
        memcpy(prior,path,strlen(path)+1);f->file_count++;if(!private_inner_checkpoint())return 0;
    }
    if(!(jarray_close(r)&&jkey(r,"origin_url",0)&&private_text(r,&f->origin,4096,1)&&jkey(r,"policy_branch",0)&&private_text(r,&f->policy_branch,4096,1)
        &&jkey(r,"public_keys",0)&&jref(r,&f->keys)&&jkey(r,"record_blob",0)&&private_sha40(r,f->blob)&&jkey(r,"record_path",0)&&private_text(r,&f->record,4096,1)
        &&jkey(r,"repo",0)&&jdir(r,&repo)&&jkey(r,"signer_primary",0)&&private_fingerprint(r,f->signer)
        &&jkey(r,"valid_commit",0)&&private_sha40(r,f->valid)&&jkey(r,"valid_payload",0)&&jref(r,&f->payload)&&jkey(r,"valid_signature",0)&&jref(r,&f->valid_signature)&&jclose(r)))return 0;
    if(repo.dev!=outer->repo.dev||repo.ino!=outer->repo.ino||repo.uid!=outer->repo.uid||strcmp(repo.path,outer->repo.path)
        ||strcmp(f->valid,outer->valid_commit)||strcmp(f->corrupt,outer->corrupt_commit)||!strcmp(f->valid,f->corrupt)||!private_ref_outer(&f->keys,&outer->keys)
        ||!private_fixture_inventory_has(f,&f->keys)||!private_fixture_inventory_has(f,&f->payload)||!private_fixture_inventory_has(f,&f->valid_signature)||!private_fixture_inventory_has(f,&f->corrupt_signature))return 0;
    if(!private_token_text(f->policy_branch,path,sizeof(path))||strncmp(path,"refs/heads/",11)||!private_path(path,0)
        ||!private_token_text(f->record,path,sizeof(path))||!private_path(path,0)||!private_no_git_component(path)
        ||!private_token_text(f->origin,path,sizeof(path))||strchr(path,'@')||strstr(path,"://")==path)return 0;
    return f->file_count&&f->keys.len&&f->payload.len&&f->valid_signature.len&&f->corrupt_signature.len;
}
static int private_fixture_read(struct json_reader *r,struct private_basis *b,const struct private_input *outer) {
    unsigned prior=private_ref_root;private_ref_root=1;
    int result=private_fixture_read_raw(r,b,outer);private_ref_root=prior;return result;
}
// Shared C codec authority: signed builder b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9.
// Schema d70948ec375dae099ea1dac0b92dfb30ff87d7fcac94c98f42caa1c0ba2da8ec.
// Offline provenance is scalar metadata here; parsing never opens a Ref payload.
static int private_path_text(struct json_reader *r,struct private_slice *v) {
    char text[4097];const unsigned char *start=r->p;
    if(!jstring(r,text,sizeof(text))||!text[0]||strchr(text,'\r')||strchr(text,'\n'))return 0;
    *v=(struct private_slice){start,(size_t)(r->p-start)};return 1;
}
static int private_target_read(struct json_reader *r,struct private_load *load) {
    if(!jobject(r))return 0;
    if(r->end-r->p>=5&&!memcmp(r->p,"\"id\":",5)) {
        load->cache=0;
        return jkey(r,"id",1)&&private_text(r,&load->target,128,1)&&jkey(r,"kind",0)&&private_literal(r,"candidate")&&jclose(r);
    }
    load->cache=1;load->target=(struct private_slice){0};
    return jkey(r,"kind",1)&&private_literal(r,"cache")&&jclose(r);
}
static int private_load_read(struct json_reader *r,struct private_load *load) {
    char mode[16];const char *const modes[]={"load","weak","reexport","upward","lazy"};
    if(!jobject(r)||!jkey(r,"mode",1)||!jstring(r,mode,sizeof(mode)))return 0;
    load->mode=5;for(unsigned i=0;i<5;i++)if(!strcmp(mode,modes[i]))load->mode=i;
    return load->mode<5&&jkey(r,"name",0)&&private_path_text(r,&load->name)
        &&jkey(r,"target",0)&&private_target_read(r,load)&&jclose(r);
}
static int private_macho_read(struct json_reader *r,struct private_macho *m) {
    if(!(jobject(r)&&jkey(r,"cpu_subtype",1)&&j32(r,&m->subtype)&&jkey(r,"cpu_type",0)&&j32(r,&m->cpu)&&m->cpu==16777228
        &&jkey(r,"fat",0)&&r->end-r->p>=5&&!memcmp(r->p,"false",5)&&(r->p+=5)&&jkey(r,"loads",0)))return 0;
    const unsigned char *start=r->p;m->load_count=0;if(!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {struct private_load load;
        if(m->load_count>=64||(m->load_count&&!jchar(r,','))||!private_load_read(r,&load)||!private_inner_checkpoint())return 0;m->load_count++;
    }
    if(!jarray_close(r))return 0;m->loads=(struct private_slice){start,(size_t)(r->p-start)};
    if(!jkey(r,"rpaths",0))return 0;start=r->p;m->rpath_count=0;if(!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {struct private_slice path;
        if(m->rpath_count>=64||(m->rpath_count&&!jchar(r,','))||!private_path_text(r,&path)||!private_inner_checkpoint())return 0;m->rpath_count++;
    }
    if(!jarray_close(r))return 0;m->rpaths=(struct private_slice){start,(size_t)(r->p-start)};
    return jkey(r,"uuid",0)&&private_uuid(r,m->uuid)&&jclose(r);
}
static int private_node_read(struct json_reader *r,struct private_node *n) {
    char kind[16];n->nullable=0;
    if(!(jobject(r)&&jkey(r,"file",1)&&jref(r,&n->file)&&jkey(r,"id",0)&&private_text(r,&n->id,128,1)&&jkey(r,"install_name",0)))return 0;
    if(r->end-r->p>=4&&!memcmp(r->p,"null",4)){r->p+=4;n->nullable=2;n->install_name=(struct private_slice){0};}
    else if(!private_path_text(r,&n->install_name))return 0;
    if(!(jkey(r,"kind",0)&&jstring(r,kind,sizeof(kind))&&jkey(r,"macho",0)&&private_macho_read(r,&n->macho)
        &&jkey(r,"source",0)&&private_source_index(r,&n->source)&&jclose(r)))return 0;
    if(!strcmp(kind,"binary")){n->kind=0;return n->nullable==2&&n->macho.subtype==0;}
    if(!strcmp(kind,"dylib")){n->kind=1;return !n->nullable;}return 0;
}
static int private_archive_read(struct json_reader *r,struct private_node *n) {
    n->kind=2;
    return jobject(r)&&jkey(r,"file",1)&&jref(r,&n->file)&&jkey(r,"id",0)&&private_text(r,&n->id,128,1)
        &&jkey(r,"member_index",0)&&jref(r,&n->member_index)&&jkey(r,"source",0)&&private_source_index(r,&n->source)&&jclose(r);
}
static int private_static_use_read(struct json_reader *r,struct private_slice *id,uint32_t *count,char hash[65]) {
    return jobject(r)&&jkey(r,"id",1)&&private_text(r,id,128,1)&&jkey(r,"member_count",0)&&j32(r,count)&&*count
        &&jkey(r,"members_sha256",0)&&jhex(r,hash)&&jclose(r);
}
static int private_link_read(struct json_reader *r,struct private_link_map *m) {
    if(!jobject(r)||!jkey(r,"file",1)||!jref(r,&m->file)||!jkey(r,"output",0)||!private_text(r,&m->output,128,1)
        ||!jkey(r,"output_sha256",0)||!jhex(r,m->output_sha)||!jkey(r,"statics",0))return 0;
    const unsigned char *start=r->p;m->count=0;if(!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {struct private_slice id;uint32_t count;char hash[65];
        if(m->count>=64||(m->count&&!jchar(r,','))||!private_static_use_read(r,&id,&count,hash)||!private_inner_checkpoint())return 0;m->count++;
    }
    if(!jarray_close(r))return 0;m->inputs=(struct private_slice){start,(size_t)(r->p-start)};
    char tool[4];if(!jkey(r,"tool",0)||!jstring(r,tool,sizeof(tool))||!jclose(r))return 0;
    if(!strcmp(tool,"git"))m->tool=0;else if(!strcmp(tool,"gpg"))m->tool=1;else return 0;return 1;
}
static int private_node_index(const struct private_basis *b,struct private_slice id) {
    for(unsigned i=0;i<b->node_count;i++)if(private_token_same(b->nodes[i].id,id))return (int)i;return -1;
}
static int private_id_order(struct private_slice a,struct private_slice b) {
    char x[129],y[129];if(!private_token_text(a,x,sizeof(x))||!private_token_text(b,y,sizeof(y)))return 0;return strcmp(x,y);
}
static int private_closure_read(struct json_reader *r,struct private_basis *b) {
    b->node_count=b->image_count=b->link_count=b->edge_count=0;
    if(!jobject(r)||!jkey(r,"images",1)||!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {
        unsigned i=b->node_count;if(i>=64||(i&&!jchar(r,','))||!private_node_read(r,&b->nodes[i]))return 0;
        if(private_node_index(b,b->nodes[i].id)>=0||(i>2&&private_id_order(b->nodes[i-1].id,b->nodes[i].id)>=0))return 0;
        b->node_count++;if(!private_inner_checkpoint())return 0;
    }
    b->image_count=b->node_count;
    if(b->image_count<2||!jarray_close(r)||!jkey(r,"link_maps",0)||!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {unsigned i=b->link_count;
        if(i>=64||(i&&!jchar(r,','))||!private_link_read(r,&b->links[i])||!private_inner_checkpoint())return 0;b->link_count++;
    }
    const char *const roots[]={"git","gpg"};struct private_slice raw;unsigned count;
    if(b->link_count<2||!jarray_close(r)||!jkey(r,"platform",0)||!private_platform_read(r,&b->platform)
        ||!jkey(r,"roots",0)||!private_string_array(r,&raw,&count,roots,2)||!jkey(r,"statics",0)||!jarray(r))return 0;
    while(r->p<r->end&&*r->p!=']') {unsigned i=b->node_count;
        if(i>=64||(i>b->image_count&&!jchar(r,','))||!private_archive_read(r,&b->nodes[i])||private_node_index(b,b->nodes[i].id)>=0
            ||(i>b->image_count&&private_id_order(b->nodes[i-1].id,b->nodes[i].id)>=0)||!private_inner_checkpoint())return 0;
        b->node_count++;
    }
    return jarray_close(r)&&jclose(r)&&private_platform_same(&b->platform,&b->build.platform);
}
static int private_command_index(struct json_reader *r,unsigned *out) {
    char name[129];if(!jstring(r,name,sizeof(name)))return 0;
    for(unsigned i=0;i<16;i++)if(!strcmp(name,private_forms[i].id)){*out=i;return 1;}return 0;
}
static int private_projection_hash(struct private_slice projection,const char *expected) {
    char hash[65];private_hash(projection.data,projection.len,hash);return !strcmp(hash,expected);
}
static int private_review_read(struct json_reader *r,struct private_basis *b) {
    char hashes[5][65];const char *const keys[]={"build","closure","commands","config","fixture"};
    const struct private_slice slices[]={b->build_raw,b->closure_raw,b->commands_raw,b->config_raw,b->fixture_raw};
    if(!jobject(r)||!jkey(r,"bindings",1)||!jobject(r))return 0;
    for(unsigned i=0;i<5;i++)if(!jkey(r,keys[i],i==0)||!jhex(r,hashes[i])||!private_projection_hash(slices[i],hashes[i]))return 0;
    if(!jclose(r)||!jkey(r,"findings",0)||!jarray(r))return 0;
    const char *const ids[]={"git_async","git_child","git_shell","gpg_service","gpg_thread","libraries","input_config","platform"};
    const char *const guards[]={"deny_async","one_verify_child","deny_shell","deny_service","deny_thread","review_only","fixed_inputs","review_only"};
    const unsigned required_sources[]={1u,(1u|2u),1u,(2u|4u|16u),(2u|8u|32u),(4u|8u|16u|32u|64u),(1u|2u),(1u|2u)};
    const char *const phases[]={"entry","normal","error","cleanup","initializer"};
    unsigned seen=0;b->source_coverage=0;b->command_coverage=0;
    for(unsigned i=0;i<8;i++) {
        if((i&&!jchar(r,','))||!jobject(r)||!jkey(r,"commands",1)||!jarray(r))return 0;
        unsigned commands=0,count=0;
        while(r->p<r->end&&*r->p!=']'){unsigned index;if(count>=16||(count&&!jchar(r,','))||!private_command_index(r,&index)||(commands&(1u<<index)))return 0;commands|=1u<<index;count++;}
        if(!commands||!jarray_close(r)||!jkey(r,"finding",0)||!private_literal(r,"conditional_source_review")||!jkey(r,"guard",0))return 0;
        char guard[129],id[129];if(!jstring(r,guard,sizeof(guard))||!jkey(r,"id",0)||!jstring(r,id,sizeof(id)))return 0;
        unsigned which=8;for(unsigned n=0;n<8;n++)if(!strcmp(id,ids[n]))which=n;
        if(which==8||(seen&(1u<<which))||strcmp(guard,guards[which])||!jkey(r,"patches",0)||!jarray(r))return 0;seen|=1u<<which;
        unsigned patches=0;count=0;
        while(r->p<r->end&&*r->p!=']') {
            char sha[65];if(count>=8||(count&&!jchar(r,','))||!jhex(r,sha))return 0;
            unsigned n=b->source.patch_count;for(unsigned k=0;k<b->source.patch_count;k++)if(!strcmp(sha,b->source.patch_sha[k]))n=k;
            if(n>=b->source.patch_count||(patches&(1u<<n)))return 0;patches|=1u<<n;count++;
        }
        struct private_slice raw;unsigned phase_count;
        if(!jarray_close(r)||!jkey(r,"phases",0)||!private_string_array(r,&raw,&phase_count,phases,5)
            ||!jkey(r,"rationale",0)||!private_text(r,&raw,512,1)||!jkey(r,"sources",0)||!jarray(r))return 0;
        unsigned sources=0;count=0;
        while(r->p<r->end&&*r->p!=']'){unsigned index;if(count>=7||(count&&!jchar(r,','))||!private_source_index(r,&index)||(sources&(1u<<index)))return 0;sources|=1u<<index;count++;}
        if(!jarray_close(r)||(sources&required_sources[which])!=required_sources[which]||!jkey(r,"spans",0)||!jarray(r))return 0;
        unsigned span_sources=0,applicable_patches=0;count=0;
        while(r->p<r->end&&*r->p!=']') {
            uint32_t first,last;unsigned source;struct private_slice path;char text[4097],sha[65];
            if(count>=16||(count&&!jchar(r,','))||!jobject(r)||!jkey(r,"first",1)||!j32(r,&first)||!first
                ||!jkey(r,"last",0)||!j32(r,&last)||last<first||!jkey(r,"path",0)||!private_text(r,&path,4096,1)
                ||!private_token_text(path,text,sizeof(text))||!private_path(text,0)||!jkey(r,"sha256",0)||!jhex(r,sha)
                ||!jkey(r,"source",0)||!private_source_index(r,&source)||!(sources&(1u<<source))||!jclose(r))return 0;
            for(unsigned k=0;k<b->build.patches.count;k++) {
                const struct private_patch_span *p=&b->build.patches.spans[k];
                if(p->source_index==source&&private_token_same(p->path,path)) {
                    if(strcmp(p->after,sha))return 0;applicable_patches|=p->patch_mask;
                }
            }
            span_sources|=1u<<source;count++;if(!private_inner_checkpoint())return 0;
        }
        if(!count||!jarray_close(r)||span_sources!=sources||(patches&applicable_patches)!=applicable_patches||!jclose(r))return 0;
        if(which==6&&commands!=65535u)return 0;
        b->source_coverage|=sources;b->command_coverage|=commands;
    }
    return jarray_close(r)&&jclose(r)&&seen==255u&&b->source_coverage==127u&&b->command_coverage==65535u;
}
// Ref consistency includes repeated offline metadata without opening its payload.
static int private_ref_consistent(const struct private_ref *a,const struct private_ref *b) {
    return !private_token_same(a->path,b->path)||(a->len==b->len&&!strcmp(a->sha,b->sha));
}
static int private_normalize(const char *base,const char *suffix,size_t floor,char out[4097]) {
    size_t used=strlen(base);if(used>4096||used<floor)return 0;memcpy(out,base,used+1);
    while(used>floor&&out[used-1]=='/')out[--used]=0;
    const char *p=suffix;
    while(*p){const char *end=strchr(p,'/');size_t len=end?(size_t)(end-p):strlen(p);
        if(!len||(len==1&&*p=='.')){}
        else if(len==2&&p[0]=='.'&&p[1]=='.'){
            if(used<=floor)return 0;while(used&&out[used-1]!='/')used--;if(used)used--;
            if(used<floor)return 0;out[used]=0;
        } else {if(used+1+len>4096)return 0;out[used++]='/';memcpy(out+used,p,len);used+=len;out[used]=0;}
        if(!end)break;p=end+1;
    }
    return used>floor;
}
static int private_load_resolves(const struct private_basis *b,unsigned from,const struct private_load *load,const char *prefix) {
    char name[4097],path[4097],base[4097],resolved[4097],expected[4097];
    if(!private_token_text(load->name,name,sizeof(name)))return 0;
    const char *last=strrchr(name,'/');if(from<2&&!strcmp(last?last+1:name,"libobjc-env.dylib"))return 0;
    size_t prefix_len=strlen(prefix);
    if(load->cache)return (strncmp(name,prefix,prefix_len)||name[prefix_len]!='/')
        &&(!strncmp(name,"/usr/lib/",9)||!strncmp(name,"/System/",8))&&private_path(name,1);
    int to=private_node_index(b,load->target);if(to<2||(unsigned)to>=b->image_count||b->nodes[to].kind!=1
        ||!private_token_text(b->nodes[to].file.path,path,sizeof(path)))return 0;
    int n=snprintf(expected,sizeof(expected),"%s/%s",prefix,path);if(n<0||(size_t)n>=sizeof(expected))return 0;
    if(name[0]=='/')return !strcmp(name,expected);
    const char *suffix=NULL;
    if(!strncmp(name,"@loader_path/",13)) {
        if(!private_token_text(b->nodes[from].file.path,path,sizeof(path)))return 0;
        char *slash=strrchr(path,'/');if(slash)*slash=0;else path[0]=0;
        n=snprintf(base,sizeof(base),"%s/%s",prefix,path);suffix=name+13;
    } else if(!strncmp(name,"@executable_path/",17)){n=snprintf(base,sizeof(base),"%s/bin",prefix);suffix=name+17;}
    else return 0;
    if(n<0||(size_t)n>=sizeof(base)||!private_normalize(base,suffix,prefix_len,resolved))return 0;
    return !strncmp(resolved,prefix,prefix_len)&&resolved[prefix_len]=='/'&&!strcmp(resolved,expected);
}
static int private_closure_bind_prefix(struct private_basis *b,const char *prefix) {
    const struct private_ref *refs[256];unsigned ref_count=0;b->edge_count=0;
    uint64_t graph[64]={0},reached[2]={1,2},archives=0;
    for(unsigned i=0;i<b->node_count;i++) {
        struct private_node *node=&b->nodes[i];char id[129],path[4097];
        if(!private_token_text(node->id,id,sizeof(id))||!private_token_text(node->file.path,path,sizeof(path)))return 0;
        if(i<2){if(node->kind||strcmp(id,i?"gpg":"git")||strcmp(path,i?"bin/gpg":"bin/git")||node->source!=i)return 0;
            if(node->file.len!=b->build.tools[i].ref.len||strcmp(node->file.sha,b->build.tools[i].ref.sha))return 0;}
        else if(i<b->image_count&&node->kind!=1)return 0;
        for(unsigned j=0;j<i;j++)if(private_token_same(node->file.path,b->nodes[j].file.path))return 0;
        refs[ref_count++]=&node->file;if(node->kind==2)refs[ref_count++]=&node->member_index;
        if(i>=b->image_count)continue;
        struct json_reader r={node->macho.loads.data,node->macho.loads.data+node->macho.loads.len,0,10};if(!jarray(&r))return 0;
        for(unsigned j=0;j<node->macho.load_count;j++){struct private_load load;
            if(++b->edge_count>128||(j&&!jchar(&r,','))||!private_load_read(&r,&load)||!private_load_resolves(b,i,&load,prefix))return 0;
            if(!load.cache){int to=private_node_index(b,load.target);graph[i]|=UINT64_C(1)<<(unsigned)to;}if(!private_inner_checkpoint())return 0;
        }
        if(!jarray_close(&r)||r.p!=r.end)return 0;
    }
    for(unsigned tool=0;tool<2;tool++)for(unsigned pass=0;pass<64;pass++) {
        uint64_t prior=reached[tool];for(unsigned i=0;i<b->image_count;i++)if(reached[tool]&(UINT64_C(1)<<i))reached[tool]|=graph[i];
        if(!private_inner_checkpoint())return 0;if(prior==reached[tool])break;
    }
    uint64_t image_mask=b->image_count==64?UINT64_MAX:(UINT64_C(1)<<b->image_count)-1;
    if((reached[0]|reached[1])!=image_mask)return 0;
    uint64_t rows[2]={0,0};unsigned prior_tool=0,prior_output=0;
    for(unsigned i=0;i<b->link_count;i++) {
        const struct private_link_map *map=&b->links[i];int output=private_node_index(b,map->output);
        if(output<0||(unsigned)output>=b->image_count||!(reached[map->tool]&(UINT64_C(1)<<(unsigned)output))
            ||(rows[map->tool]&(UINT64_C(1)<<(unsigned)output))||strcmp(map->output_sha,b->nodes[output].file.sha))return 0;
        if(i&&(map->tool<prior_tool||(map->tool==prior_tool&&(output==(int)map->tool
            ||(prior_output!=map->tool&&private_id_order(b->nodes[prior_output].id,map->output)>=0)))))return 0;
        prior_tool=map->tool;prior_output=(unsigned)output;rows[map->tool]|=UINT64_C(1)<<(unsigned)output;
        refs[ref_count++]=&map->file;
        for(unsigned j=0;j<i;j++)if(private_token_same(map->output,b->links[j].output)) {
            if(!private_token_same(map->file.raw,b->links[j].file.raw)||strcmp(map->output_sha,b->links[j].output_sha)
                ||!private_token_same(map->inputs,b->links[j].inputs))return 0;
        }
        struct json_reader r={map->inputs.data,map->inputs.data+map->inputs.len,0,10};if(!jarray(&r))return 0;uint64_t selected=0;
        for(unsigned j=0;j<map->count;j++){struct private_slice id;uint32_t count;char hash[65];
            if(++b->edge_count>128||(j&&!jchar(&r,','))||!private_static_use_read(&r,&id,&count,hash))return 0;
            int to=private_node_index(b,id);if(to<0||(unsigned)to<b->image_count||(selected&(UINT64_C(1)<<(unsigned)to)))return 0;
            selected|=UINT64_C(1)<<(unsigned)to;if(!private_inner_checkpoint())return 0;
        }
        if(!jarray_close(&r)||r.p!=r.end)return 0;archives|=selected;
    }
    if(rows[0]!=reached[0]||rows[1]!=reached[1])return 0;
    uint64_t all=b->node_count==64?UINT64_MAX:(UINT64_C(1)<<b->node_count)-1;if((archives|image_mask)!=all)return 0;
    for(unsigned i=0;i<ref_count;i++)for(unsigned j=0;j<i;j++) {
        if(!private_ref_consistent(refs[i],refs[j])||!private_inner_checkpoint())return 0;
    }
    return 1;
}
static __attribute__((unused)) int private_basis_read(const unsigned char *bytes,size_t len,struct private_basis *b,const struct private_input *outer) {
    if(!len||len>49152||!private_canonical(bytes,len))return 0;
    struct json_reader r={bytes,bytes+len,0,10};const unsigned char *start;
    b->raw=(struct private_slice){bytes,len};b->node_count=b->edge_count=0;
    if(!jobject(&r)||!jkey(&r,"build",1))return 0;
    start=r.p;if(!private_build_read(&r,b,outer))return 0;b->build_raw=(struct private_slice){start,(size_t)(r.p-start)};
    if(!jkey(&r,"closure",0))return 0;start=r.p;if(!private_closure_read(&r,b))return 0;b->closure_raw=(struct private_slice){start,(size_t)(r.p-start)};
    if(!jkey(&r,"commands",0))return 0;start=r.p;if(!private_commands_read(&r))return 0;b->commands_raw=(struct private_slice){start,(size_t)(r.p-start)};
    if(!jkey(&r,"config",0))return 0;start=r.p;if(!private_config_read(&r,b))return 0;b->config_raw=(struct private_slice){start,(size_t)(r.p-start)};
    if(!jkey(&r,"fixture",0))return 0;start=r.p;if(!private_fixture_read(&r,b,outer))return 0;b->fixture_raw=(struct private_slice){start,(size_t)(r.p-start)};
    return jkey(&r,"format",0)&&private_literal(&r,"bboxy.dream.private-pretest-basis/1")&&jkey(&r,"review",0)&&private_review_read(&r,b)
        &&jclose(&r)&&r.p==r.end&&r.depth==0&&private_closure_bind_prefix(b,outer->prefix.path)&&private_inner_checkpoint();
}

// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// bboxy450-harness: https://github.com/bboxy450/bboxy450-harness
// Retain origin/credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Staged private-C interface only; no runtime approval or public wire extension.
struct private_engine_context {
    const struct private_input *outer;
    const struct private_basis *basis;
    const char *input_sha256;
};
struct private_engine_metric {
    uint64_t value; // MAX retains a failed unknown/overflow raw observation.
    unsigned known,observed;
};
struct private_engine_result {
    const char *phase,*outcome,*error_code,*error_detail,*observed_proof;
    struct private_engine_metric ordinary_max_ns,controller_max_ns,cleanup_max_ns;
    struct private_engine_metric native_cpu_max_ns,native_wall_max_ns,inclusive_max_ns;
    struct private_engine_metric W_ns,A_ns,C_ns,Q_max_ns,post_cpu_ns,late_kernel_ns;
    struct private_engine_metric debt_charged_ns,debt_paid_ns;
    struct private_engine_metric helper_group_rss_peak_bytes,probe_rss_peak_bytes,metadata_bytes;
    uint16_t attempts,created,reaped;
    uint8_t members_max;
    unsigned group_gone,retired_coverage;
    unsigned engine_entered,semantic_forms,commands_completed,terminal_conservation_checked,terminal_conservation_passed;
};
// Implemented by the surrounding bound-Basis owner. Engine calls these only
// before birth or during an established stopped/no-grant ordinary lane.
static __attribute__((unused)) int private_read_bound_ref(const struct private_engine_context *,const struct private_ref *,int fixture_root,void *buffer,size_t capacity,size_t *len);
static __attribute__((unused)) int private_revalidate_context(const struct private_engine_context *);
// Sole private fixed-profile command engine. Missing temp-signature/source
// premise remains review_unbound before any original process effect.
int private_command_engine(const struct private_engine_context *,struct private_engine_result *);
static int private_emit(const struct private_input *,const char *,struct private_engine_result *);

// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Per-invocation retained immutable-reference custody, never an approval store.
#define PRIVATE_GUARD_MAX 256u
struct private_guard {
    int fd;unsigned root,verified;struct private_slice path;
    uint64_t len;char sha[65];struct stat original;
};
struct private_guard_owner {
    const struct private_input *outer;const struct private_basis *basis;const char *input_hash;
    struct private_guard files[PRIVATE_GUARD_MAX];unsigned count,stage;
};
static struct private_guard_owner private_guards;
#define PRIVATE_CORROBORATION_CAPACITY (1024u*1024u)
#define PRIVATE_CODEC_STACK_CAPACITY (12u*8194u+32768u)
static int private_context_same(const struct private_engine_context *ctx) {
    return ctx&&ctx->outer==private_guards.outer&&ctx->basis==private_guards.basis&&ctx->input_sha256
        &&private_guards.input_hash&&!strcmp(ctx->input_sha256,private_guards.input_hash);
}
static int private_guard_fresh(const struct private_guard *g) {
    char path[4097];if(!private_token_text(g->path,path,sizeof(path)))return private_fail("input_invalid","retained reference path token changed");
    int root=g->root==2?-1:(g->root?private_guards.outer->repo.fd:private_guards.outer->prefix.fd);
    return private_named_current(g->fd,root,path,g->root==2,&g->original);
}
static struct private_guard *private_guard_open(const struct private_engine_context *ctx,const struct private_ref *ref,unsigned fixture) {
    if(!private_context_same(ctx)||private_guards.stage<1||fixture>1){private_fail("review_unbound","reference outside current parsed private context");return NULL;}
    for(unsigned i=0;i<private_guards.count;i++) {
        struct private_guard *g=&private_guards.files[i];
        if(g->root==fixture&&private_token_same(g->path,ref->path)) {
            if(g->len!=ref->len||strcmp(g->sha,ref->sha)){private_fail("review_unbound","one reference path has conflicting immutable bindings");return NULL;}
            return private_guard_fresh(g)?g:NULL;
        }
    }
    if(private_guards.count>=PRIVATE_GUARD_MAX){private_fail("resource_limit","retained private reference count");return NULL;}
    char path[4097];if(!private_token_text(ref->path,path,sizeof(path))||!private_path(path,0)){private_fail("input_invalid","bound reference path invalid");return NULL;}
    int root=fixture?ctx->outer->repo.fd:ctx->outer->prefix.fd;
    int fd=private_open_walk(root,path,0,O_RDONLY);if(fd<0){private_fail("source_changed","bound reference file unavailable");return NULL;}
    // Retain the returned descriptor before any clock/hash/custody verdict.
    struct private_guard *g=&private_guards.files[private_guards.count++];memset(g,0,sizeof(*g));g->fd=fd;g->root=fixture;g->path=ref->path;g->len=ref->len;memcpy(g->sha,ref->sha,65);
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int ok=fstat(fd,&g->original)==0&&S_ISREG(g->original.st_mode)
        &&g->original.st_uid==geteuid()&&g->original.st_nlink==1&&(g->original.st_mode&0022)==0&&(g->original.st_mode&06000)==0
        &&g->original.st_size>=0&&(uint64_t)g->original.st_size==ref->len;
    if(!private_end_unit(at)||!ok){private_fail("source_changed","bound reference identity or allocation differs");return NULL;}
    return g;
}
static __attribute__((unused)) struct private_guard *private_guard_ref(const struct private_engine_context *ctx,const struct private_ref *ref,unsigned fixture) {
    struct private_guard *g=private_guard_open(ctx,ref,fixture);if(!g)return NULL;
    if(!g->verified){char hash[65];if(!private_hash_fd(g->fd,ref->len,hash)||strcmp(hash,ref->sha)||!private_guard_fresh(g)){private_fail("source_changed","bound reference bytes changed");return NULL;}g->verified=1;}
    return g;
}
// Absolute access is limited to the already typed compiler/linker identities
// and the fixed SDKSettings.json suffix. It is not a Ref/path resolver API.
static __attribute__((unused)) struct private_guard *private_guard_absolute(const struct private_engine_context *ctx,const struct private_ref *ref,
    const struct private_abs *identity,int executable) {
    if(!private_context_same(ctx)||private_guards.stage<1||ref->len>INT64_MAX)
        {private_fail("review_unbound","absolute build identity outside current bounded context");return NULL;}
    char path[4097];if(!private_token_text(ref->path,path,sizeof(path))||!private_path(path,1))
        {private_fail("input_invalid","absolute build identity path invalid");return NULL;}
    struct private_guard *g=NULL;
    for(unsigned i=0;i<private_guards.count;i++)if(private_guards.files[i].root==2&&private_token_same(private_guards.files[i].path,ref->path)){g=&private_guards.files[i];break;}
    if(g) {
        if(g->len!=ref->len||strcmp(g->sha,ref->sha)||!private_guard_fresh(g))
            {private_fail("source_changed","absolute build identity changed");return NULL;}
    } else {
        if(private_guards.count>=PRIVATE_GUARD_MAX){private_fail("resource_limit","absolute build identity count");return NULL;}
        int fd=private_open_walk(-1,path,1,O_RDONLY);if(fd<0){private_fail("source_changed","absolute build identity unavailable");return NULL;}
        g=&private_guards.files[private_guards.count++];memset(g,0,sizeof(*g));g->fd=fd;g->root=2;g->path=ref->path;g->len=ref->len;memcpy(g->sha,ref->sha,65);
        uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int valid=fstat(fd,&g->original)==0&&S_ISREG(g->original.st_mode)
            &&(g->original.st_uid==0||g->original.st_uid==geteuid())&&g->original.st_nlink==1
            &&!(g->original.st_mode&0022)&&!(g->original.st_mode&06000)&&g->original.st_size>=0
            &&(uint64_t)g->original.st_size==ref->len&&(!executable||(g->original.st_mode&0100));
        if(identity)valid=valid&&(uint64_t)g->original.st_dev==identity->dev&&(uint64_t)g->original.st_ino==identity->ino
            &&g->original.st_mtimespec.tv_sec==identity->mtime_sec&&(uint32_t)g->original.st_mtimespec.tv_nsec==identity->mtime_nsec;
        if(!private_end_unit(at)||!valid){private_fail("source_changed","absolute build identity stamp differs");return NULL;}
    }
    if(!g->verified){char hash[65];if(!private_hash_fd(g->fd,ref->len,hash)||strcmp(hash,ref->sha)||!private_guard_fresh(g))
        {private_fail("source_changed","absolute build identity bytes differ");return NULL;}g->verified=1;}
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int valid=!executable||(g->original.st_mode&0100);
    if(identity)valid=valid&&(uint64_t)g->original.st_dev==identity->dev&&(uint64_t)g->original.st_ino==identity->ino
        &&g->original.st_mtimespec.tv_sec==identity->mtime_sec&&(uint32_t)g->original.st_mtimespec.tv_nsec==identity->mtime_nsec;
    if(!private_end_unit(at)||!valid){private_fail("source_changed","repeated absolute build identity stamp differs");return NULL;}
    return g;
}
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
static int private_read_bound_ref(const struct private_engine_context *ctx,const struct private_ref *ref,int fixture_root,void *buffer,size_t capacity,size_t *len) {
    if(!buffer||!len||fixture_root<0||fixture_root>1||ref->len>capacity||capacity>2*1024*1024)
        return private_fail("resource_limit","private bounded input buffer");
    struct private_guard *g=private_guard_open(ctx,ref,(unsigned)fixture_root);if(!g)return 0;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);CC_SHA256_CTX hash;int initialized=CC_SHA256_Init(&hash);
    if(!private_end_unit(at)||!initialized)return private_fail("coverage_unknown","reference hash initialization failed");
    size_t copied=0;
    while(copied<(size_t)ref->len) {
        at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int ok=1;
        for(unsigned step=0;step<64&&copied<(size_t)ref->len;step++) {
            size_t n=(size_t)ref->len-copied;if(n>4096)n=4096;
            ssize_t got=pread(g->fd,(char *)buffer+copied,n,(off_t)copied);
            if(got<=0||(size_t)got!=n){private_fail("source_changed","retained reference read incomplete");ok=0;break;}
            if(!CC_SHA256_Update(&hash,(char *)buffer+copied,(CC_LONG)got)){private_fail("coverage_unknown","reference hash update failed");ok=0;break;}
            copied+=(size_t)got;
            uint64_t now=clock_ns(CLOCK_THREAD_CPUTIME_ID);if(now<at){private_fail("clock_unknown","reversed reference copy clock");ok=0;break;}if(now-at>=400000)break;
        }
        int paid=private_end_unit(at);if(!paid||!ok)return 0;
    }
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);unsigned char value[32];char hex[65];int final=CC_SHA256_Final(value,&hash);
    if(final)for(unsigned i=0;i<32;i++)snprintf(hex+2*i,3,"%02x",value[i]);
    int paid=private_end_unit(at);if(!paid||!final||strcmp(hex,ref->sha))return private_fail("source_changed","copied reference bytes differ");
    if(!private_guard_fresh(g))return 0;g->verified=1;*len=copied;return 1;
}
#pragma clang diagnostic pop
static int private_actual_platform(const struct private_platform *platform);
static int private_revalidate_context(const struct private_engine_context *ctx) {
    if(private_failed)return 0;
    if(!private_context_same(ctx)||private_guards.stage!=2)return private_fail("review_unbound","full private basis currentness is not established");
    if(!private_root_current(&ctx->outer->prefix)||!private_root_current(&ctx->outer->repo)||!private_root_current(&ctx->outer->output))return 0;
    for(unsigned i=0;i<private_guards.count;i++)if(!private_guard_fresh(&private_guards.files[i])||!private_inner_checkpoint())return 0;
    return private_actual_platform(&ctx->basis->platform)&&private_end_unit(clock_ns(CLOCK_THREAD_CPUTIME_ID));
}
// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Finite selected-file corroboration. Parsed load commands establish bytes,
// not runtime image completeness or the conditional source-review premise.
static int private_read_at(struct private_guard *g,uint64_t offset,void *buffer,size_t count) {
    if(count>4096||offset>g->len||count>g->len-offset||offset>INT64_MAX)
        return private_fail("review_unbound","selected image read range");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    ssize_t got=pread(g->fd,buffer,count,(off_t)offset);
    int paid=private_end_unit(at);
    if(!paid)return 0;
    if(got<0||(size_t)got!=count)return private_fail("source_changed","selected image read incomplete");
    return 1;
}
static int private_macho_name(struct private_guard *g,uint64_t command,uint32_t size,uint32_t offset,char text[4097]) {
    if(offset>=size)return private_fail("review_unbound","Mach-O selected name range");
    size_t length=size-offset;if(length>4097)length=4097;
    size_t first=length>4096?4096:length;
    if(!private_read_at(g,command+offset,text,first))return 0;
    if(length>first&&!private_read_at(g,command+offset+first,text+first,length-first))return 0;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    const char *end=memchr(text,0,length);
    int ok=end&&end!=text&&private_utf8((const unsigned char *)text,(size_t)(end-text));
    if(!private_end_unit(at))return 0;
    if(!ok)return private_fail("review_unbound","Mach-O selected name encoding");return 1;
}
static int private_macho_expected(struct json_reader *r,unsigned index,const char *actual) {
    char expected[4097];
    return (!index||jchar(r,','))&&jstring(r,expected,sizeof(expected))&&!strcmp(expected,actual);
}
static int private_actual_macho(const struct private_engine_context *ctx,const struct private_node *node) {
    if(node->kind>1)return private_fail("coverage_unknown","non-file Mach-O requires separate platform binding");
    struct private_guard *g=private_guard_ref(ctx,&node->file,0);if(!g)return 0;
    struct mach_header_64 header;
    if(!private_read_at(g,0,&header,sizeof(header)))return 0;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int valid=header.magic==MH_MAGIC_64&&(uint32_t)header.cputype==16777228u
        &&(uint32_t)header.cpusubtype==node->macho.subtype&&(node->kind||node->macho.subtype==0)
        &&header.filetype==(node->kind?MH_DYLIB:MH_EXECUTE)&&header.ncmds<=4096
        &&header.sizeofcmds<=2u*1024u*1024u&&g->len>=sizeof(header)
        &&header.sizeofcmds<=g->len-sizeof(header);
    if(!private_end_unit(at))return 0;
    if(!valid)return private_fail("review_unbound","actual thin ARM64 Mach-O header differs");
    struct json_reader loads={node->macho.loads.data,node->macho.loads.data+node->macho.loads.len,0,10};
    struct json_reader rpaths={node->macho.rpaths.data,node->macho.rpaths.data+node->macho.rpaths.len,0,10};
    if(!jarray(&loads)||!jarray(&rpaths))return private_fail("review_unbound","retained Mach-O arrays invalid");
    unsigned load_count=0,rpath_count=0,uuid_count=0,id_count=0;uint64_t offset=sizeof(header),end=offset+header.sizeofcmds;
    for(uint32_t i=0;i<header.ncmds;i++) {
        struct load_command command;if(!private_read_at(g,offset,&command,sizeof(command)))return 0;
        if(command.cmdsize<sizeof(command)||(command.cmdsize&7u)||command.cmdsize>end-offset)
            return private_fail("review_unbound","Mach-O load-command bounds differ");
        if(command.cmd==LC_UUID) {
            struct uuid_command uuid;if(command.cmdsize!=sizeof(uuid))return private_fail("review_unbound","actual Mach-O UUID command size differs");
            if(!private_read_at(g,offset,&uuid,sizeof(uuid)))return 0;
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);char hex[33];for(unsigned k=0;k<16;k++)snprintf(hex+2*k,3,"%02x",uuid.uuid[k]);
            valid=++uuid_count==1&&!strcmp(hex,node->macho.uuid);if(!private_end_unit(at))return 0;
            if(!valid)return private_fail("review_unbound","actual Mach-O UUID differs");
        } else if(command.cmd==LC_RPATH) {
            struct rpath_command path;char name[4097];
            if(command.cmdsize<sizeof(path)||!private_read_at(g,offset,&path,sizeof(path))||path.path.offset<sizeof(path)
                ||!private_macho_name(g,offset,command.cmdsize,path.path.offset,name))return private_fail("review_unbound","actual Mach-O rpath range");
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);valid=private_macho_expected(&rpaths,rpath_count,name);rpath_count++;
            if(!private_end_unit(at))return 0;if(!valid)return private_fail("review_unbound","actual Mach-O rpaths differ");
        } else if(command.cmd==LC_LOAD_DYLIB||command.cmd==LC_LOAD_WEAK_DYLIB||command.cmd==LC_REEXPORT_DYLIB
            ||command.cmd==LC_LOAD_UPWARD_DYLIB||command.cmd==LC_LAZY_LOAD_DYLIB||command.cmd==LC_ID_DYLIB) {
            struct dylib_command dylib;char name[4097];
            if(command.cmdsize<sizeof(dylib)||!private_read_at(g,offset,&dylib,sizeof(dylib))||dylib.dylib.name.offset<sizeof(dylib)
                ||!private_macho_name(g,offset,command.cmdsize,dylib.dylib.name.offset,name))return private_fail("review_unbound","actual Mach-O dylib range");
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            if(command.cmd==LC_ID_DYLIB) {char expected[4097];valid=node->kind==1&&++id_count==1
                &&private_token_text(node->install_name,expected,sizeof(expected))&&!strcmp(expected,name);}
            else {struct private_load load;unsigned mode=command.cmd==LC_LOAD_DYLIB?0:command.cmd==LC_LOAD_WEAK_DYLIB?1:
                    command.cmd==LC_REEXPORT_DYLIB?2:command.cmd==LC_LOAD_UPWARD_DYLIB?3:4;char expected[4097];
                valid=(!load_count||jchar(&loads,','))&&private_load_read(&loads,&load)&&load.mode==mode
                    &&private_token_text(load.name,expected,sizeof(expected))&&!strcmp(expected,name);load_count++;}
            if(!private_end_unit(at))return 0;if(!valid)return private_fail("review_unbound","actual Mach-O dylib names differ");
        } else if(command.cmd==LC_SEGMENT)return private_fail("review_unbound","32-bit segment in selected ARM64 image");
        offset+=command.cmdsize;
    }
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    valid=offset==end&&uuid_count==1&&id_count==(node->kind?1u:0u)
        &&load_count==node->macho.load_count&&rpath_count==node->macho.rpath_count
        &&jarray_close(&loads)&&loads.p==loads.end&&jarray_close(&rpaths)&&rpaths.p==rpaths.end;
    if(!private_end_unit(at))return 0;
    if(!valid)return private_fail("review_unbound","actual Mach-O command inventory differs");return private_guard_fresh(g);
}
static int private_actual_config(const struct private_engine_context *ctx) {
    const char config[]="batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n";
    unsigned char bytes[sizeof(config)];size_t length=0;
    if(!private_read_bound_ref(ctx,&ctx->basis->gpg_conf,0,bytes,sizeof(bytes),&length))return 0;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int same=length==sizeof(config)-1&&!memcmp(bytes,config,length);
    if(!private_end_unit(at))return 0;
    if(!same)return private_fail("review_unbound","actual fixed public-only GPG configuration differs");
    return private_read_bound_ref(ctx,&ctx->basis->common_conf,0,bytes,sizeof(bytes),&length)&&length==0;
}
// Read only this process's public SDK dyld-info ABI, not another task's port
// or an ambient cache pathname. Cache UUID corroboration alone cannot certify
// all per-image mappings or the candidate's future runtime loaded closure.
static int private_actual_platform(const struct private_platform *platform) {
    if(private_failed)return 0;
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct utsname host;char build[129];size_t length=sizeof(build);
    int known=uname(&host)==0&&sysctlbyname("kern.osversion",build,&length,NULL,0)==0&&length>0&&length<=sizeof(build)&&build[length-1]==0;
    int same=known&&!strcmp(platform->machine,host.machine)&&!strcmp(platform->release,host.release)&&!strcmp(platform->os_build,build);
    if(!private_end_unit(at))return 0;
    if(!same)return private_fail("review_unbound","current platform machine release or build differs");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);task_dyld_info_data_t info;memset(&info,0,sizeof(info));mach_msg_type_number_t count=TASK_DYLD_INFO_COUNT;
    kern_return_t result=task_info(mach_task_self(),TASK_DYLD_INFO,(task_info_t)&info,&count);
    size_t needed=offsetof(struct dyld_all_image_infos,sharedCacheUUID)+16;
    known=result==KERN_SUCCESS&&count==TASK_DYLD_INFO_COUNT&&info.all_image_info_format==TASK_DYLD_ALL_IMAGE_INFO_64
        &&info.all_image_info_addr&&info.all_image_info_size>=needed&&needed<=4096;
    if(!private_end_unit(at))return 0;
    if(!known)return private_fail("coverage_unknown","current process shared-cache identity ABI unavailable");
    struct dyld_all_image_infos snapshot;memset(&snapshot,0,sizeof(snapshot));mach_vm_size_t copied=0;
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    result=mach_vm_read_overwrite(mach_task_self(),info.all_image_info_addr,(mach_vm_size_t)needed,(mach_vm_address_t)&snapshot,&copied);
    known=result==KERN_SUCCESS&&copied==needed&&snapshot.version>=13&&!snapshot.processDetachedFromSharedRegion;
    char uuid[33];unsigned nonzero=0;for(unsigned i=0;i<16;i++){snprintf(uuid+2*i,3,"%02x",snapshot.sharedCacheUUID[i]);nonzero|=snapshot.sharedCacheUUID[i];}
    same=known&&nonzero&&!strcmp(uuid,platform->cache_uuid);
    if(!private_end_unit(at))return 0;
    if(!known)return private_fail("coverage_unknown","current shared-cache bytes unavailable or detached");
    if(!same)return private_fail("review_unbound","current shared-cache UUID differs");return 1;
}
struct private_inventory_walk {char path[4097];uint32_t seen;unsigned entries;};
_Static_assert(128u*(4097u+256u+2u*sizeof(struct stat)+128u)<PRIVATE_CORROBORATION_CAPACITY,"fixture bounded recursive frame capacity");
static int private_directory_close(DIR *dir) {
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int closed=closedir(dir);int paid=private_end_unit(at);
    if(!paid||closed!=0)return private_fail("source_changed","fixture directory close effect unknown; no retry");return 1;
}
// Enumerate only the explicitly selected private fixture root. Every name must
// already be one finite inventory file or its directory prefix. No KB, HOME,
// source-tree or system-directory discovery is performed.
static int private_inventory_directory(const struct private_engine_context *ctx,int fd,unsigned depth,
    size_t prefix,struct private_inventory_walk *walk) {
    if(depth>128){private_close(fd);return private_fail("resource_limit","fixture directory depth bound");}
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat original;
    int valid=fstat(fd,&original)==0&&S_ISDIR(original.st_mode)&&original.st_uid==geteuid()&&(original.st_mode&0022)==0;
    DIR *dir=valid?fdopendir(fd):NULL;
    int paid=private_end_unit(at);
    if(!paid||!dir){if(dir)private_directory_close(dir);else private_close(fd);return private_fail("source_changed","fixture directory custody unavailable");}
    int success=1;
    for(;;) {
        at=clock_ns(CLOCK_THREAD_CPUTIME_ID);errno=0;struct dirent *entry=readdir(dir);int read_error=errno;
        char name[256];int present=entry!=NULL;
        if(present) {
            size_t n=strnlen(entry->d_name,sizeof(name));
            valid=n>0&&n<sizeof(name)&&!memchr(entry->d_name,'/',n);
            if(valid)memcpy(name,entry->d_name,n+1);
        } else valid=read_error==0;
        paid=private_end_unit(at);
        if(!paid||!valid){success=private_fail("source_changed","fixture directory read failed");break;}
        if(!present)break;
        if(!strcmp(name,".")||!strcmp(name,".."))continue;
        at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        if(++walk->entries>4096){success=private_fail("resource_limit","fixture inventory entry bound");(void)private_end_unit(at);break;}
        size_t n=strlen(name);valid=prefix+n<sizeof(walk->path);
        if(valid){memcpy(walk->path+prefix,name,n+1);valid=private_path(walk->path,0);}
        unsigned exact=32;int expected_directory=0;
        for(unsigned i=0;valid&&i<ctx->basis->fixture.file_count;i++) {
            char expected[4097];
            if(!private_token_text(ctx->basis->fixture.files[i].path,expected,sizeof(expected))){valid=0;break;}
            size_t full=prefix+n;
            if(!strcmp(expected,walk->path))exact=i;
            else if(!strncmp(expected,walk->path,full)&&expected[full]=='/')expected_directory=1;
        }
        struct stat named;if(valid)valid=fstatat(dirfd(dir),name,&named,AT_SYMLINK_NOFOLLOW)==0;
        if(!private_end_unit(at)){success=0;break;}
        if(!valid){success=private_fail("source_changed","fixture named entry unavailable");break;}
        if(S_ISREG(named.st_mode)&&exact<32&&!expected_directory) {
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);uint32_t bit=UINT32_C(1)<<exact;
            valid=!(walk->seen&bit);walk->seen|=bit;
            if(!private_end_unit(at)){success=0;break;}
            struct private_guard *g=valid?private_guard_ref(ctx,&ctx->basis->fixture.files[exact],1):NULL;
            if(!g){success=private_fail("review_unbound","duplicate or changed fixture inventory file");break;}
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);valid=private_same_file(&named,&g->original);
            if(!private_end_unit(at)){success=0;break;}
            if(!valid){success=private_fail("source_changed","fixture inventory named stamp changed");break;}
        } else if(S_ISDIR(named.st_mode)&&expected_directory&&exact==32) {
            at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int child=openat(dirfd(dir),name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
            if(!private_end_unit(at)){private_close(child);success=0;break;}
            if(child<0){success=private_fail("source_changed","fixture child directory unavailable");break;}
            walk->path[prefix+n]='/';walk->path[prefix+n+1]=0;
            if(!private_inventory_directory(ctx,child,depth+1,prefix+n+1,walk)){success=0;break;}
        } else {success=private_fail("review_unbound","unlisted symlink special file or directory in fixture");break;}
        walk->path[prefix]=0;
    }
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat after;
    valid=fstat(dirfd(dir),&after)==0&&private_same_file(&original,&after);int closed=closedir(dir);
    paid=private_end_unit(at);
    if(!paid||closed!=0)return private_fail("source_changed","fixture directory close failed");
    if(!valid)return private_fail("source_changed","fixture directory changed during complete inventory");return success;
}
static int private_actual_fixture(const struct private_engine_context *ctx) {
    struct private_inventory_walk walk={{0},0,0};
    // An independent file description is required: dup would share readdir's
    // offset with the retained root and make a later fence start at EOF.
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int fd=openat(ctx->outer->repo.fd,".",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
    if(!private_end_unit(at)){private_close(fd);return 0;}
    if(fd<0)return private_fail("source_changed","fixture inventory root descriptor unavailable");
    if(!private_inventory_directory(ctx,fd,0,0,&walk))return 0;
    uint32_t expected=ctx->basis->fixture.file_count==32?UINT32_MAX:(UINT32_C(1)<<ctx->basis->fixture.file_count)-1;
    if(walk.seen!=expected)return private_fail("review_unbound","complete fixture inventory omitted a file");
    return private_root_current(&ctx->outer->repo);
}
static __attribute__((unused)) void private_guards_dispose(void) {
    unsigned next=0;
    while(next<private_guards.count) {
        uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        for(unsigned step=0;step<64&&next<private_guards.count;step++) {
            struct private_guard *g=&private_guards.files[next++];if(g->fd>=0){int fd=g->fd;g->fd=-1;if(close(fd)!=0)private_fail("source_changed","retained reference close effect unknown; no retry");}
            uint64_t now=clock_ns(CLOCK_THREAD_CPUTIME_ID);if(now<at){private_fail("clock_unknown","reversed reference cleanup clock");break;}if(now-at>=400000)break;
        }
        (void)private_end_unit(at);
    }
    private_guards.count=0;private_guards.stage=0;private_guards.outer=NULL;private_guards.basis=NULL;private_guards.input_hash=NULL;
}

// BEGIN GIT_SIGNATURE_TEMP -- read_wire owns these shared B primitives.
// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Origin: https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Decision65960 section B: match Git's ACTUAL argv, never expand the matcher
// or synthesize a verifier call. All raw work below belongs to the caller's
// existing measured grant/post unit; these helpers add no scheduler or debt.
// They do not prove which inode gpg opened before GS1. The private0700 scratch
// having no other writer remains the explicit conditional source premise.
#define PRIVATE_SIG_PROCARGS 8192u
#define PRIVATE_SIG_INPUT_MAX (2u*1024u*1024u)
struct private_git_sig_temp {
    int scratch_fd,signature_fd,transient_fd;
    unsigned initialized,ready,bound,settled,quarantined,records,signature_close_attempted,scratch_close_attempted,transient_close_attempted;
    char scratch[4097],name[20],sha256[65];uint64_t len;
    const struct private_engine_context *context;const struct private_ref *selected;
    struct stat scratch_stat,signature_stat;mode_t recorded_umask;
    pid_t pid;uint64_t birth;uint32_t pidversion;
    unsigned char procargs[PRIVATE_SIG_PROCARGS];size_t procargs_len;
    uint16_t argv_offset[6],argv_len[6];
};
static int private_sig_error(struct private_git_sig_temp *s,const char *code,const char *detail,int quarantine) {
    if(quarantine)s->quarantined=1;return private_fail(code,detail);
}
static int private_sig_cost(struct private_git_sig_temp *s,uint64_t begin) {
    uint64_t end;
    if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&end)||end<begin)
        return private_sig_error(s,"clock_unknown","Git signature checkpoint clock unavailable or reversed",1);
    if(end-begin>1000000)
        return private_sig_error(s,"resource_limit","Git signature checkpoint exceeds one millisecond",1);
    return 1;
}
static const struct private_ref *private_sig_selected(const struct private_engine_context *ctx) {
    return !strcmp(ctx->outer->case_name,"corrupt_proof")?&ctx->basis->fixture.corrupt_signature:&ctx->basis->fixture.valid_signature;
}
static int private_sig_close_transient(struct private_git_sig_temp *s,DIR *dir) {
    if(s->transient_fd<0||s->transient_close_attempted)return -1;
    s->transient_close_attempted=1;
    int result=dir?closedir(dir):close(s->transient_fd);
    if(result!=0){s->quarantined=1;return -1;}
    s->transient_fd=-1;s->transient_close_attempted=0;return 0;
}
// Exactly one bounded enumeration of the retained scratch. Dup shares its
// offset, so explicitly rewind it before EVERY enumeration. Two persistent
// descriptors are held; this directory duplicate is closed inside the unit.
static int private_sig_entries(struct private_git_sig_temp *s,const char *expected) {
    if(s->transient_fd>=0||s->transient_close_attempted)return -1;
    int fd=dup(s->scratch_fd);if(fd<0)return -1;s->transient_fd=fd;
    if(fcntl(fd,F_SETFD,FD_CLOEXEC)!=0||lseek(fd,0,SEEK_SET)<0){private_sig_close_transient(s,NULL);return -1;}
    DIR *dir=fdopendir(fd);if(!dir){private_sig_close_transient(s,NULL);return -1;}
    unsigned entries=0,matched=0;int result=1;
    for(;;) {
        errno=0;struct dirent *e=readdir(dir);int error=errno;
        if(!e){if(error)result=-1;break;}
        if(++entries>8){result=0;break;}
        if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;
        if(!expected||strcmp(e->d_name,expected)||matched++){result=0;break;}
    }
    if(result==1&&expected&&matched!=1)result=0;
    if(private_sig_close_transient(s,dir)!=0)result=-1;return result;
}
static int private_sig_scratch_current(struct private_git_sig_temp *s) {
    struct stat held,named;
    if(fstat(s->scratch_fd,&held)!=0||fstatat(s->context->outer->output.fd,"t",&named,AT_SYMLINK_NOFOLLOW)!=0)return 0;
    return S_ISDIR(held.st_mode)&&S_ISDIR(named.st_mode)&&held.st_uid==geteuid()&&named.st_uid==geteuid()
        &&(held.st_mode&07777)==0700&&(named.st_mode&07777)==0700
        &&held.st_dev==s->scratch_stat.st_dev&&held.st_ino==s->scratch_stat.st_ino
        &&named.st_dev==held.st_dev&&named.st_ino==held.st_ino;
}
static __attribute__((unused)) int private_git_sig_before_birth(const struct private_engine_context *ctx,struct private_git_sig_temp *s) {
    uint64_t begin;if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&begin))return private_sig_error(s,"clock_unknown","Git scratch fence beginning clock unavailable",0);
    if(!s->initialized||!s->ready||s->context!=ctx||!private_context_same(ctx)||s->bound||s->settled||s->quarantined
        ||s->selected!=private_sig_selected(ctx)||s->selected->len!=s->len||strcmp(s->selected->sha,s->sha256))
        return private_sig_error(s,"review_unbound","Git scratch or selected-case binding is not current before birth",0);
    mode_t mask=umask(0);umask(mask);
    if(mask!=s->recorded_umask||(mask&0600)||!private_sig_scratch_current(s)||private_sig_entries(s,NULL)!=1)
        return private_sig_error(s,"review_unbound","fresh empty0700 Git scratch or recorded umask differs before birth",0);
    return private_sig_cost(s,begin);
}
static __attribute__((unused)) int private_git_sig_prepare(const struct private_engine_context *ctx,struct private_git_sig_temp *s,const char *scratch) {
    if(!ctx||!private_context_same(ctx)||!scratch||s->initialized)
        return private_sig_error(s,"review_unbound","Git signature state cannot be rebound or prepared outside its context",0);
    const struct private_ref *selected=private_sig_selected(ctx);
    if(!selected->len||selected->len>=PRIVATE_SIG_INPUT_MAX||!private_guard_ref(ctx,selected,1))
        return private_sig_error(s,"review_unbound","selected signature Ref exceeds existing input custody or allocation",0);
    uint64_t begin;if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&begin))return private_sig_error(s,"clock_unknown","Git scratch preparation clock unavailable",0);
    char expected[4097];int n=snprintf(expected,sizeof(expected),"%s/t",ctx->outer->output.path);
    if(n<=0||(size_t)n>=sizeof(expected)||strcmp(scratch,expected)||!private_path(scratch,1)
        ||strcmp(private_forms[15].argv[3],"{git_signature_temp}"))
        return private_sig_error(s,"review_unbound","Git scratch bytes or matcher-token contract differs",0);
    memset(s,0,sizeof(*s));s->initialized=1;s->scratch_fd=s->signature_fd=s->transient_fd=-1;s->context=ctx;s->selected=selected;
    memcpy(s->scratch,scratch,(size_t)n+1);s->len=selected->len;memcpy(s->sha256,selected->sha,65);
    s->recorded_umask=umask(0);umask(s->recorded_umask);
    // Retain the returned descriptor before a metadata or clock verdict.
    s->scratch_fd=openat(ctx->outer->output.fd,"t",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
    if(s->scratch_fd<0||fstat(s->scratch_fd,&s->scratch_stat)!=0)
        return private_sig_error(s,"review_unbound","owned fresh Git scratch descriptor unavailable",0);
    s->ready=1;
    if(!private_sig_cost(s,begin))return 0;
    return private_git_sig_before_birth(ctx,s);
}
static int private_sig_arg(const struct private_git_sig_temp *s,unsigned index,const char *expected) {
    size_t n=strlen(expected);return s->argv_len[index]==n&&!memcmp(s->procargs+s->argv_offset[index],expected,n);
}
// Raw KERN_PROCARGS2 layout only: argc, executable NUL, NUL padding, argc
// NUL-terminated argv strings. Preserve the exact returned bytes and offsets.
static int private_git_sig_match_argv(const struct private_engine_context *ctx,struct private_git_sig_temp *s,const unsigned char *bytes,size_t length) {
    if(length<sizeof(int)||length>sizeof(s->procargs))return private_sig_error(s,"coverage_unknown","Git verifier procargs observation incomplete or oversize",1);
    if(bytes!=s->procargs)memcpy(s->procargs,bytes,length);s->procargs_len=length;
    int argc;memcpy(&argc,s->procargs,sizeof(argc));
    if(argc!=6)return private_sig_error(s,"signature_unexpected","actual Git verifier argc differs",1);
    size_t offset=sizeof(argc);const unsigned char *end=memchr(s->procargs+offset,0,length-offset);
    if(!end)return private_sig_error(s,"coverage_unknown","procargs executable terminator unavailable",1);
    offset=(size_t)(end-s->procargs)+1;while(offset<length&&!s->procargs[offset])offset++;
    for(unsigned i=0;i<6;i++) {
        if(offset>=length||(end=memchr(s->procargs+offset,0,length-offset))==NULL)
            return private_sig_error(s,"coverage_unknown","actual verifier argv observation truncated",1);
        size_t n=(size_t)(end-(s->procargs+offset));
        if(offset>UINT16_MAX||n>UINT16_MAX)return private_sig_error(s,"coverage_unknown","actual verifier argv range unavailable",1);
        s->argv_offset[i]=(uint16_t)offset;s->argv_len[i]=(uint16_t)n;offset+=(size_t)n+1;
    }
    char gpg[4097];
    if(!private_token_text(ctx->basis->build.tools[1].ref.path,gpg,sizeof(gpg))
        ||!private_sig_arg(s,0,gpg)||!private_sig_arg(s,1,"--keyid-format=long")
        ||!private_sig_arg(s,2,"--status-fd=1")||!private_sig_arg(s,3,"--verify")||!private_sig_arg(s,5,"-"))
        return private_sig_error(s,"signature_unexpected","actual Git verifier fixed argv differs",1);
    size_t scratch_len=strlen(s->scratch),name_len=19;
    const unsigned char *v=s->procargs+s->argv_offset[4];size_t n=s->argv_len[4];
    if(n!=scratch_len+1+name_len||memcmp(v,s->scratch,scratch_len)||v[scratch_len]!='/'
        ||memcmp(v+scratch_len+1,".git_vtag_tmp",13))
        return private_sig_error(s,"signature_unexpected","actual Git signature path is not its exact TMPDIR temporary",1);
    const unsigned char *name=v+scratch_len+1;
    for(unsigned i=13;i<19;i++)if(!((name[i]>='a'&&name[i]<='z')||(name[i]>='A'&&name[i]<='Z')||(name[i]>='0'&&name[i]<='9')))
        return private_sig_error(s,"signature_unexpected","actual Git signature suffix differs",1);
    memcpy(s->name,name,19);s->name[19]=0;return 1;
}
static int private_sig_read(struct private_git_sig_temp *s,unsigned char *buffer,size_t capacity,char digest[65]) {
    if(!buffer||capacity>PRIVATE_SIG_INPUT_MAX||s->len>=capacity)
        return private_sig_error(s,"resource_limit","signature bytes exceed the existing reusable input buffer",1);
    ssize_t n=pread(s->signature_fd,buffer,(size_t)s->len+1,0);
    if(n<0)return private_sig_error(s,"coverage_unknown","retained Git signature read unavailable",1);
    if((uint64_t)n!=s->len)return private_sig_error(s,"signature_unexpected","actual Git signature byte length differs",1);
    private_hash(buffer,(size_t)s->len,digest);
    if(strcmp(digest,s->sha256))return private_sig_error(s,"signature_unexpected","actual Git signature hash differs from selected case",1);return 1;
}
static int private_sig_record(struct private_git_sig_temp *s,const char *phase,const struct stat *st,const char *digest) {
    uint64_t now;if(!known_clock_ns(CLOCK_MONOTONIC,&now))return private_sig_error(s,"clock_unknown","Git signature evidence clock unavailable",1);
    char line[512];int n=snprintf(line,sizeof(line),"git_sig_temp {\"dev\":%llu,\"ino\":%llu,\"mode\":%u,\"name\":\"%s\",\"nlink\":%llu,\"phase\":\"%s\",\"pid\":%d,\"pidversion\":%u,\"sha256\":\"%s\",\"size\":%llu,\"t_ns\":%llu}\n",
        (unsigned long long)st->st_dev,(unsigned long long)st->st_ino,(unsigned)(st->st_mode&07777),s->name,(unsigned long long)st->st_nlink,phase,s->pid,s->pidversion,digest,(unsigned long long)st->st_size,(unsigned long long)now);
    if(s->records>=2||n<=0||(size_t)n>=sizeof(line))return private_sig_error(s,"resource_limit","two bounded Git signature evidence records exhausted",1);
    s->records++;if(write(STDERR_FILENO,line,(size_t)n)!=n)return private_sig_error(s,"coverage_unknown","Git signature evidence write effect unresolved; no retry",1);return 1;
}
static int private_sig_token_path(const audit_token_t *retained,char path[PROC_PIDPATHINFO_MAXSIZE],const char *expected) {
    audit_token_t token=*retained;
    int n=proc_pidpath_audittoken(&token,path,PROC_PIDPATHINFO_MAXSIZE);
    return n>0&&n<=PROC_PIDPATHINFO_MAXSIZE&&memchr(path,0,PROC_PIDPATHINFO_MAXSIZE)&&!strcmp(path,expected);
}
static __attribute__((unused)) int private_git_sig_gs1(const struct private_engine_context *ctx,struct private_git_sig_temp *s,pid_t pid,uint64_t birth,
    const audit_token_t *retained,unsigned char *buffer,size_t capacity) {
    uint64_t begin;if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&begin))return private_sig_error(s,"clock_unknown","GS1 beginning clock unavailable",1);
    if(!s->ready||s->context!=ctx||!private_context_same(ctx)||s->bound||s->settled||s->quarantined||!retained
        ||pid<=0||retained->val[5]!=(unsigned)pid||!retained->val[7])
        return private_sig_error(s,"coverage_unknown","GS1 does not name the retained current verifier incarnation",1);
    s->pid=pid;s->birth=birth;s->pidversion=retained->val[7];
    audit_token_t before,after;char path[PROC_PIDPATHINFO_MAXSIZE],gpg[4097];
    if(!private_token_text(ctx->basis->build.tools[1].ref.path,gpg,sizeof(gpg))
        ||!known_control_token(pid,birth,&before)||memcmp(&before,retained,sizeof(before))
        ||!private_sig_token_path(retained,path,gpg))
        return private_sig_error(s,"coverage_unknown","GS1 opening verifier token/path observation unavailable or changed",1);
    int mib[3]={CTL_KERN,KERN_PROCARGS2,pid};size_t length=sizeof(s->procargs);
    int observed=sysctl(mib,3,s->procargs,&length,NULL,0);
    if(observed!=0||length>sizeof(s->procargs)||!private_sig_token_path(retained,path,gpg)
        ||!known_control_token(pid,birth,&after)||memcmp(&before,&after,sizeof(after))||after.val[7]!=s->pidversion)
        return private_sig_error(s,"coverage_unknown","GS1 bracketed procargs unavailable or verifier incarnation raced",1);
    if(!private_git_sig_match_argv(ctx,s,s->procargs,length)||!private_sig_cost(s,begin))return 0;
    // Retain the returned signature fd before any stat/hash/mode verdict.
    s->signature_fd=openat(s->scratch_fd,s->name,O_RDONLY|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
    if(s->signature_fd<0)return private_sig_error(s,errno==ENOENT||errno==ELOOP?"signature_unexpected":"coverage_unknown","actual Git signature temporary cannot be opened",1);
    if(fstat(s->signature_fd,&s->signature_stat)!=0)return private_sig_error(s,"coverage_unknown","GS1 actual signature metadata unavailable",1);
    const struct stat *st=&s->signature_stat;
    if(!S_ISREG(st->st_mode)||st->st_nlink!=1||(st->st_mode&07777)!=0600||st->st_uid!=geteuid()
        ||st->st_dev!=s->scratch_stat.st_dev||st->st_size<0||(uint64_t)st->st_size!=s->len)
        return private_sig_error(s,"signature_unexpected","GS1 signature kind link mode uid device or size differs",1);
    char digest[65];if(!private_sig_read(s,buffer,capacity,digest)||!private_sig_cost(s,begin))return 0;
    struct stat held,named;
    if(fstat(s->signature_fd,&held)!=0||fstatat(s->scratch_fd,s->name,&named,AT_SYMLINK_NOFOLLOW)!=0)
        return private_sig_error(s,errno==ENOENT?"signature_unexpected":"coverage_unknown","GS1 signature held/path observation unavailable",1);
    if(!private_same_file(st,&held)||!private_same_file(&held,&named)||!private_sig_scratch_current(s))
        return private_sig_error(s,"signature_unexpected","GS1 held signature path or scratch identity differs",1);
    int entries=private_sig_entries(s,s->name);
    if(entries!=1)return private_sig_error(s,entries==0?"signature_unexpected":"coverage_unknown","GS1 scratch is not exclusively the actual Git signature",1);
    s->bound=1;
    if(!private_sig_record(s,"GS1",&held,digest)||!private_sig_cost(s,begin))return 0;return 1;
}
static __attribute__((unused)) int private_git_sig_gs2(struct private_git_sig_temp *s,int leader_actual_reaped,int normal_exit,unsigned char *buffer,size_t capacity) {
    uint64_t begin;if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&begin))return private_sig_error(s,"clock_unknown","GS2 beginning clock unavailable",1);
    if(!leader_actual_reaped||!s->bound||s->settled||s->signature_fd<0||s->signature_close_attempted)
        return private_sig_error(s,"settlement_unknown","GS2 requires actual leader reap and the retained GS1 descriptor",1);
    if(!private_sig_scratch_current(s))
        return private_sig_error(s,"settlement_unknown","GS2 retained scratch no longer names the original directory",1);
    struct stat named;int present=fstatat(s->scratch_fd,s->name,&named,AT_SYMLINK_NOFOLLOW);int error=errno;
    if(present==0) {
        if(normal_exit)return private_sig_error(s,"signature_unexpected","Git left its signature temporary after normal exit",1);
        if(named.st_dev!=s->signature_stat.st_dev||named.st_ino!=s->signature_stat.st_ino)
            return private_sig_error(s,"settlement_unknown","killed Git signature name changed; owned scratch quarantined",1);
        if(unlinkat(s->scratch_fd,s->name,0)!=0)return private_sig_error(s,"settlement_unknown","owned Git signature cleanup effect unresolved",1);
        present=fstatat(s->scratch_fd,s->name,&named,AT_SYMLINK_NOFOLLOW);error=errno;
    }
    if(present==0||error!=ENOENT)return private_sig_error(s,"coverage_unknown","GS2 absence observation unavailable or raced",1);
    struct stat held;
    if(fstat(s->signature_fd,&held)!=0)return private_sig_error(s,"coverage_unknown","GS2 retained signature metadata unavailable",1);
    if(!S_ISREG(held.st_mode)||held.st_dev!=s->signature_stat.st_dev||held.st_ino!=s->signature_stat.st_ino||held.st_nlink!=0
        ||held.st_size<0||(uint64_t)held.st_size!=s->len)
        return private_sig_error(s,normal_exit?"signature_unexpected":"settlement_unknown","GS2 retained signature identity links or size differs",1);
    char digest[65];if(!private_sig_read(s,buffer,capacity,digest))return 0;
    struct stat after;
    if(fstat(s->signature_fd,&after)!=0)return private_sig_error(s,"coverage_unknown","GS2 retained posthash observation unavailable",1);
    if(!private_same_file(&held,&after))return private_sig_error(s,"signature_unexpected","GS2 retained signature changed during rehash",1);
    if(!private_sig_scratch_current(s))
        return private_sig_error(s,"settlement_unknown","GS2 scratch identity changed during settlement",1);
    if(!private_sig_record(s,"GS2",&after,digest))return 0;
    s->signature_close_attempted=1;int closed=close(s->signature_fd);
    if(closed!=0)return private_sig_error(s,"settlement_unknown","GS2 signature close effect unknown; no retry",1);
    s->signature_fd=-1;
    if(!private_sig_cost(s,begin))return 0;s->settled=1;return 1;
}
static __attribute__((unused)) int private_git_sig_release(struct private_git_sig_temp *s) {
    if(!s->initialized)return 1;
    if(s->quarantined||s->signature_fd>=0||s->transient_fd>=0||(s->bound&&!s->settled)||s->scratch_close_attempted)
        return private_sig_error(s,"settlement_unknown","Git signature state retains unresolved descriptors in quarantine",1);
    if(s->scratch_fd>=0){s->scratch_close_attempted=1;if(close(s->scratch_fd)!=0)return private_sig_error(s,"settlement_unknown","Git scratch close effect unknown; no retry",1);s->scratch_fd=-1;s->scratch_close_attempted=0;}
    s->ready=0;return 1;
}
// END GIT_SIGNATURE_TEMP

// BEGIN COMMAND_ENGINE -- proof_review owns this marked section.
// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Origin: https://github.com/bboxy450/bboxy450-harness
// Retain origin/accurate reuse credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// This is the canonical private controller's staged fixed-profile engine,
// conditional on the bound source review. It supplies no runtime approval.
#include <pthread.h>
#define ENGINE_CAPTURE_MAX (2u*1024u*1024u)
#define ENGINE_INPUT_MAX (2u*1024u*1024u)
#define ENGINE_ARGS_MAX 32768u
#define ENGINE_RECORD_MAX 4096u
#define ENGINE_WORK_LIMIT (78u*1024u*1024u)
struct engine_ticket {pid_t pid,pgid;uint64_t birth,start;uint16_t sequence;};
struct engine_incarnation {pid_t pid;uint64_t birth,start;audit_token_t token;unsigned current,kill_attempted;int kill_result;};
enum engine_image_phase {ENGINE_UNSEEN,ENGINE_CLONE,ENGINE_EXPECTED,ENGINE_EXITED};
struct engine_member {
    struct image state;audit_token_t token;unsigned token_known,phase,stop_version;
    uint64_t user,system,child_user,child_system,exit_abstime; // Actual RI Mach units.
    uint32_t exit_status;unsigned exit_status_known;
    uint64_t counter_read_ns,positive_ack_ns,exit_proved_ns;unsigned counter_post_ack;
};
struct engine_prepared {
    char bytes[ENGINE_ARGS_MAX],*argv[65],*env[12];size_t used;
    unsigned argc,envc;char image[4097],descendant[4097],cwd[4097];
    int input[2],output[2],errors[2];posix_spawnattr_t attr;posix_spawn_file_actions_t actions;
    unsigned attr_live,actions_live;
};
enum engine_closure_stage {ENGINE_STAGE_PIPE,ENGINE_STAGE_DIRECT,ENGINE_STAGE_STOP,ENGINE_STAGE_ROSTER,ENGINE_STAGE_ACTUAL,ENGINE_STAGE_COUNT};
struct engine_stage_observation {uint64_t wall_begin,wall_end,cpu_begin,cpu_end,calls,max_wall,max_cpu;};
struct engine_unit_observation {
    const char *site;unsigned line,ordinary;uint64_t open,end,supplied_actual,supplied_cost,inclusive,actual,cost,poll_before,poll_cpu;
};
struct engine_initial_physical_observation {
    unsigned stage,predicate_mask,bsd_known,audit_path_expected,own_wnowait_known,own_wnowait_terminal,group_original_only,physical_basis;
    int group_result,group_error,child_result,child_error,audit_result,audit_error,bsd_result,bsd_error;
    uint32_t pid,ppid,pgid,status,uid;uint64_t birth;
};
#define ENGINE_EXIT_TEARDOWN_CEILING_NS 150000ULL
struct engine_child_reap_proof {
    pid_t pid;uint32_t version;unsigned valid;
    uint64_t birth,start,exit_ns,zero_ns,ack_ns,read_ns,rollup_ticks;
};
struct engine_owner {
    const struct private_engine_context *context;struct private_engine_result *result;
    struct engine_ticket ticket;struct engine_member members[2];
    struct engine_stage_observation closure_stage[ENGINE_STAGE_COUNT];unsigned closure_limit_failed;
    struct prepared_spawn_outcome native;
    struct engine_incarnation retired[8];unsigned retired_count,slots,verified_descendants;
    struct window_ledger grant;uint64_t accounted_cpu,accounted_user,accounted_system,grant_cpu_before;
    uint64_t accounted_ticks,grant_tick_before;
    uint64_t startup_cpu,charged_member_cpu,justified_late_cpu,justified_post_cpu,counter_reads,wait4_reads;
    uint64_t observed_raw[3],observed_user[3],observed_system[3],post_exit_before[3],post_exit_after[3],post_exit_delta;
    unsigned post_exit_observed,post_exit_initial,post_exit_initial_source_valid,post_exit_ri_unavailable;
    unsigned post_exit_attempt_retained,post_exit_attempt_paid,post_exit_ri_attempted;
    struct engine_initial_physical_observation initial_physical;
    unsigned provisional_count,provisional_revoked,provisional_finalized,rollup_source_failed,worst_provisional_grant;
    uint64_t provisional_quantum_difference,provisional_rest_difference,provisional_debt_added;
    struct window_ledger worst_provisional_window;
    struct engine_child_reap_proof child_reap_proof;
    uint64_t independent_cpu_credit,raw_wait4_cpu,residual_required_cost,residual_raw,residual_retained_cost,residual_payment_target,residual_debit_entries,residual_last_debit_unit,last_charged_cost;
    uint64_t exit_teardown_limit_ns,exit_teardown_allowance_ns,conservation_upper;
    unsigned residual_required_known,residual_cost_paid,teardown_direct_reaps,teardown_child_reaps,teardown_process_records_written;
    uint64_t rounding_allowance,wait4_upper,accounted_upper,conservation_excess;
    unsigned startup_known,conservation_known,conservation_checked,conservation_ok,grant_accounted,accounting_pending;
    unsigned startup_charged,stopped_corroborating,stopped_units;
    uint32_t startup_timebase_numer,startup_timebase_denom;
    uint64_t stopped_max_cpu;
    struct engine_unit_observation failed_unit_first,failed_unit_last;uint64_t failed_units,last_poll_cpu;
    struct rusage retained_wait;int retained_status;uint64_t wait_read_ns;
    unsigned running,stop_requested,retiring,failed,closed,foreground,direct_reaped,direct_custody,coverage_lost;
    unsigned command,grants,case_fired,record_count,child_created,child_settled;uint64_t hold_until,pipe_cpu;
    unsigned stop_pending,stop_terminal,stop_terminal_candidate,stop_attempts;int stop_result,stop_errno;
    uint64_t stop_call_ns,stop_terminal_ns;
    audit_token_t pending_token[2];unsigned pending_final_image_mask,pending_identity_lost;
    unsigned settlement_quarantined,quarantine_recorded,quarantine_durable,group_kill_attempted;int group_kill_result;
    uint64_t settlement_prepaid_cpu;
    unsigned refusal_attempted,refusal_complete,quarantine_settled,owner_lost,owner_lost_recorded,hold_rounds,allocations_owned;
    unsigned hold_reap_normal,hold_settled_recorded,hold_late_charged;pid_t controller_parent;
    uint64_t hold_cpu_total,hold_max_cpu,hold_last_wall,hold_last_cpu,hold_unknown_units;
    unsigned source_admitted,hold_conservative_sleep,hold_unknown_recorded,hold_prior_payment_pending;
    uint64_t hold_terminal_samples,hold_roster_checks,hold_wait4_calls,hold_group_checks;
    uint64_t wait_user,wait_system,wait_rss;unsigned wait_observed;
    char image[4097],descendant[4097];
    int input,output,errors,ledger;size_t sent,out_len,err_len,ledger_bytes;
    unsigned char *capture;const unsigned char *stdin_bytes;size_t stdin_len;
    struct private_git_sig_temp *git_signature;
    unsigned char *signature_buffer;size_t signature_capacity;
};
static struct engine_owner *engine_current=NULL;
// This fixed owner outlives any ambiguous-effect return and the atexit hook.
// A second command cannot replace a physically unsettled first owner.
static struct engine_owner engine_storage;
static void engine_emergency_dispose(void);
static int engine_hold_observe(struct engine_owner *);
static void engine_quarantine_hold(struct engine_owner *);
static void engine_close_transferred(struct engine_owner *);
static int engine_observe_wait4(struct engine_owner *);
static int engine_conservation(struct engine_owner *);
static int engine_credit_member(struct engine_owner *,uint64_t,int);
static void engine_observed_counters(struct engine_owner *);
static int engine_independent_post_exit(struct engine_owner *);
static int engine_stopped_checkpoint(struct engine_owner *,const char *);
static int engine_residual_cost_covers(struct engine_owner *);
static void engine_residual_payment_retained(struct engine_owner *,uint64_t,uint64_t,uint64_t);
// Scratch custody and the actual verifier argv survive all direct commands
// and any refused owned effect. This is not a second controller or tool grant.
static struct private_git_sig_temp engine_signature_storage={.scratch_fd=-1,.signature_fd=-1};
static volatile sig_atomic_t engine_cancel_signal=0,engine_owner_loss_signal=0;
static void engine_cancel_handler(int signal) {(void)signal;engine_cancel_signal=1;}
static void engine_owner_loss_handler(int signal) {(void)signal;engine_owner_loss_signal=1;engine_cancel_signal=1;}
static __attribute__((unused)) int engine_owner_priority(void) {
    static unsigned ready=0;if(ready)return 1;
    errno=0;int current=getpriority(PRIO_PROCESS,0);
    if((current==-1&&errno)||(current<10&&setpriority(PRIO_PROCESS,0,10)!=0))
        return private_fail("review_unbound","private owner nice priority unavailable");
    if(pthread_set_qos_class_self_np(QOS_CLASS_UTILITY,0)!=0)
        return private_fail("review_unbound","private owner utility priority unavailable");
    ready=1;return 1;
}
static void engine_max(struct private_engine_metric *m,uint64_t value) {
    if(!m->observed){m->value=value;m->known=value!=UINT64_MAX;m->observed=1;return;}
    if(value==UINT64_MAX||!m->known){m->value=UINT64_MAX;m->known=0;return;}
    if(value>m->value)m->value=value;
}
static void engine_sum(struct private_engine_metric *m,uint64_t value) {
    if(!m->observed){m->value=value;m->known=value!=UINT64_MAX;m->observed=1;return;}
    if(value==UINT64_MAX||!m->known||UINT64_MAX-m->value<value){m->value=UINT64_MAX;m->known=0;return;}
    m->value+=value;
}
static int engine_failure(struct engine_owner *owner,const char *code,const char *detail) {
    owner->failed=1;private_fail(code,detail);
    owner->result->error_code=private_error;owner->result->error_detail=private_detail;
    owner->result->outcome=owner->settlement_quarantined?"quarantined":"failed";return 0;
}
static uint64_t engine_interval(uint64_t begin,uint64_t end) {
    return begin==UINT64_MAX||end==UINT64_MAX||end<begin?UINT64_MAX:end-begin;
}
static int engine_clock(struct engine_owner *owner,clockid_t id,uint64_t *value) {
    if(known_clock_ns(id,value))return 1;
    *value=UINT64_MAX;return engine_failure(owner,"clock_unknown","engine clock unavailable or reversed");
}
static uint64_t engine_now(struct engine_owner *owner,clockid_t id) {
    uint64_t value=UINT64_MAX;engine_clock(owner,id,&value);return value;
}
static int engine_stage_begin(struct engine_owner *owner,unsigned stage) {
    struct engine_stage_observation *s=&owner->closure_stage[stage];
    return engine_clock(owner,CLOCK_MONOTONIC,&s->wall_begin)&&engine_clock(owner,CLOCK_THREAD_CPUTIME_ID,&s->cpu_begin);
}
static int engine_stage_end(struct engine_owner *owner,unsigned stage) {
    struct engine_stage_observation *s=&owner->closure_stage[stage];
    if(!engine_clock(owner,CLOCK_THREAD_CPUTIME_ID,&s->cpu_end)||!engine_clock(owner,CLOCK_MONOTONIC,&s->wall_end))return 0;
    if(s->wall_end<s->wall_begin||s->cpu_end<s->cpu_begin||s->calls==UINT64_MAX)
        return engine_failure(owner,"clock_unknown","closure stage entry/exit clock or count is unknown");
    s->calls++;uint64_t wall=s->wall_end-s->wall_begin,cpu=s->cpu_end-s->cpu_begin;
    if(wall>s->max_wall)s->max_wall=wall;if(cpu>s->max_cpu)s->max_cpu=cpu;return 1;
}
static int engine_direct_exited(struct engine_owner *owner,int *done) {
    if(owner->ticket.pid<=0||!known_exited(owner->ticket.pid,done))
        return engine_failure(owner,"settlement_unknown","original unreaped direct handle unavailable");
    return 1;
}
static int engine_ticket_equal(const struct engine_ticket *a,const struct engine_ticket *b) {
    return a->pid==b->pid&&a->pgid==b->pgid&&a->birth==b->birth&&a->start==b->start&&a->sequence==b->sequence;
}
static int engine_token_signal(const audit_token_t *token,int signal) {
    errno=0;int result=proc_signal_with_audittoken((audit_token_t *)token,signal);
    return result==0?0:(result>0?result:errno);
}
static int engine_token_gone(const audit_token_t *token) {
    char path[PROC_PIDPATHINFO_MAXSIZE];errno=0;
    int result=proc_pidpath_audittoken((audit_token_t *)token,path,sizeof(path));
    return result<=0&&errno==ESRCH;
}
static int engine_owned_token(struct engine_owner *owner,pid_t pid,uint64_t birth,uint64_t start,const audit_token_t *token) {
    for(unsigned i=0;i<owner->retired_count;i++)if(owner->retired[i].pid==pid&&!memcmp(&owner->retired[i].token,token,sizeof(*token)))return 1;
    if(owner->retired_count>=8)return engine_failure(owner,"coverage_unknown","retained incarnation capacity exceeded");
    for(unsigned i=0;i<owner->retired_count;i++)if(owner->retired[i].pid==pid)owner->retired[i].current=0;
    struct engine_incarnation *member=&owner->retired[owner->retired_count++];
    member->pid=pid;member->birth=birth;member->start=start;member->token=*token;member->current=1;return 1;
}
static int engine_safe_sum(uint64_t a,uint64_t b,uint64_t *sum) {if(UINT64_MAX-a<b)return 0;*sum=a+b;return 1;}
static int engine_scaled_ticks(uint64_t raw,uint32_t numer,uint32_t denom,int upper,uint64_t *ns) {
    if(!numer||!denom)return 0;
    __uint128_t scaled=(__uint128_t)raw*numer;
    __uint128_t result=scaled/denom+(upper&&scaled%denom!=0);
    if(result>UINT64_MAX)return 0;*ns=(uint64_t)result;return 1;
}
static int engine_ticks_ns(uint64_t raw,uint64_t *ns) {
    mach_timebase_info_data_t scale;
    if(mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom)return 0;
    return engine_scaled_ticks(raw,scale.numer,scale.denom,1,ns);
}
static int engine_usage(struct engine_owner *owner,unsigned index,const struct rusage_info_v1 *usage) {
    struct engine_member *member=&owner->members[index];
    unsigned reads=index?2:4;
    if(!engine_safe_sum(owner->counter_reads,reads,&owner->counter_reads))
        return engine_failure(owner,"clock_unknown","counter-read rounding count overflow");
    if(!usage->ri_proc_start_abstime||(member->state.start_absolute&&usage->ri_proc_start_abstime!=member->state.start_absolute)
        ||usage->ri_user_time<member->user||usage->ri_system_time<member->system)
        return engine_failure(owner,"coverage_unknown","nonmonotonic or foreign resource identity");
    uint64_t raw;
    if(!engine_safe_sum(usage->ri_user_time,usage->ri_system_time,&raw))return engine_failure(owner,"clock_unknown","helper CPU sum overflow");
    member->state.start_absolute=usage->ri_proc_start_abstime;member->state.raw_rusage_cpu=raw;
    if(!engine_ticks_ns(raw,&member->state.rusage_cpu))return engine_failure(owner,"clock_unknown","helper CPU conversion unknown");
    member->user=usage->ri_user_time;member->system=usage->ri_system_time;member->counter_post_ack=0;
    member->counter_read_ns=engine_now(owner,CLOCK_MONOTONIC);
    if(member->counter_read_ns==UINT64_MAX)return 0;
    if(member->exit_abstime&&usage->ri_proc_exit_abstime!=member->exit_abstime)
        return engine_failure(owner,"coverage_unknown","recorded original exit changed");
    member->exit_abstime=usage->ri_proc_exit_abstime;
    for(unsigned i=0;i<owner->retired_count;i++)if(owner->retired[i].pid==member->state.pid&&owner->retired[i].current
        &&owner->retired[i].token.val[7]==member->token.val[7])owner->retired[i].start=member->state.start_absolute;
    if(index==0) {
        if(usage->ri_child_user_time<member->child_user||usage->ri_child_system_time<member->child_system)
            return engine_failure(owner,"coverage_unknown","descendant rollup reversed");
        member->child_user=usage->ri_child_user_time;member->child_system=usage->ri_child_system_time;
        if(!engine_safe_sum(member->child_user,member->child_system,&raw))return engine_failure(owner,"clock_unknown","descendant CPU sum overflow");
        member->state.raw_waited=raw;
        if(!engine_ticks_ns(raw,&member->state.waited))return engine_failure(owner,"clock_unknown","descendant CPU conversion unknown");
    }
    return 1;
}
static int engine_actual_ticks(struct engine_owner *owner,uint64_t *cpu,uint64_t *user,uint64_t *system) {
    struct engine_member *a=&owner->members[0],*b=&owner->members[1];uint64_t u,s;
    if(!engine_safe_sum(a->user,b->user>a->child_user?b->user:a->child_user,&u)
        ||!engine_safe_sum(a->system,b->system>a->child_system?b->system:a->child_system,&s)
        ||!engine_safe_sum(a->state.raw_rusage_cpu,b->state.raw_rusage_cpu>a->state.raw_waited?b->state.raw_rusage_cpu:a->state.raw_waited,cpu))
        return engine_failure(owner,"clock_unknown","nonduplicated helper CPU sum overflow");
    *user=u;*system=s;return 1;
}
static int engine_actual(struct engine_owner *owner,uint64_t *cpu,uint64_t *user,uint64_t *system) {
    uint64_t raw,u,s;
    if(!engine_actual_ticks(owner,&raw,&u,&s)||!engine_ticks_ns(raw,cpu)||!engine_ticks_ns(u,user)||!engine_ticks_ns(s,system))
        return engine_failure(owner,"clock_unknown","nonduplicated CPU conversion unknown");
    return 1;
}
static int engine_actual_delta(struct engine_owner *owner,uint64_t baseline,uint64_t *delta,uint64_t *current) {
    uint64_t u,s;
    if(!engine_actual_ticks(owner,current,&u,&s)||*current<baseline||!engine_ticks_ns(*current-baseline,delta)) {
        *delta=UINT64_MAX;return engine_failure(owner,"clock_unknown","helper interval ticks unknown or reversed");
    }
    return 1;
}
// Decision66050: a consumed failed STOP keeps the original grant open. The
// errno supplies no ACK or endpoint; only independent original terminal and
// complete member/RI/roster observations can close it. No second signal follows.
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static unsigned source_known_closure_late_stop;
#endif
static int engine_stop_result(struct engine_owner *owner,int result,int error,uint64_t at) {
    owner->stop_result=result;owner->stop_errno=error;owner->stop_call_ns=at;
    owner->stop_attempts++;
    if(result==0)return 1;
    if(error==EPERM||error==ESRCH){
        owner->stop_pending=1;owner->pending_final_image_mask=0;
        for(unsigned i=0;i<owner->slots;i++)if(owner->members[i].token_known
            &&owner->members[i].phase==ENGINE_EXPECTED&&owner->members[i].state.start_absolute) {
            owner->pending_token[i]=owner->members[i].token;owner->pending_final_image_mask|=1u<<i;
        }
        return 1;
    }
    return engine_failure(owner,"coverage_unknown","original group STOP effect unresolved");
}
static int engine_stop_group(struct engine_owner *owner) {
    if(owner->settlement_quarantined)return engine_failure(owner,"coverage_unknown","quarantine admits observations only, never another STOP");
    if(owner->stop_requested)return 1;
    int done;if(!known_exited(owner->ticket.pid,&done))return engine_failure(owner,"settlement_unknown","original unreaped group custody unavailable");
    owner->stop_requested=1;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    // Deliberate source-known violation only: the actual STOP call is late,
    // and the unchanged original W/Q/debt machinery must reject all of it.
    if(source_known_closure_late_stop)usleep(30000); // Decision66236's separate hard25ms wall negative.
#endif
    uint64_t at=engine_now(owner,CLOCK_MONOTONIC);errno=0;
    int result=kill(-owner->ticket.pgid,SIGSTOP),error=result?errno:0;
    return engine_stop_result(owner,result,error,at);
}
static int engine_member_exit(struct engine_owner *owner,unsigned index) {
    struct engine_member *member=&owner->members[index];struct rusage_info_v1 usage;
    if(owner->stop_pending&&(!owner->grant.source_complete||owner->pending_identity_lost
        ||!(owner->pending_final_image_mask&(1u<<index))
        ||memcmp(&member->token,&owner->pending_token[index],sizeof(member->token))))
        return engine_failure(owner,"coverage_unknown","pending terminal lacks admitted final-image exec continuity");
    // Public RI/BSDINFO does not expose a terminal exec version. Once a fully
    // admitted final image vanishes, the no-further-exec relation is ONLY the
    // bound source/platform premise. Live continuity below invalidates it on
    // every observed change; unknown/pre-exec slots never acquire this premise.
    if(!member->token_known||!member->state.start_absolute||!engine_token_gone(&member->token))return 0;
    if(proc_pid_rusage(member->state.pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)!=0)
        return errno==ESRCH&&member->phase==ENGINE_EXITED&&member->exit_abstime!=0&&member->exit_proved_ns!=0;
    if(usage.ri_proc_exit_abstime==0||!engine_usage(owner,index,&usage))return 0;
    // Q2 is not the verifier's parent. Retain its actual BSD wait status while
    // the original zombie is still observable; no status is invented from RI,
    // Git's generic exit code or a later empty census. Optional status
    // corroboration is separate from the exact Git cryptographic status path.
    struct proc_bsdinfo b;errno=0;
    int got=proc_pidinfo(member->state.pid,PROC_PIDTBSDINFO,0,&b,sizeof(b));
    if(got==(int)sizeof(b)) {
        uint64_t birth=(uint64_t)b.pbi_start_tvsec*1000000+b.pbi_start_tvusec;
        if(b.pbi_pid!=(uint32_t)member->state.pid||birth!=member->state.birth||b.pbi_status!=SZOMB)
            return engine_failure(owner,"coverage_unknown","terminal BSD status belongs to a foreign or live incarnation");
        if(member->exit_status_known&&member->exit_status!=b.pbi_xstatus)
            return engine_failure(owner,"coverage_unknown","retained original terminal status changed");
        member->exit_status=b.pbi_xstatus;member->exit_status_known=1;
    }
    member->phase=ENGINE_EXITED;
    if(!member->exit_proved_ns)member->exit_proved_ns=engine_now(owner,CLOCK_MONOTONIC);
    return member->exit_proved_ns!=UINT64_MAX&&member->exit_proved_ns!=0;
}
static int engine_pending_live_member(struct engine_owner *owner,unsigned index,const struct image *current) {
    struct engine_member *member=&owner->members[index];audit_token_t before,after;char path[PROC_PIDPATHINFO_MAXSIZE];
    if(owner->pending_identity_lost||!(owner->pending_final_image_mask&(1u<<index))
        ||!member->token_known||member->phase!=ENGINE_EXPECTED||!owner->grant.source_complete)
        goto changed;
    if(!known_control_token(current->pid,current->birth,&before))goto changed;
    if(memcmp(&before,&owner->pending_token[index],sizeof(before))) {
        // Retain the newly observed real effect for identity-safe disposal,
        // without reviving or admitting it as the original logical member.
        (void)engine_owned_token(owner,current->pid,current->birth,current->start_absolute,&before);goto changed;
    }
    if(proc_pidpath_audittoken(&before,path,sizeof(path))<=0
        ||!memchr(path,0,sizeof(path))||strcmp(path,index?owner->descendant:owner->image))goto changed;
    struct rusage_info_v1 usage;
    if(proc_pid_rusage(current->pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)!=0
        ||!engine_usage(owner,index,&usage)||!known_control_token(current->pid,current->birth,&after)
        ||memcmp(&before,&after,sizeof(before)))goto changed;
    member->state.status=current->status;member->state.runnable=current->runnable;return 1;
changed:
    owner->pending_identity_lost=owner->coverage_lost=1;
    return engine_failure(owner,"coverage_unknown","pending STOP observed missing or changed exec token path or RI identity");
}
static int engine_sample_member(struct engine_owner *owner,unsigned index,pid_t pid,int *ack) {
    *ack=0;struct engine_member *member=&owner->members[index];struct image current;
    // Terminal proof precedes live-thread/image predicates: a zombie may have
    // zero threads and no pathname. The old token + RI exit/start continuity,
    // plus WNOWAIT for the original direct leader, are independent evidence.
    int original_done=0;
    if(member->token_known&&(index|| (engine_direct_exited(owner,&original_done)&&original_done))
        &&engine_member_exit(owner,index)){*ack=1;return 1;}
    if(!control_observe(pid,&current)) {
        int done=0;
        if(index==0&&engine_direct_exited(owner,&done)&&done&&engine_member_exit(owner,index)){*ack=1;return 1;}
        if(index==1&&engine_member_exit(owner,index)){*ack=1;return 1;}
        return engine_failure(owner,"coverage_unknown","owned member observation unavailable");
    }
    if(current.pid!=pid||current.pgid!=owner->ticket.pgid||current.ppid!=(index?owner->ticket.pid:getpid())
        ||(member->state.birth&&current.birth!=member->state.birth)||current.uid!=geteuid()||current.nice<10||current.threads!=1||current.runnable>1)
        return engine_failure(owner,"coverage_unknown","member lineage birth group or one-thread predicate differs");
    if(member->phase==ENGINE_EXITED)return engine_failure(owner,"coverage_unknown","retired logical member revived");
    if(owner->stop_pending&&current.status!=SZOMB) {
        // A live pending member still requires exact bracketed audit/path/RI
        // continuity. It receives no ACK and no second STOP/CONT on success.
        *ack=0;return engine_pending_live_member(owner,index,&current);
    }
    if(index==0&&current.birth!=owner->ticket.birth)return engine_failure(owner,"coverage_unknown","leader birth differs");
    unsigned previous=member->token_known?member->token.val[7]:0;int fresh=0;
    audit_token_t token=member->token;
    if(!member->token_known&&!known_control_token(pid,current.birth,&token))return engine_failure(owner,"coverage_unknown","owned current audit token unavailable");
    char path[PROC_PIDPATHINFO_MAXSIZE];errno=0;
    int found=proc_pidpath_audittoken(&token,path,sizeof(path));
    if(found<=0&&errno==ESRCH) {
        if(!known_control_token(pid,current.birth,&token))return engine_failure(owner,"coverage_unknown","new exec token capture unavailable");
        if(!engine_owned_token(owner,pid,current.birth,current.start_absolute,&token))return 0;
        if(index==0||token.val[7]<=previous)return engine_failure(owner,"coverage_unknown","unexpected leader or reverse exec incarnation");
        if(engine_token_signal(&token,SIGSTOP)!=0)return engine_failure(owner,"coverage_unknown","new exec task STOP unresolved");
        fresh=1;member->stop_version=token.val[7];
        found=proc_pidpath_audittoken(&token,path,sizeof(path));
    }
    // Always retain the new real recipient before any image or timing verdict.
    if(!engine_owned_token(owner,pid,current.birth,current.start_absolute,&token))return 0;
    member->token=token;member->token_known=1;
    if(found<=0)return engine_failure(owner,"coverage_unknown","current incarnation path unavailable");
    if(index==0) {
        if(strcmp(path,owner->image)||fresh)return engine_failure(owner,"coverage_unknown","direct executable incarnation differs");
        member->phase=ENGINE_EXPECTED;
    } else if(!strcmp(path,owner->descendant)) {
        if(member->stop_version!=token.val[7]) {
            if(engine_token_signal(&token,SIGSTOP)!=0)return engine_failure(owner,"coverage_unknown","first known verifier STOP unresolved");
            member->stop_version=token.val[7];fresh=1;
        }
        if(member->phase==ENGINE_EXPECTED) {
            if(!owner->git_signature||!owner->git_signature->bound
                ||owner->git_signature->pid!=pid||owner->git_signature->pidversion!=token.val[7])
                return engine_failure(owner,"coverage_unknown","verified descendant incarnation changed after GS1");
        } else {
            // Decision B: retain the ACTUAL child argv and generated Git file
            // before phase/count/ACK or another CONT. Every procargs, directory,
            // FD/hash/receipt operation remains inside this open grant's C.
            if(!owner->git_signature||!owner->signature_buffer
                ||!private_git_sig_gs1(owner->context,owner->git_signature,pid,current.birth,&token,
                    owner->signature_buffer,owner->signature_capacity))
                return engine_failure(owner,private_failed?private_error:"coverage_unknown",
                    private_failed?private_detail:"GS1 binding unavailable");
            owner->verified_descendants++;
            member->phase=ENGINE_EXPECTED;
        }
    } else if(!strcmp(path,owner->image)&&member->phase!=ENGINE_EXPECTED&&!fresh)member->phase=ENGINE_CLONE;
    else return engine_failure(owner,"coverage_unknown","unapproved or reverse image phase");
    uint64_t positive=0;
    if(!fresh&&current.status==SSTOP&&current.runnable==0) {
        positive=engine_now(owner,CLOCK_MONOTONIC);if(positive==UINT64_MAX)return 0;
    }
    struct rusage_info_v1 usage;
    if(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)!=0)return engine_failure(owner,"coverage_unknown","current resource snapshot unavailable");
    uint64_t prior_start=member->state.start_absolute;member->state=current;member->state.start_absolute=prior_start;
    memcpy(member->state.path,path,sizeof(member->state.path));
    if(!engine_usage(owner,index,&usage))return 0;
    member->positive_ack_ns=positive;member->counter_post_ack=positive&&member->counter_read_ns>=positive;
    if(current.status==SZOMB&&engine_member_exit(owner,index)){*ack=1;return 1;}
    *ack=!fresh&&current.status==SSTOP&&current.runnable==0;return 1;
}
static int engine_qualified_rollup(struct engine_owner *owner,int children_count) {
    struct window_ledger *g=&owner->grant;struct engine_member *leader=&owner->members[0],*child=&owner->members[1];
    if(!g->t0||owner->grant_accounted||owner->slots!=2)return 1;
    g->rollup_mask=0;
    if(children_count||child->phase!=ENGINE_EXITED||!child->exit_proved_ns||g->unknown_mask
        ||!(g->baseline_mask&2)||!child->token_known||owner->verified_descendants!=1
        ||!owner->git_signature||!owner->git_signature->bound||owner->git_signature->pid!=child->state.pid
        ||owner->git_signature->pidversion!=child->token.val[7])return 1;
    if(owner->coverage_lost||owner->pending_identity_lost||!g->source_complete
        ||owner->context->basis->source_coverage!=127||owner->context->basis->command_coverage!=65535)
        return engine_failure(owner,"coverage_unknown","qualified rollup lacks the unique admitted source roster");
    // The already observed child's original positive exit precedes a fresh
    // count-zero census AND the new leader RI read. A live/old sample cannot
    // be promoted by later evidence. No child wait4 is fabricated here.
    if(!engine_member_exit(owner,1))return engine_failure(owner,"coverage_unknown","qualified rollup lost the retained child exit identity");
    int children[3]={0};int n=proc_listchildpids(owner->ticket.pid,children,sizeof(children));
    if(n!=0){owner->rollup_source_failed=owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","qualified rollup observed another or unretired child");}
    uint64_t zero=engine_now(owner,CLOCK_MONOTONIC),ack=0,prior_user=leader->user;
    if(leader->phase==ENGINE_EXITED) {
        int done=0;if(!engine_direct_exited(owner,&done)||!done||!g->end[0])return engine_failure(owner,"coverage_unknown","terminal rollup lacks original leader ACK");
        ack=g->end[0];
        if(!engine_member_exit(owner,0))return engine_failure(owner,"coverage_unknown","terminal qualified leader RI unavailable");
    } else {
        struct image current;audit_token_t before,after;
        if(!control_observe(owner->ticket.pid,&current)||current.birth!=owner->ticket.birth||current.ppid!=getpid()
            ||current.pgid!=owner->ticket.pgid||current.status!=SSTOP||current.threads!=1||current.runnable
            ||!known_control_token(owner->ticket.pid,owner->ticket.birth,&before)||memcmp(&before,&leader->token,sizeof(before)))
            return engine_failure(owner,"coverage_unknown","qualified rollup leader lacks positive current-task ACK");
        ack=engine_now(owner,CLOCK_MONOTONIC);struct rusage_info_v1 usage;
        if(proc_pid_rusage(owner->ticket.pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)!=0||!engine_usage(owner,0,&usage)
            ||!known_control_token(owner->ticket.pid,owner->ticket.birth,&after)||memcmp(&before,&after,sizeof(before)))
            return engine_failure(owner,"coverage_unknown","qualified rollup read lost leader incarnation continuity");
        leader->positive_ack_ns=ack;leader->counter_post_ack=leader->counter_read_ns>=ack;
    }
    if(leader->user!=prior_user){owner->rollup_source_failed=1;return engine_failure(owner,"coverage_unknown","qualified rollup has unexplained leader user growth after ACK");}
    int group[3]={0};n=proc_listpgrppids(owner->ticket.pgid,group,sizeof(group));
    int count=proc_listchildpids(owner->ticket.pid,children,sizeof(children));
    if(count||n<0||n>1||(n==1&&group[0]!=owner->ticket.pid)) {
        owner->rollup_source_failed=owner->coverage_lost=1;
        return engine_failure(owner,"coverage_unknown","qualified rollup final roster contains another child or member");
    }
    if(zero==UINT64_MAX||ack==UINT64_MAX||leader->counter_read_ns<zero||leader->counter_read_ns<ack
        ||leader->counter_read_ns<child->exit_proved_ns)return engine_failure(owner,"clock_unknown","qualified rollup clocks lack causal ordering");
    g->rollup_mask=2;g->rollup_unique=g->rollup_child_retired=1;
    g->rollup_child_exit_ns=child->exit_proved_ns;g->rollup_child_zero_ns=zero;
    g->rollup_leader_ack_ns=ack;g->rollup_read_ns=leader->counter_read_ns;
    return 1;
}
static int engine_roster(struct engine_owner *owner,int final,int *closed) {
    *closed=0;pid_t child=0;int children[3]={0};
    int count=proc_listchildpids(owner->ticket.pid,children,sizeof(children));
    if(count<0||count>2)return engine_failure(owner,"coverage_unknown","owned child census unavailable or overflowing");
    if(count>1||((unsigned)count&&owner->slots!=2)) {
        owner->coverage_lost=owner->rollup_source_failed=1;
        if(owner->settlement_quarantined){owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","quarantined roster exceeded original reservation");}
        engine_stop_group(owner);return engine_failure(owner,"coverage_unknown","source-bounded process count exceeded");
    }
    if(count)child=children[0];
    if(child>0&&!owner->members[1].state.pid) {
        // Capture the real logical effect before expensive token/image checks.
        owner->members[1].state.pid=child;
        if(!owner->child_created){owner->child_created=1;owner->result->created++;owner->result->members_max=2;}
        if(owner->stopped_corroborating){owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","stopped corroboration discovered an unbound member");}
        if(owner->settlement_quarantined){owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","quarantined observation discovered an unbound child");}
        if(owner->stop_pending)return engine_failure(owner,"coverage_unknown","new child cannot close a pending terminal unknown slot");
        owner->stop_requested=0;
        if(!engine_stop_group(owner))return 0;
        if(owner->grant.t0)owner->grant.unknown_mask|=2u;
    }
    if(child>0&&owner->members[1].state.pid!=child) {
        owner->coverage_lost=owner->rollup_source_failed=1;
        return engine_failure(owner,"coverage_unknown","second logical child identity changed");
    }
    if(!engine_stopped_checkpoint(owner,"stopped_child_census"))return 0;
    int pids[3]={0};int n=proc_listpgrppids(owner->ticket.pgid,pids,sizeof(pids));
    if(n<0||n>2||(unsigned)n>owner->slots)return engine_failure(owner,"coverage_unknown","exact group roster unavailable or exceeds reservation");
    if(!engine_stopped_checkpoint(owner,"stopped_group_census"))return 0;
    unsigned seen=0;int all=1;
    for(int i=0;i<n;i++) {
        unsigned index=pids[i]==owner->ticket.pid?0u:1u;
        if(index&&(!child||pids[i]!=child))return engine_failure(owner,"coverage_unknown","group member has no owned parent edge");
        if(seen&(1u<<index))return engine_failure(owner,"coverage_unknown","duplicate logical member");
        seen|=1u<<index;int ack;
        if(!engine_sample_member(owner,index,pids[i],&ack))return 0;
        all&=ack;
        if(ack&&owner->grant.t0&&!owner->grant.end[index]) {
            owner->grant.end[index]=engine_now(owner,CLOCK_MONOTONIC);owner->grant.ack_mask|=1u<<index;
            owner->grant.identity_mask|=1u<<index;
        }
        if(owner->stopped_corroborating&&!ack)return engine_failure(owner,"coverage_unknown","corroboration lost original stopped member");
        if(!engine_stopped_checkpoint(owner,"stopped_member_identity"))return 0;
    }
    if(!(seen&1u)) {
        int done=0;if(!engine_direct_exited(owner,&done)||!done||!engine_member_exit(owner,0))return engine_failure(owner,"coverage_unknown","missing leader lacks exact unreaped exit evidence");
        if(owner->grant.t0&&!owner->grant.end[0])owner->grant.end[0]=engine_now(owner,CLOCK_MONOTONIC);
        owner->grant.ack_mask|=1;owner->grant.identity_mask|=1;
        if(!engine_stopped_checkpoint(owner,"stopped_leader_exit"))return 0;
    }
    if(owner->members[1].state.pid&&!(seen&2u)) {
        if(!engine_member_exit(owner,1))return engine_failure(owner,"coverage_unknown","missing original child has no retained exact exit evidence");
        if(owner->grant.t0&&!owner->grant.end[1])owner->grant.end[1]=engine_now(owner,CLOCK_MONOTONIC);
        owner->grant.ack_mask|=2;owner->grant.identity_mask|=2;
        if(!engine_stopped_checkpoint(owner,"stopped_child_exit"))return 0;
    }
    if(!all)return 1;
    if(owner->members[1].state.pid) {
        int own[3]={0};int c=proc_listchildpids(owner->members[1].state.pid,own,sizeof(own));
        if(c<0&&owner->members[1].phase!=ENGINE_EXITED)return engine_failure(owner,"coverage_unknown","verifier child-edge census unavailable");
        if(c>0){owner->coverage_lost=owner->rollup_source_failed=1;return engine_failure(owner,"coverage_unknown","verifier created an unapproved child");}
        if(!engine_stopped_checkpoint(owner,"stopped_verifier_census"))return 0;
    }
    if(!final){*closed=1;return 1;}
    // A second, current-incarnation pass is required in this same grant. Only
    // after its final parent/group census may tentative endpoints be used.
    uint64_t users[2]={owner->members[0].user,owner->members[1].user};
    uint32_t versions[2]={owner->members[0].token.val[7],owner->members[1].token.val[7]};
    for(unsigned i=0;i<owner->slots;i++)if(owner->members[i].state.pid) {
        int ack;
        if(owner->members[i].phase==ENGINE_EXITED)ack=engine_member_exit(owner,i);
        else if(!engine_sample_member(owner,i,owner->members[i].state.pid,&ack))return 0;
        if(!ack){*closed=0;return 1;}
        if(owner->members[i].token.val[7]!=versions[i]) {
            owner->grant.end[i]=0;owner->grant.unknown_mask|=1u<<i;*closed=0;return 1;
        }
        if(owner->members[i].user!=users[i])return engine_failure(owner,"coverage_unknown","user work advanced after tentative stop acknowledgement");
        if(!engine_stopped_checkpoint(owner,"stopped_member_continuity"))return 0;
    }
    int final_group[3]={0},final_children[3]={0};
    int g=proc_listpgrppids(owner->ticket.pgid,final_group,sizeof(final_group));
    int c=proc_listchildpids(owner->ticket.pid,final_children,sizeof(final_children));
    if(g<0||g>2||c<0||c>1)return engine_failure(owner,"coverage_unknown","final stopped roster is incomplete");
    if(c&&!owner->members[1].state.pid) {
        owner->members[1].state.pid=final_children[0];owner->grant.unknown_mask|=2u;
        if(!owner->child_created){owner->child_created=1;owner->result->created++;owner->result->members_max=2;}
        if(owner->stopped_corroborating){owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","stopped final corroboration discovered an unbound member");}
        if(owner->settlement_quarantined){owner->coverage_lost=1;return engine_failure(owner,"coverage_unknown","quarantined final roster discovered an unbound child");}
        if(owner->stop_pending)return engine_failure(owner,"coverage_unknown","late unknown child cannot close pending terminal proof");
        owner->stop_requested=0;
        if(!engine_stop_group(owner))return 0;
        return 1; // Every possible interval remains open from the original t0.
    }
    for(int i=0;i<g;i++)if(final_group[i]!=owner->ticket.pid&&final_group[i]!=owner->members[1].state.pid)
        return engine_failure(owner,"coverage_unknown","new member appeared during mandatory continuity");
    if(c&&final_children[0]!=owner->members[1].state.pid)return engine_failure(owner,"coverage_unknown","new child appeared during closure");
    if(!engine_stopped_checkpoint(owner,"stopped_final_census"))return 0;
    if(owner->slots==2&&!owner->members[1].state.pid) {
        if(owner->members[0].child_user||owner->members[0].child_system) {
            owner->coverage_lost=1; // An observed unbound effect cannot become vacuous group settlement.
            return engine_failure(owner,"coverage_unknown","unobserved retired child cannot acquire a minted endpoint");
        }
        owner->grant.unknown_mask|=2u;owner->grant.ack_mask|=2;owner->grant.identity_mask|=2;
    }
    if(owner->stop_pending) {
        int done=0;
        if(!engine_direct_exited(owner,&done)||!done)return 1;
        if(owner->pending_identity_lost||owner->pending_final_image_mask!=((1u<<owner->slots)-1))
            return engine_failure(owner,"coverage_unknown","pending terminal source premise has unknown or pre-exec members");
        if(owner->slots==2&&!owner->members[1].state.pid)
            return engine_failure(owner,"coverage_unknown","pending terminal proof retains an unknown possible child");
        for(unsigned i=0;i<owner->slots;i++)if(owner->members[i].phase!=ENGINE_EXITED
            ||!owner->members[i].exit_abstime||!owner->members[i].token_known)
            return engine_failure(owner,"coverage_unknown","pending STOP lacks every original member terminal continuity");
        owner->stop_terminal_candidate=1;
    }
    if(!engine_qualified_rollup(owner,c))return 0;
    owner->grant.t_close=engine_now(owner,CLOCK_MONOTONIC);
    if(owner->stop_pending&&owner->direct_reaped&&owner->wait_observed)owner->stop_terminal_ns=owner->grant.t_close;
    for(unsigned i=0;i<owner->slots;i++)if(owner->grant.unknown_mask&(1u<<i))owner->grant.end[i]=owner->grant.t_close;
    owner->grant.roster_closed=1;owner->closed=1;*closed=1;return 1;
}
static int engine_pipe_io(struct engine_owner *owner) {
    uint64_t at=engine_now(owner,CLOCK_THREAD_CPUTIME_ID);unsigned char bytes[4096];
    if(owner->input>=0) {
        if(owner->sent==owner->stdin_len){int fd=owner->input;owner->input=-1;if(close(fd)!=0)return engine_failure(owner,"source_changed","owned stdin close unresolved");}
        else {
            size_t n=owner->stdin_len-owner->sent;if(n>sizeof(bytes))n=sizeof(bytes);
            // This controller alone owns STOP state. Recheck immediately at
            // EACH bounded write, including the next chunk of a partial input.
            // Pending/terminal observation permits output drain and charging,
            // but never new command payload for the consumed original grant.
            if(!owner->stop_pending&&!owner->stop_terminal&&!owner->accounting_pending) {
                ssize_t sent=write(owner->input,owner->stdin_bytes+owner->sent,n);
                if(sent>0)owner->sent+=(size_t)sent;
                else if(sent==0||(errno!=EAGAIN&&errno!=EWOULDBLOCK&&errno!=EINTR))return engine_failure(owner,"coverage_unknown","owned nonblocking stdin failed");
            }
        }
    }
    for(unsigned stream=0;stream<2;stream++) {
        int *fd=stream?&owner->errors:&owner->output;if(*fd<0)continue;
        ssize_t n=read(*fd,bytes,sizeof(bytes));
        if(n<0) {if(errno==EAGAIN||errno==EWOULDBLOCK||errno==EINTR)continue;return engine_failure(owner,"coverage_unknown","owned output pipe failed");}
        if(n==0){int old=*fd;*fd=-1;if(close(old)!=0)return engine_failure(owner,"source_changed","owned output close unresolved");continue;}
        if((size_t)n>ENGINE_CAPTURE_MAX-owner->out_len-owner->err_len)return engine_failure(owner,"resource_limit","shared two-MiB helper capture exhausted");
        if(!stream){memcpy(owner->capture+owner->out_len,bytes,(size_t)n);owner->out_len+=(size_t)n;}
        else {for(ssize_t i=0;i<n;i++)owner->capture[ENGINE_CAPTURE_MAX-1-owner->err_len-(size_t)i]=bytes[i];owner->err_len+=(size_t)n;}
    }
    uint64_t used=engine_interval(at,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    if(used==UINT64_MAX||UINT64_MAX-owner->pipe_cpu<used)return engine_failure(owner,"clock_unknown","pipe CPU clock overflow");
    owner->pipe_cpu+=used;return 1;
}
static int engine_memory(struct engine_owner *owner) {
    uint64_t group=owner->members[0].state.rss;
    if(!engine_safe_sum(group,owner->members[1].state.rss,&group))return engine_failure(owner,"clock_unknown","RSS sum overflow");
    engine_max(&owner->result->helper_group_rss_peak_bytes,group);
    struct rusage parent;
    if(getrusage(RUSAGE_SELF,&parent)!=0||parent.ru_maxrss<0)return engine_failure(owner,"coverage_unknown","private probe RSS unavailable");
    uint64_t rss=(uint64_t)parent.ru_maxrss;engine_max(&owner->result->probe_rss_peak_bytes,rss);
    if(group>16u*1024u*1024u||rss>96u*1024u*1024u-group)return engine_failure(owner,"resource_limit","existing group or combined RSS gate exceeded");
    return 1;
}
static int engine_write_record(struct engine_owner *owner,const char *bytes,int n) {
    if(n<=0||(size_t)n>=4096||owner->record_count>=ENGINE_RECORD_MAX||owner->ledger_bytes>ENGINE_CAPTURE_MAX
        ||(size_t)n>ENGINE_CAPTURE_MAX-owner->ledger_bytes)
        return engine_failure(owner,"resource_limit","fixed raw ledger record or two-MiB file bound exhausted");
    size_t offset=0;unsigned interrupted=0;
    while(offset<(size_t)n) {
        ssize_t written=write(owner->ledger,bytes+offset,(size_t)n-offset);
        if(written<0&&errno==EINTR&&interrupted++<4)continue;
        if(written<=0)return engine_failure(owner,"source_changed","owned raw ledger write unresolved");
        offset+=(size_t)written;
    }
    owner->ledger_bytes+=(size_t)n;owner->record_count++;return 1;
}
static uint64_t engine_unit_tail(const struct engine_unit_observation *unit) {
    if(unit->actual==UINT64_MAX||unit->supplied_actual==UINT64_MAX||unit->actual<unit->supplied_actual)return UINT64_MAX;
    return unit->actual-unit->supplied_actual;
}
static int engine_teardown_records(struct engine_owner *owner) {
    if(!owner->conservation_checked||owner->teardown_process_records_written)return 1;
    // Attempt once: an unresolved write remains a refusal, never a reason to
    // repeat a receipt or book the aggregate original wait4 cost again.
    owner->teardown_process_records_written=1;char bytes[4096];
    int n=snprintf(bytes,sizeof(bytes),
        "{\"kind\":\"exit_teardown_accounting\",\"sequence\":%u,\"form\":%u,"
        "\"T_ns\":%llu,\"direct_actual_reaps\":%u,\"qualified_child_reaps\":%u,\"N_reaped\":%u,\"N_times_T_ns\":%llu,"
        "\"raw_wait4_cpu_ns\":%llu,\"wait4_upper_ns\":%llu,\"independent_helper_credit_ns\":%llu,"
        "\"rounding_allowance_ns\":%llu,\"independent_plus_rounding_ns\":%llu,\"conservation_upper_ns\":%llu,\"excess_ns\":%llu,"
        "\"residual_required_known\":%u,\"raw_residual_ns\":%llu,\"residual_required_cost_ns\":%llu,"
        "\"residual_retained_cost_ns\":%llu,\"residual_debit_entries\":%llu,\"residual_cost_paid\":%u,"
        "\"residual_scope\":\"original_leader_wait4_subtree_once\",\"fixed_ceiling_is_credit\":false,\"runtime_approved\":false}\n",
        owner->ticket.sequence,owner->command,(unsigned long long)owner->exit_teardown_limit_ns,
        owner->teardown_direct_reaps,owner->teardown_child_reaps,owner->teardown_direct_reaps+owner->teardown_child_reaps,
        (unsigned long long)owner->exit_teardown_allowance_ns,(unsigned long long)owner->raw_wait4_cpu,(unsigned long long)owner->wait4_upper,
        (unsigned long long)owner->independent_cpu_credit,(unsigned long long)owner->rounding_allowance,(unsigned long long)owner->accounted_upper,
        (unsigned long long)owner->conservation_upper,(unsigned long long)owner->conservation_excess,owner->residual_required_known,
        (unsigned long long)owner->residual_raw,(unsigned long long)owner->residual_required_cost,
        (unsigned long long)owner->residual_retained_cost,(unsigned long long)owner->residual_debit_entries,owner->residual_cost_paid);
    if(!engine_write_record(owner,bytes,n))return 0;
    if(owner->teardown_direct_reaps) {
        n=snprintf(bytes,sizeof(bytes),
            "{\"kind\":\"exit_teardown_process\",\"sequence\":%u,\"pid\":%d,\"birth\":%llu,\"start\":%llu,\"pidversion\":%u,"
            "\"source\":\"original_own_wait4\",\"T_ns\":%llu,\"raw_user_ns\":%llu,\"raw_system_ns\":%llu,"
            "\"cost_scope\":\"original_leader_wait4_subtree_once\",\"residual_cost_ns\":%llu,\"residual_paid\":%u}\n",
            owner->ticket.sequence,owner->ticket.pid,(unsigned long long)owner->ticket.birth,(unsigned long long)owner->ticket.start,
            owner->members[0].token.val[7],(unsigned long long)owner->exit_teardown_limit_ns,
            (unsigned long long)owner->wait_user,(unsigned long long)owner->wait_system,
            (unsigned long long)owner->residual_required_cost,owner->residual_cost_paid);
        if(!engine_write_record(owner,bytes,n))return 0;
    }
    if(owner->teardown_child_reaps) {
        const struct engine_child_reap_proof *p=&owner->child_reap_proof;
        n=snprintf(bytes,sizeof(bytes),
            "{\"kind\":\"exit_teardown_process\",\"sequence\":%u,\"pid\":%d,\"birth\":%llu,\"start\":%llu,\"pidversion\":%u,"
            "\"source\":\"qualified_unique_leader_rollup\",\"T_ns\":%llu,\"child_exit_ns\":%llu,\"count_zero_ns\":%llu,"
            "\"leader_ACK_ns\":%llu,\"rollup_read_ns\":%llu,\"raw_rollup_ticks\":%llu,"
            "\"individual_wait4\":null,\"individual_residual_cost_ns\":null,\"cost_scope\":\"included_in_original_leader_subtree_once\"}\n",
            owner->ticket.sequence,p->pid,(unsigned long long)p->birth,(unsigned long long)p->start,p->version,
            (unsigned long long)owner->exit_teardown_limit_ns,(unsigned long long)p->exit_ns,(unsigned long long)p->zero_ns,
            (unsigned long long)p->ack_ns,(unsigned long long)p->read_ns,(unsigned long long)p->rollup_ticks);
        if(!engine_write_record(owner,bytes,n))return 0;
    }
    return 1;
}
static int engine_accounting_record(struct engine_owner *owner) {
    if(!owner->conservation_checked&&!owner->failed_units&&!owner->provisional_count)return 1;
    char bytes[4096];const struct engine_unit_observation *a=&owner->failed_unit_first,*b=&owner->failed_unit_last;
    int n=snprintf(bytes,sizeof(bytes),
        "{\"kind\":\"accounting\",\"phase\":\"accounting\",\"sequence\":%u,\"form\":%u,\"grant\":%u,"
        "\"conservation_known\":%u,\"conservation_checked\":%u,\"conservation_passed\":%u,"
        "\"startup_cpu_ns\":%llu,\"startup_charged\":%u,\"startup_timebase\":[%u,%u],\"stopped_units\":%u,\"stopped_max_cpu_ns\":%llu,"
        "\"charged_member_cpu_ns\":%llu,\"justified_late_cpu_ns\":%llu,\"member_post_cpu_ns\":%llu,"
        "\"counter_reads\":%llu,\"wait4_reads\":%llu,\"rounding_allowance_ns\":%llu,\"wait4_upper_ns\":%llu,"
        "\"post_exit_observed\":%u,\"post_exit_initial\":%u,\"post_exit_initial_source_valid\":%u,\"post_exit_ri_unavailable\":%u,\"post_exit_ri_attempted\":%u,\"post_exit_attempt_retained\":%u,\"post_exit_attempt_paid\":%u,"
        "\"initial_physical_basis\":%u,\"initial_own_WNOWAIT_known\":%u,\"initial_own_WNOWAIT_terminal\":%u,\"initial_BSD_known\":%u,\"initial_BSD_result\":%d,\"initial_BSD_errno\":%d,"
        "\"post_exit_before_ticks\":[%llu,%llu,%llu],\"post_exit_after_ticks\":[%llu,%llu,%llu],\"post_exit_delta_ns\":%llu,"
        "\"accounted_upper_ns\":%llu,\"conservation_excess_ns\":%llu,\"failed_units\":%llu,"
        "\"rollup_provisional_count\":%u,\"rollup_disposition\":\"%s\",\"rollup_source_failed\":%u,"
        "\"rollup_Q_difference_ns\":%llu,\"rollup_rest_difference_ns\":%llu,\"rollup_fallback_debt_ns\":%llu,"
        "\"rollup_worst_window\":[%u,%llu,%llu,%llu,%llu,%llu,%llu,%llu],"
        "\"retained_adjustment_cost_ns\":%llu,\"retained_adjustments\":%llu,\"booked_adjustments\":%llu,"
        "\"unit_fields\":[\"site\",\"line\",\"ordinary\",\"open_cpu\",\"end_cpu\",\"supplied_actual\",\"supplied_cost\",\"inclusive\",\"actual\",\"cost\",\"included_tail\",\"prior_poll_cpu\",\"poll_cpu\"],"
        "\"first_failed_unit\":[\"%s\",%u,%u,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu],"
        "\"last_failed_unit\":[\"%s\",%u,%u,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu,%llu],\"conditional_source_only\":true}\n",
        owner->ticket.sequence,owner->command,owner->grants,owner->conservation_known,owner->conservation_checked,owner->conservation_ok,
        (unsigned long long)owner->startup_cpu,owner->startup_charged,owner->startup_timebase_numer,owner->startup_timebase_denom,
        owner->stopped_units,(unsigned long long)owner->stopped_max_cpu,(unsigned long long)owner->charged_member_cpu,
        (unsigned long long)owner->justified_late_cpu,(unsigned long long)owner->justified_post_cpu,
        (unsigned long long)owner->counter_reads,(unsigned long long)owner->wait4_reads,(unsigned long long)owner->rounding_allowance,
        (unsigned long long)owner->wait4_upper,owner->post_exit_observed,owner->post_exit_initial,
        owner->post_exit_initial_source_valid,owner->post_exit_ri_unavailable,owner->post_exit_ri_attempted,owner->post_exit_attempt_retained,owner->post_exit_attempt_paid,
        owner->initial_physical.physical_basis,owner->initial_physical.own_wnowait_known,owner->initial_physical.own_wnowait_terminal,
        owner->initial_physical.bsd_known,owner->initial_physical.bsd_result,owner->initial_physical.bsd_error,
        (unsigned long long)owner->post_exit_before[0],(unsigned long long)owner->post_exit_before[1],(unsigned long long)owner->post_exit_before[2],
        (unsigned long long)owner->post_exit_after[0],(unsigned long long)owner->post_exit_after[1],(unsigned long long)owner->post_exit_after[2],
        (unsigned long long)owner->post_exit_delta,(unsigned long long)owner->accounted_upper,(unsigned long long)owner->conservation_excess,
        (unsigned long long)owner->failed_units,owner->provisional_count,
        !owner->provisional_count?"none":owner->provisional_revoked?"revoked_effective_window":owner->provisional_finalized?"certified_actual_wait4":"pending_actual_wait4",owner->rollup_source_failed,
        (unsigned long long)owner->provisional_quantum_difference,(unsigned long long)owner->provisional_rest_difference,(unsigned long long)owner->provisional_debt_added,
        owner->worst_provisional_grant,(unsigned long long)owner->worst_provisional_window.t0,
        (unsigned long long)owner->worst_provisional_window.end[0],(unsigned long long)owner->worst_provisional_window.end[1],
        (unsigned long long)owner->worst_provisional_window.windows,(unsigned long long)owner->worst_provisional_window.actual_cpu,
        (unsigned long long)owner->worst_provisional_window.controller,(unsigned long long)owner->worst_provisional_window.charge,
        (unsigned long long)private_cost.retained_adjustment_cost_ns,(unsigned long long)private_cost.retained_adjustments,(unsigned long long)private_cost.booked_adjustments,
        a->site?a->site:"none",a->line,a->ordinary,(unsigned long long)a->open,(unsigned long long)a->end,(unsigned long long)a->supplied_actual,
        (unsigned long long)a->supplied_cost,(unsigned long long)a->inclusive,(unsigned long long)a->actual,(unsigned long long)a->cost,
        (unsigned long long)engine_unit_tail(a),(unsigned long long)a->poll_before,(unsigned long long)a->poll_cpu,
        b->site?b->site:"none",b->line,b->ordinary,(unsigned long long)b->open,(unsigned long long)b->end,(unsigned long long)b->supplied_actual,
        (unsigned long long)b->supplied_cost,(unsigned long long)b->inclusive,(unsigned long long)b->actual,(unsigned long long)b->cost,
        (unsigned long long)engine_unit_tail(b),(unsigned long long)b->poll_before,(unsigned long long)b->poll_cpu);
    return engine_write_record(owner,bytes,n)&&engine_teardown_records(owner);
}
static int engine_record(struct engine_owner *owner,const char *phase) {
    if(owner->ledger<0)return engine_failure(owner,"settlement_unknown","owned raw ledger is unavailable");
    char bytes[4096],stages[1408];size_t stage_used=0;struct window_ledger *g=&owner->grant;mach_timebase_info_data_t scale={0,0};
    if(mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom)
        return engine_failure(owner,"clock_unknown","raw ledger timebase unavailable");
    for(unsigned i=0;i<ENGINE_STAGE_COUNT;i++) {
        const struct engine_stage_observation *s=&owner->closure_stage[i];
        int n=snprintf(stages+stage_used,sizeof(stages)-stage_used,"%s[%llu,%llu,%llu,%llu,%llu,%llu,%llu]",i?",":"",
            (unsigned long long)s->wall_begin,(unsigned long long)s->wall_end,(unsigned long long)s->cpu_begin,(unsigned long long)s->cpu_end,
            (unsigned long long)s->calls,(unsigned long long)s->max_wall,(unsigned long long)s->max_cpu);
        if(n<=0||(size_t)n>=sizeof(stages)-stage_used)return engine_failure(owner,"resource_limit","fixed closure stage diagnostics exhausted");
        stage_used+=(size_t)n;
    }
    int n=snprintf(bytes,sizeof(bytes),
        "{\"phase\":\"%s\",\"sequence\":%u,\"form\":%u,\"grant\":%u,\"leader\":%d,\"child\":%d,"
        "\"birth\":[%llu,%llu],\"token_version\":[%u,%u],\"t0\":%llu,\"end\":[%llu,%llu],\"close\":%llu,"
        "\"W\":%llu,\"A\":%llu,\"C\":%llu,\"Q\":%llu,\"basis\":\"%s\",\"slot_A_ns\":[%llu,%llu],"
        "\"baseline_ticks\":[%llu,%llu],\"endpoint_ticks\":[%llu,%llu],\"endpoint_wait4_ns\":[%llu,%llu],"
        "\"baseline_read_ns\":[%llu,%llu],\"counter_read_ns\":[%llu,%llu],\"baseline_version\":[%u,%u],\"counter_version\":[%u,%u],"
        "\"baseline_mask\":%u,\"counter_mask\":%u,\"post_ack_mask\":%u,\"wait4_mask\":%u,\"counter_identity_mask\":%u,\"conversion_mask\":%u,"
        "\"counter_kind\":[%u,%u],\"rollup_mask\":%u,\"rollup_unique\":%u,\"rollup_child_retired\":%u,"
        "\"rollup_child_exit_ns\":%llu,\"rollup_child_zero_ns\":%llu,\"rollup_leader_ack_ns\":%llu,\"rollup_read_ns\":%llu,"
        "\"live_child_cpu_ns\":%llu,\"rollup_child_cpu_ns\":%llu,\"wait4_subtree_ns\":%llu,"
        "\"pipe_cpu_subset\":%llu,\"ack_mask\":%u,\"unknown_mask\":%u,"
        "\"closed\":%u,\"failed\":%u,\"direct_reaped\":%u,\"native_attempted\":%d,\"native_errno\":%d,"
        "\"native_cpu\":%llu,\"native_wall\":%llu,\"ri_cpu_ticks\":[%llu,%llu,%llu],\"timebase\":[%u,%u],"
        "\"wait4_observed\":%u,\"wait4_user_ns\":%llu,\"wait4_system_ns\":%llu,\"wait4_rss\":%llu,"
        "\"stop_attempts\":%u,\"stop_result\":%d,\"stop_errno\":%d,\"stop_call_ns\":%llu,"
        "\"stop_pending\":%u,\"stop_terminal\":%u,\"stop_terminal_ns\":%llu,"
        "\"child_status_known\":%u,\"child_status_raw\":%u,\"hold_rounds\":%u,\"hold_unknown_units\":%llu,"
        "\"hold_unbooked_units\":%llu,\"hold_retained_cost_ns\":%llu,\"hold_retained_raw_ns\":%llu,\"hold_retained_unknown_units\":%llu,"
        "\"hold_retained_overflow\":%u,\"hold_debt_charged_ns\":%llu,\"hold_debt_paid_ns\":%llu,"
        "\"owner_lost\":%u,\"closure_stage_order\":[\"pipe\",\"direct_exit\",\"stop\",\"roster\",\"actual\"],"
        "\"closure_stage_fields\":[\"wall_begin\",\"wall_end\",\"cpu_begin\",\"cpu_end\",\"calls\",\"max_wall\",\"max_cpu\"],"
        "\"closure_stages\":[%s],\"closure_limit_failed\":%u,\"conditional_source_only\":true}\n",
        phase,owner->ticket.sequence,owner->command,owner->grants,owner->ticket.pid,owner->members[1].state.pid,
        (unsigned long long)owner->members[0].state.birth,(unsigned long long)owner->members[1].state.birth,
        owner->members[0].token.val[7],owner->members[1].token.val[7],(unsigned long long)g->t0,
        (unsigned long long)g->end[0],(unsigned long long)g->end[1],(unsigned long long)g->t_close,
        (unsigned long long)g->windows,(unsigned long long)g->actual_cpu,(unsigned long long)g->controller,
        (unsigned long long)g->charge,g->measured?"measured":"window",(unsigned long long)g->slot_cpu[0],(unsigned long long)g->slot_cpu[1],
        (unsigned long long)g->baseline_ticks[0],(unsigned long long)g->baseline_ticks[1],
        (unsigned long long)g->endpoint_ticks[0],(unsigned long long)g->endpoint_ticks[1],
        (unsigned long long)g->endpoint_wait4_ns[0],(unsigned long long)g->endpoint_wait4_ns[1],
        (unsigned long long)g->baseline_read[0],(unsigned long long)g->baseline_read[1],
        (unsigned long long)g->counter_read[0],(unsigned long long)g->counter_read[1],
        g->baseline_version[0],g->baseline_version[1],g->counter_version[0],g->counter_version[1],
        g->baseline_mask,g->counter_mask,g->post_ack_mask,g->wait4_mask,g->counter_identity_mask,g->conversion_mask,
        g->counter_kind[0],g->counter_kind[1],g->rollup_mask,g->rollup_unique,g->rollup_child_retired,
        (unsigned long long)g->rollup_child_exit_ns,(unsigned long long)g->rollup_child_zero_ns,
        (unsigned long long)g->rollup_leader_ack_ns,(unsigned long long)g->rollup_read_ns,
        (unsigned long long)g->live_child_cpu_ns,(unsigned long long)g->rollup_child_cpu_ns,(unsigned long long)g->wait4_subtree_ns,
        (unsigned long long)owner->pipe_cpu,g->ack_mask,g->unknown_mask,
        g->roster_closed,owner->failed,owner->direct_reaped,owner->native.attempted,owner->native.error,
        (unsigned long long)owner->native.cpu,(unsigned long long)owner->native.wall,
        (unsigned long long)owner->members[0].state.raw_rusage_cpu,(unsigned long long)owner->members[1].state.raw_rusage_cpu,
        (unsigned long long)owner->members[0].state.raw_waited,scale.numer,scale.denom,owner->wait_observed,
        (unsigned long long)owner->wait_user,(unsigned long long)owner->wait_system,(unsigned long long)owner->wait_rss,
        owner->stop_attempts,owner->stop_result,owner->stop_errno,(unsigned long long)owner->stop_call_ns,
        owner->stop_pending,owner->stop_terminal,(unsigned long long)owner->stop_terminal_ns,
        owner->members[1].exit_status_known,owner->members[1].exit_status,owner->hold_rounds,
        (unsigned long long)owner->hold_unknown_units,(unsigned long long)private_cost.retained_units,
        (unsigned long long)private_cost.retained_cost_ns,(unsigned long long)private_cost.retained_raw_ns,
        (unsigned long long)private_cost.retained_unknown_units,private_cost.retained_overflow,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,owner->owner_lost,stages,owner->closure_limit_failed);
    return engine_write_record(owner,bytes,n)&&engine_accounting_record(owner);
}
static void engine_failed_unit(struct engine_owner *owner,const struct engine_unit_observation *observation) {
    if(owner->failed_units==UINT64_MAX){(void)engine_failure(owner,"clock_unknown","failed-unit diagnostic count overflow");return;}
    if(!owner->failed_units)owner->failed_unit_first=*observation;
    owner->failed_units++;owner->failed_unit_last=*observation;
}
static int engine_charge_at(struct engine_owner *owner,uint64_t actual,uint64_t cost,int ordinary,const char *site,unsigned line) {
    uint64_t end=engine_now(owner,CLOCK_THREAD_CPUTIME_ID);
    uint64_t inclusive=engine_interval(private_cost.open_cpu,end);
    struct engine_unit_observation observation={.site=site,.line=line,.ordinary=(unsigned)ordinary,.open=private_cost.open_cpu,.end=end,
        .supplied_actual=actual,.supplied_cost=cost,.inclusive=inclusive,.poll_before=owner->last_poll_cpu,.poll_cpu=private_cost.rest_poll_cpu};
    owner->last_poll_cpu=private_cost.rest_poll_cpu;
    if(inclusive==UINT64_MAX||actual==UINT64_MAX||cost==UINT64_MAX)actual=cost=UINT64_MAX;
    else if(inclusive>actual) {
        uint64_t extra=inclusive-actual;actual=inclusive;
        if(!engine_safe_sum(cost,extra,&cost))cost=UINT64_MAX;
    }
    private_cost.last_unit=actual;
    owner->last_charged_cost=cost;
    observation.actual=actual;observation.cost=cost;
    if(ordinary)engine_max(&owner->result->ordinary_max_ns,actual);
    int scheduled=private_schedule_cost(cost);
    private_cost.open_cpu=end;
    if(!scheduled||!private_pay_pending()) {
        engine_failed_unit(owner,&observation);
        return private_unit_outcome(engine_failure(owner,"clock_unknown","shared unit debt could not be paid"));
    }
    engine_max(&owner->result->debt_charged_ns,private_cost.debt);
    engine_max(&owner->result->debt_paid_ns,private_cost.debt_paid);
    if(cost==UINT64_MAX||actual==UINT64_MAX) {
        engine_failed_unit(owner,&observation);
        return private_unit_outcome(engine_failure(owner,"clock_unknown","closed engine unit has unknown CPU"));
    }
    if(ordinary&&(actual>1000000||cost>1000000)) {
        engine_failed_unit(owner,&observation);
        return private_unit_outcome(engine_failure(owner,"resource_limit","ordinary engine unit exceeds one millisecond"));
    }
    if(!private_cost.custody_hold&&engine_now(owner,CLOCK_MONOTONIC)>=owner->context->outer->deadline) {
        engine_failed_unit(owner,&observation);
        return private_unit_outcome(engine_failure(owner,"deadline","original private deadline expired"));
    }
    return 1;
}
#define engine_charge(owner,actual,cost,ordinary) engine_charge_at((owner),(actual),(cost),(ordinary),__func__,__LINE__)
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static uint64_t source_known_stopped_stage_cpu;
#endif
static int engine_stopped_checkpoint(struct engine_owner *owner,const char *stage) {
    if(!owner->stopped_corroborating)return 1;
    if(!owner->closed||owner->running||owner->accounting_pending||owner->stop_pending
        ||(owner->grant.t0&&!owner->grant_accounted))
        return engine_failure(owner,"coverage_unknown","stopped checkpoint cannot split an original open grant");
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_stopped_stage_cpu) {
        uint64_t start=engine_now(owner,CLOCK_THREAD_CPUTIME_ID);
        while(engine_interval(start,engine_now(owner,CLOCK_THREAD_CPUTIME_ID))<source_known_stopped_stage_cpu){}
    }
#endif
    // One bounded census or one member observation per unit gives nominal
    // <=0.5ms headroom. An unexpectedly costly primitive still hits the SAME
    // 1ms hard gate, with its full cost/rest; no historical span is split.
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    int paid=engine_charge_at(owner,used,used,1,stage,__LINE__);
    if(owner->stopped_units==UINT_MAX)return engine_failure(owner,"clock_unknown","stopped stage count overflow");
    owner->stopped_units++;
    if(private_cost.last_unit>owner->stopped_max_cpu)owner->stopped_max_cpu=private_cost.last_unit;
    engine_sum(&owner->result->post_cpu_ns,private_cost.last_unit);
    return paid;
}
static int engine_stopped_corroboration(struct engine_owner *owner) {
    if(!owner->closed||owner->running||owner->accounting_pending||owner->stop_pending||(owner->grant.t0&&!owner->grant_accounted))
        return engine_failure(owner,"coverage_unknown","corroboration requires already closed original members");
    uint64_t cpu_before,user_before,system_before;int paid=0;
    owner->stopped_corroborating=1;
    if(!engine_actual(owner,&cpu_before,&user_before,&system_before))goto finished;
    if(!engine_stopped_checkpoint(owner,"stopped_entry"))goto finished;
    int closed;
    if(!engine_roster(owner,1,&closed)||!closed||!engine_memory(owner)) {
        (void)engine_failure(owner,"coverage_unknown","post-closure seal could not be corroborated");goto finished;
    }
    if(!engine_stopped_checkpoint(owner,"stopped_memory"))goto finished;
    uint64_t cpu,user,system;if(!engine_actual(owner,&cpu,&user,&system))goto finished;
    if(user!=user_before||system<system_before||cpu<owner->accounted_cpu) {
        (void)engine_failure(owner,"coverage_unknown","unexplained stopped user or resource movement");goto finished;
    }
    uint64_t late,ticks;
    if(!engine_actual_delta(owner,owner->accounted_ticks,&late,&ticks))goto finished;
    int initial=!owner->startup_known;
    if(initial) {
        mach_timebase_info_data_t scale;
        if(mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom) {
            (void)engine_failure(owner,"clock_unknown","initial stopped timebase unavailable");goto finished;
        }
        owner->startup_timebase_numer=scale.numer;owner->startup_timebase_denom=scale.denom;
        owner->startup_cpu=cpu;owner->startup_known=owner->conservation_known=1;
    }
    owner->accounted_ticks=ticks;owner->accounted_cpu=cpu;owner->accounted_user=user;owner->accounted_system=system;
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID)),cost;
    if(!engine_safe_sum(used,late,&cost))cost=UINT64_MAX;
    engine_sum(&owner->result->post_cpu_ns,used);engine_sum(&owner->result->late_kernel_ns,late);
    uint64_t booked=private_cost.closed_units;paid=engine_charge(owner,used,cost,1);
    if(private_cost.closed_units!=booked+1||cost==UINT64_MAX)owner->conservation_known=0;
    else if(initial)owner->startup_charged=1;
    else if(!initial)(void)engine_credit_member(owner,late,1);
    engine_observed_counters(owner);
    if(paid&&engine_record(owner,"closed"))paid=engine_stopped_checkpoint(owner,"stopped_ledger");
    else paid=0;
finished:
    owner->stopped_corroborating=0;return paid;
}
static int engine_window_cost(struct engine_owner *owner,uint64_t end,int closed) {
    struct window_ledger *g=&owner->grant;enum ledger_result verdict;
    if(closed)verdict=window_charge(g);
    else {
        struct window_ledger outstanding=*g;outstanding.slots=owner->slots;outstanding.t_close=end;
        for(unsigned i=0;i<owner->slots;i++)outstanding.end[i]=end;
        verdict=window_amounts(&outstanding,0); // An open grant NEVER acquires measured authority.
        g->windows=outstanding.windows;g->controller=outstanding.controller;g->charge=outstanding.charge;g->measured=0;
    }
    if(verdict==LEDGER_CLOCK)g->windows=g->controller=g->charge=UINT64_MAX;
    engine_sum(&owner->result->W_ns,g->windows);engine_sum(&owner->result->A_ns,g->actual_cpu);
    engine_sum(&owner->result->C_ns,g->controller);engine_max(&owner->result->Q_max_ns,g->charge);
    engine_max(&owner->result->controller_max_ns,g->controller);return (int)verdict;
}
static int engine_slot_ticks(struct engine_owner *owner,unsigned index,uint64_t *ticks) {
    if(index==0)*ticks=owner->members[0].state.raw_rusage_cpu;
    else *ticks=owner->members[1].state.raw_rusage_cpu>owner->members[0].state.raw_waited
        ?owner->members[1].state.raw_rusage_cpu:owner->members[0].state.raw_waited;
    return *ticks!=UINT64_MAX;
}
static int engine_credit_member(struct engine_owner *owner,uint64_t value,int late) {
    uint64_t *sum=late?&owner->justified_late_cpu:&owner->charged_member_cpu;
    if(value==UINT64_MAX||!engine_safe_sum(*sum,value,sum)) {
        owner->conservation_known=0;
        return engine_failure(owner,"clock_unknown","charged helper conservation ledger overflow or unknown");
    }
    return 1;
}
static void engine_observed_counters(struct engine_owner *owner) {
    // Independent RAW observations. Neither charged/ns totals nor wait4's
    // final value may rewrite the baseline for justified late CPU.
    owner->observed_raw[0]=owner->members[0].state.raw_rusage_cpu;
    owner->observed_raw[1]=owner->members[1].state.raw_rusage_cpu;
    owner->observed_raw[2]=owner->members[0].state.raw_waited;
    owner->observed_user[0]=owner->members[0].user;owner->observed_user[1]=owner->members[1].user;
    owner->observed_user[2]=owner->members[0].child_user;
    owner->observed_system[0]=owner->members[0].system;owner->observed_system[1]=owner->members[1].system;
    owner->observed_system[2]=owner->members[0].child_system;
}
static int engine_paid_stopped_roster(struct engine_owner *owner,int *closed) {
    if(!owner->closed||owner->running||owner->stop_pending||owner->accounting_pending
        ||(owner->grant.t0&&!owner->grant_accounted))
        return engine_failure(owner,"coverage_unknown","paid stopped roster cannot split an original open grant");
    unsigned prior=owner->stopped_corroborating;owner->stopped_corroborating=1;
    int proved=engine_roster(owner,1,closed);owner->stopped_corroborating=prior;return proved;
}
static int engine_grant_baseline(struct engine_owner *owner) {
    // The bounded already-stopped roster stages pay separately. The final
    // raw baseline/kernel observation is ONE paid unit; never recursively
    // re-observe after its rest and pretend that its sample was at CONT.
    int closed=0;
    if(!engine_paid_stopped_roster(owner,&closed)||!closed)return engine_failure(owner,"coverage_unknown","pre-CONT original stopped roster unavailable");
    uint64_t cpu,user,system,ticks,u,s,late=0;
    if(!engine_actual(owner,&cpu,&user,&system)||!engine_actual_ticks(owner,&ticks,&u,&s))return 0;
    int initial=!owner->startup_known;
    if(initial) {
        mach_timebase_info_data_t scale;
        if(mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom)
            return engine_failure(owner,"clock_unknown","initial baseline timebase unavailable");
        owner->startup_timebase_numer=scale.numer;owner->startup_timebase_denom=scale.denom;
        owner->startup_known=owner->conservation_known=1;owner->startup_cpu=cpu;late=cpu;
    } else {
        if(user!=owner->accounted_user||system<owner->accounted_system||ticks<owner->accounted_ticks)
            return engine_failure(owner,"coverage_unknown","stopped baseline has unexplained user or reversed CPU");
        if(!engine_ticks_ns(ticks-owner->accounted_ticks,&late))return engine_failure(owner,"clock_unknown","pre-CONT stopped CPU conversion unknown");
    }
    owner->accounted_cpu=cpu;owner->accounted_ticks=ticks;owner->accounted_user=user;owner->accounted_system=system;
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID)),cost;
    if(!engine_safe_sum(used,late,&cost))cost=UINT64_MAX;
    uint64_t booked=private_cost.closed_units;int paid=engine_charge(owner,used,cost,1);
    if(private_cost.closed_units!=booked+1||cost==UINT64_MAX)owner->conservation_known=0;
    else if(initial)owner->startup_charged=1;
    else if(!initial)(void)engine_credit_member(owner,late,1);
    engine_observed_counters(owner);
    engine_sum(&owner->result->post_cpu_ns,used);engine_sum(&owner->result->late_kernel_ns,late);
    return paid&&owner->members[0].phase!=ENGINE_EXITED;
}
static int engine_counter_end(struct engine_owner *owner) {
    struct window_ledger *g=&owner->grant;uint64_t sum=0;
    g->counter_mask=g->post_ack_mask=g->wait4_mask=g->counter_identity_mask=g->conversion_mask=0;
    g->counter_kind[0]=g->counter_kind[1]=COUNTER_SOURCE_NONE;g->live_child_cpu_ns=g->rollup_child_cpu_ns=0;
    for(unsigned i=0;i<owner->slots;i++) {
        struct engine_member *m=&owner->members[i];uint64_t ticks,delta;
        if(!engine_slot_ticks(owner,i,&ticks))return engine_failure(owner,"coverage_unknown","per-slot grant CPU unknown");
        if(i==1&&g->rollup_mask==2) {
            uint64_t rolled=owner->members[0].state.raw_waited;
            if(rolled<g->baseline_ticks[i]||rolled<m->state.raw_rusage_cpu) {
                owner->rollup_source_failed=1;g->force_window=1;g->rollup_mask=0;
                (void)engine_failure(owner,"coverage_unknown","qualified child rollup reverses its retained raw lifetime floor");
            } else {ticks=rolled;g->counter_kind[i]=COUNTER_SOURCE_LEADER_ROLLUP;}
        }
        if(ticks<g->baseline_ticks[i])return engine_failure(owner,"coverage_unknown","per-slot grant CPU reversed");
        g->endpoint_ticks[i]=ticks;g->counter_read[i]=m->counter_read_ns;g->counter_version[i]=m->token.val[7];
        if(i==1&&g->rollup_mask==2)g->counter_read[i]=g->rollup_read_ns;
        if(!engine_scaled_ticks(ticks-g->baseline_ticks[i],g->timebase_numer,g->timebase_denom,1,&delta))
            return engine_failure(owner,"clock_unknown","per-slot grant timebase unknown");
        if(m->state.pid&&m->token_known) {
            g->counter_mask|=1u<<i;g->conversion_mask|=1u<<i;
            if(m->token.val[7]==g->baseline_version[i])g->counter_identity_mask|=1u<<i;
            if(m->phase!=ENGINE_EXITED&&m->counter_post_ack&&m->counter_read_ns>=g->end[i]) {
                g->post_ack_mask|=1u<<i;g->counter_kind[i]=COUNTER_SOURCE_POST_ACK_RI;
            }
        }
        if(i==0&&(owner->slots==1||g->rollup_mask==2)&&owner->wait_observed&&owner->direct_reaped) {
            uint64_t wait,baseline;
            if(!engine_safe_sum(owner->wait_user,owner->wait_system,&wait)||!engine_safe_sum(wait,2000,&wait)
                ||!engine_scaled_ticks(g->baseline_ticks[i],g->timebase_numer,g->timebase_denom,0,&baseline)||wait<baseline)
                return engine_failure(owner,"coverage_unknown","retained wait4 cannot bound the original baseline");
            g->wait4_subtree_ns=wait;
            if(owner->slots==2) {
                uint64_t child_floor;
                if(!engine_scaled_ticks(owner->members[0].state.raw_waited,g->timebase_numer,g->timebase_denom,0,&child_floor)||wait<child_floor)
                    return engine_failure(owner,"coverage_unknown","own wait4 cannot separate the uniquely qualified child");
                wait-=child_floor;
                if(wait<baseline)return engine_failure(owner,"coverage_unknown","own leader self wait4 upper bound reverses baseline");
            }
            delta=wait-baseline;g->endpoint_wait4_ns[i]=wait;g->counter_read[i]=owner->wait_read_ns;
            g->wait4_mask|=1u<<i;g->counter_mask|=1u<<i;g->conversion_mask|=1u<<i;
            g->post_ack_mask&=~(1u<<i);g->counter_kind[i]=COUNTER_SOURCE_OWN_WAIT4;
        }
        g->slot_cpu[i]=delta;
        if(!engine_safe_sum(sum,delta,&sum))return engine_failure(owner,"clock_unknown","per-slot measured CPU sum overflow");
    }
    if(owner->slots==2&&owner->direct_reaped&&owner->wait_observed&&g->rollup_mask!=2) {
        // Actual own wait4 can bound the WHOLE original terminal grant without
        // pretending it is the verifier's per-slot wait4. Missing per-slot
        // terminal provenance therefore still keeps WINDOW basis.
        uint64_t raw_baseline,baseline,wait;
        if(!engine_safe_sum(g->baseline_ticks[0],g->baseline_ticks[1],&raw_baseline)
            ||!engine_scaled_ticks(raw_baseline,g->timebase_numer,g->timebase_denom,0,&baseline)
            ||!engine_safe_sum(owner->wait_user,owner->wait_system,&wait)||!engine_safe_sum(wait,2000,&wait)||wait<baseline)
            return engine_failure(owner,"coverage_unknown","retained subtree wait4 cannot bound terminal original grant");
        g->wait4_subtree_ns=wait;
        if(wait-baseline>sum)sum=wait-baseline;
    }
    if(owner->slots==2) {
        if(g->rollup_mask==2)g->rollup_child_cpu_ns=g->slot_cpu[1];
        else g->live_child_cpu_ns=g->slot_cpu[1];
    }
    g->actual_cpu=sum;return 1;
}
static int engine_provisional_add(struct engine_owner *owner) {
    struct window_ledger *g=&owner->grant;
    if(!g->measured||g->rollup_mask!=2)return 1;
    struct window_ledger fallback=*g;
    enum ledger_result result=window_amounts(&fallback,0);
    uint64_t difference,rest_difference,measured_rest=g->charge>1000000?g->charge:1000000;
    uint64_t window_rest=fallback.charge>1000000?fallback.charge:1000000;
    if(result==LEDGER_CLOCK||fallback.charge<g->charge||window_rest<measured_rest
        ||owner->provisional_count>=ENGINE_RECORD_MAX
        ||!engine_safe_sum(owner->provisional_quantum_difference,fallback.charge-g->charge,&difference)
        ||!engine_safe_sum(owner->provisional_rest_difference,window_rest-measured_rest,&rest_difference)) {
        // No bounded provisional entry means no measured allowance. The
        // current full window is charged now, with the original endpoints.
        g->force_window=1;(void)window_amounts(g,0);owner->rollup_source_failed=1;
        return engine_failure(owner,"coverage_unknown","qualified rollup provisional inventory exhausted or unknown");
    }
    owner->provisional_count++;owner->provisional_quantum_difference=difference;owner->provisional_rest_difference=rest_difference;
    struct engine_member *child=&owner->members[1];
    if(owner->command==12&&owner->verified_descendants==1&&owner->child_created&&child->phase==ENGINE_EXITED
        &&child->token_known&&child->state.pid>0&&child->state.birth&&child->state.start_absolute&&child->exit_abstime
        &&owner->git_signature&&owner->git_signature->bound&&owner->git_signature->pid==child->state.pid
        &&owner->git_signature->pidversion==child->token.val[7]) {
        owner->child_reap_proof=(struct engine_child_reap_proof){.pid=child->state.pid,.version=child->token.val[7],.valid=1,
            .birth=child->state.birth,.start=child->state.start_absolute,.exit_ns=g->rollup_child_exit_ns,.zero_ns=g->rollup_child_zero_ns,
            .ack_ns=g->rollup_leader_ack_ns,.read_ns=g->rollup_read_ns,.rollup_ticks=owner->members[0].state.raw_waited};
    }
    if(owner->provisional_count==1||fallback.charge>owner->worst_provisional_window.charge) {
        owner->worst_provisional_window=fallback;owner->worst_provisional_grant=owner->grants;
    }
    return 1;
}
static int engine_revoke_provisional(struct engine_owner *owner) {
    if(!owner->provisional_count||owner->provisional_revoked)return 1;
    uint64_t difference=owner->provisional_rest_difference;
    if(difference>UINT64_MAX/9||!private_retain_debt_adjustment(difference))return 0;
    // Once retained, the shared owner/barrier owns these arrears even if
    // scheduling or payment fails. Never enqueue them a second time.
    owner->provisional_revoked=1;owner->provisional_finalized=0;owner->provisional_debt_added=difference*9;
    engine_max(&owner->result->Q_max_ns,owner->worst_provisional_window.charge);
    if(owner->worst_provisional_window.charge>1000000)
        (void)engine_failure(owner,"resource_limit","revoked provisional rollup retains original excessive full-window charge");
    return private_book_retained_cost()&&private_pay_pending();
}
static int engine_finish_grant(struct engine_owner *owner) {
    if(owner->grant_accounted)return !owner->failed;
    struct window_ledger *g=&owner->grant;uint64_t cpu=UINT64_MAX,user=UINT64_MAX,system=UINT64_MAX,ticks=UINT64_MAX,u,s;
    int stage=engine_stage_begin(owner,ENGINE_STAGE_ACTUAL);
    if(!engine_actual(owner,&cpu,&user,&system)||!engine_actual_ticks(owner,&ticks,&u,&s)||!engine_counter_end(owner))g->actual_cpu=UINT64_MAX;
    if(!stage||!engine_stage_end(owner,ENGINE_STAGE_ACTUAL))g->actual_cpu=UINT64_MAX;
    g->t_close=engine_now(owner,CLOCK_MONOTONIC);
    if(owner->stop_pending&&owner->direct_reaped&&owner->wait_observed)owner->stop_terminal_ns=g->t_close;
    for(unsigned i=0;i<owner->slots;i++)if(g->unknown_mask&(1u<<i))g->end[i]=g->t_close;
    g->cpu_close=engine_now(owner,CLOCK_THREAD_CPUTIME_ID);g->pipe_actual=g->pipe_charged=0;
    enum ledger_result verdict=(enum ledger_result)engine_window_cost(owner,g->t_close,1);
    if(verdict!=LEDGER_OK)engine_failure(owner,verdict==LEDGER_LIMIT?"resource_limit":"coverage_unknown","canonical member-window closure refused");
    (void)engine_provisional_add(owner);
    owner->running=0;owner->accounting_pending=0;owner->grant_accounted=1;
    owner->accounted_cpu=cpu;owner->accounted_ticks=ticks;owner->accounted_user=user;owner->accounted_system=system;
    uint64_t prepaid=owner->settlement_prepaid_cpu;
    uint64_t actual=g->controller>=prepaid?g->controller-prepaid:UINT64_MAX;
    uint64_t cost=g->charge>=prepaid?g->charge-prepaid:UINT64_MAX;
    uint64_t booked=private_cost.closed_units;int paid=engine_charge(owner,actual,cost,1);
    if(private_cost.closed_units!=booked+1||cost==UINT64_MAX)owner->conservation_known=0;
    else (void)engine_credit_member(owner,g->actual_cpu,0);
    engine_observed_counters(owner);
    return paid&&!owner->failed;
}
static int engine_admit_grant(struct engine_owner *owner) {
    uint64_t now=engine_now(owner,CLOCK_MONOTONIC);
    if(now==UINT64_MAX||owner->failed)return 0;
    if(owner->stop_pending||owner->accounting_pending)return engine_failure(owner,"coverage_unknown","no CONT or next work while consumed STOP or terminal accounting awaits settlement");
    if(now<private_cost.rest_until)return engine_failure(owner,"coverage_unknown","CONT refused while additive debt remains");
    if(engine_cancel_signal||owner->foreground||now>=owner->context->outer->deadline)
        return engine_failure(owner,now>=owner->context->outer->deadline?"deadline":"cancelled","CONT revoked before the original effect");
    return 1;
}
static int engine_closure_bound(struct engine_owner *owner) {
    uint64_t now=engine_now(owner,CLOCK_MONOTONIC),cpu=engine_now(owner,CLOCK_THREAD_CPUTIME_ID),windows=0;
    if(now==UINT64_MAX||cpu==UINT64_MAX||cpu<owner->grant.cpu_start)return engine_failure(owner,"clock_unknown","closure lower-bound clock unavailable");
    for(unsigned i=0;i<owner->slots;i++) {
        uint64_t end=owner->grant.end[i]?owner->grant.end[i]:now;
        if(end<owner->grant.t0||!engine_safe_sum(windows,end-owner->grant.t0,&windows))
            return engine_failure(owner,"clock_unknown","closure lower-bound interval unknown");
    }
    if(windows>25000000||cpu-owner->grant.cpu_start>1000000) {
        owner->closure_limit_failed=1;
        return engine_failure(owner,"resource_limit",windows>25000000?"current member-window wall exceeds twenty-five milliseconds":"current active controller exceeds one millisecond");
    }
    return 1;
}
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static unsigned source_known_closure_over_cpu;
#endif
static int engine_grant(struct engine_owner *owner) {
    if(owner->failed||owner->retiring||!owner->closed||owner->stop_pending||owner->accounting_pending||owner->direct_reaped||private_failed||engine_cancel_signal)
        return engine_failure(owner,"cancelled","grant refused after revocation or unfinished closure");
    if(!strcmp(owner->context->outer->case_name,"debt_overlap")&&!owner->case_fired) {
        owner->case_fired=1;
        if(!private_schedule_cost(1))return engine_failure(owner,private_error,private_detail);
        if(engine_admit_grant(owner))return engine_failure(owner,"coverage_unknown","debt-overlap control lost its pending-debt premise");
        return 0; // A rejected proposal never issues CONT.
    }
    if(!private_pay_pending()||!engine_admit_grant(owner)||!engine_grant_baseline(owner)||!engine_admit_grant(owner))return 0;
    uint64_t cpu_before,user_before,system_before;if(!engine_actual(owner,&cpu_before,&user_before,&system_before))return 0;
    owner->grant_cpu_before=cpu_before;
    uint64_t tick_u,tick_s;
    if(!engine_actual_ticks(owner,&owner->grant_tick_before,&tick_u,&tick_s))return 0;
    (void)user_before;(void)system_before;
    memset(&owner->grant,0,sizeof(owner->grant));owner->grant.operation=owner->ticket.start;
    owner->grant_accounted=owner->accounting_pending=0;
    memset(owner->closure_stage,0,sizeof(owner->closure_stage));owner->closure_limit_failed=0;
    owner->grant.sequence=owner->ticket.sequence;owner->grant.slots=owner->slots;
    mach_timebase_info_data_t scale;
    if(mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom)
        return engine_failure(owner,"clock_unknown","pre-CONT timebase unavailable");
    owner->grant.timebase_numer=scale.numer;owner->grant.timebase_denom=scale.denom;
    for(unsigned i=0;i<owner->slots;i++) {
        struct engine_member *m=&owner->members[i];
        if(!engine_slot_ticks(owner,i,&owner->grant.baseline_ticks[i]))return engine_failure(owner,"clock_unknown","pre-CONT slot counter unavailable");
        owner->grant.baseline_read[i]=m->counter_read_ns;owner->grant.baseline_version[i]=m->token.val[7];
        if(m->state.pid&&m->token_known&&m->state.start_absolute&&m->counter_read_ns)owner->grant.baseline_mask|=1u<<i;
    }
    // This is the fully-bound conditional source premise, not a runtime grant
    // or a theorem that the following censuses observe every possible event.
    owner->grant.source_complete=1;
    if(owner->slots==2&&!owner->members[1].state.pid)owner->grant.unknown_mask=2;
    owner->grant.cpu_start=private_cost.open_cpu;owner->pipe_cpu=0;owner->closed=0;
    int done;if(!known_exited(owner->ticket.pid,&done)||done)return engine_failure(owner,"settlement_unknown","CONT lacks original live unreaped leader");
    owner->grant.t0=engine_now(owner,CLOCK_MONOTONIC);
    if(kill(-owner->ticket.pgid,SIGCONT)!=0)return engine_failure(owner,"coverage_unknown","original CONT effect unresolved");
    owner->running=1;owner->stop_requested=0;owner->stop_attempts=0;owner->stop_result=owner->stop_errno=0;
    owner->stop_call_ns=owner->stop_terminal_ns=0;owner->stop_terminal=owner->stop_terminal_candidate=0;owner->grants++;
    const char *test=owner->context->outer->case_name;
    for(;;) {
        uint64_t now;if(!engine_clock(owner,CLOCK_MONOTONIC,&now))break;int stop=0;
        if(!engine_stage_begin(owner,ENGINE_STAGE_PIPE))break;
        int pipe_ok=engine_pipe_io(owner);if(!engine_stage_end(owner,ENGINE_STAGE_PIPE)||!pipe_ok)break;
        if(!strcmp(test,"cancel")&&!owner->case_fired){owner->case_fired=1;engine_cancel_signal=1;}
        if(!strcmp(test,"pause_resume")&&!owner->case_fired){owner->case_fired=1;owner->foreground=1;owner->hold_until=now+20000000;}
        int done=0;if(!engine_stage_begin(owner,ENGINE_STAGE_DIRECT))break;
        int direct_ok=engine_direct_exited(owner,&done);if(!engine_stage_end(owner,ENGINE_STAGE_DIRECT)||!direct_ok)break;
        if(engine_cancel_signal||owner->foreground||now>=owner->context->outer->deadline||done)stop=1;
        uint64_t target=owner->slots==2?25000:100000;
        if(!strcmp(test,"late_stop")&&!owner->case_fired&&!engine_cancel_signal&&!owner->foreground) {
            // A controlled negative keeps the original interval open; the
            // same hard Q gate must reject the actual delayed STOP. It does
            // not receive a larger quantum or an invented earlier endpoint.
            if(now-owner->grant.t0<1200000)continue;
            owner->case_fired=1;
        }
        if(now-owner->grant.t0>=target)stop=1;
        if(stop) {
            if(!engine_stage_begin(owner,ENGINE_STAGE_STOP))break;
            int stopped=engine_stop_group(owner);if(!engine_stage_end(owner,ENGINE_STAGE_STOP)||!stopped)break;
            if(!engine_stage_begin(owner,ENGINE_STAGE_ROSTER))break;
            int closed=0,roster_ok=engine_roster(owner,1,&closed);
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
            if(source_known_closure_over_cpu) {
                uint64_t until=clock_ns(CLOCK_THREAD_CPUTIME_ID)+1100000;
                while(clock_ns(CLOCK_THREAD_CPUTIME_ID)<until){}
            }
#endif
            if(!engine_stage_end(owner,ENGINE_STAGE_ROSTER)||!roster_ok)break;
            // Re-read clocks AFTER the native observation. The old loop's
            // pre-observation `now` could miss an overlong pending/closed span.
            // Never split/backdate W or pay/sleep inside a still-open grant.
            if(!engine_closure_bound(owner)||closed)break;
        }
        if(!engine_closure_bound(owner))break;
    }
    if(!owner->grant.roster_closed)return 0; // Retirement retains the original open ledger.
    if(owner->members[0].phase==ENGINE_EXITED) {
        // The physical roster is closed. Only this original owner's actual
        // wait4 may supply the terminal counter; RI/errno is never wait4.
        owner->accounting_pending=1;owner->running=0;
        if(!strcmp(owner->context->outer->case_name,"reap_delay")||!engine_observe_wait4(owner))return 0;
    }
    if(!engine_finish_grant(owner))return 0;
    if(engine_cancel_signal)return engine_failure(owner,"cancelled","private fixture cancellation requested");
    if(owner->failed)return 0;
    return owner->direct_reaped?1:engine_stopped_corroboration(owner);
}
static int engine_put(struct engine_prepared *p,const char *text,char **out) {
    size_t n=strlen(text);
    if(n>4096||n+1>sizeof(p->bytes)-p->used)return 0;
    *out=p->bytes+p->used;memcpy(*out,text,n+1);p->used+=n+1;return 1;
}
static int engine_expand(const struct private_engine_context *context,const char *text,const char *home,const char *scratch,char out[4097]) {
    // Both the B matcher and its retired fixture-path spelling are forbidden
    // inputs to every expander. Only the actual argv observed at GS1 binds V.
    if(strstr(text,"{git_signature_temp}")||strstr(text,"{signature}"))return 0;
    const struct private_fixture *f=&context->basis->fixture;char gpg[4097],record[4097],branch[4097];
    if(!private_token_text(context->basis->build.tools[1].ref.path,gpg,sizeof(gpg))
        ||!private_token_text(f->record,record,sizeof(record))||!private_token_text(f->policy_branch,branch,sizeof(branch)))return 0;
    const char *commit=!strcmp(context->outer->case_name,"corrupt_proof")?f->corrupt:f->valid;size_t used=0;
    while(*text) {
        const char *value=NULL;size_t take=1;
        if(*text=='{') {
            const char *end=strchr(text,'}');if(!end)return 0;take=(size_t)(end-text)+1;
#define EXPAND(name,content) if(take==sizeof(name)-1&&!memcmp(text,name,take))value=(content)
            EXPAND("{gpg}",gpg);else EXPAND("{commit}",commit);else EXPAND("{ancestor}",f->ancestor);
            else EXPAND("{record}",record);else EXPAND("{policy_branch}",branch);
            else EXPAND("{home}",home);else EXPAND("{scratch}",scratch);else return 0;
#undef EXPAND
        }
        if(value){size_t len=strlen(value);if(len>4096-used)return 0;memcpy(out+used,value,len);used+=len;}
        else {if(used>=4096)return 0;out[used++]=*text;}
        text+=take;
    }
    out[used]=0;return 1;
}
static int engine_prepare_argv(const struct private_engine_context *context,unsigned form,const char *home,const char *scratch,struct engine_prepared *p) {
    memset(p,0,sizeof(*p));for(unsigned i=0;i<2;i++)p->input[i]=p->output[i]=p->errors[i]=-1;
    if(form>=16||form==15)return 0; // verify_child is the implicit child, never a third direct process.
    const struct private_command_form *command=&private_forms[form];unsigned tool=!strcmp(command->tool,"gpg");
    if(!private_token_text(context->basis->build.tools[tool].ref.path,p->image,sizeof(p->image)))return 0;
    if(!private_token_text(context->basis->build.tools[1].ref.path,p->descendant,sizeof(p->descendant)))return 0;
    if(strlen(context->outer->repo.path)>=sizeof(p->cwd))return 0;strcpy(p->cwd,context->outer->repo.path);
    if(!engine_put(p,p->image,&p->argv[p->argc++]))return 0;
    const char *const *prefix=tool?private_gpg_prefix:private_git_prefix;unsigned count=tool?4:16;char expanded[4097];
    for(unsigned i=0;i<count;i++)if(p->argc>=64||!engine_expand(context,prefix[i],home,scratch,expanded)||!engine_put(p,expanded,&p->argv[p->argc++]))return 0;
    for(unsigned i=0;i<command->argc;i++)if(p->argc>=64||!engine_expand(context,command->argv[i],home,scratch,expanded)||!engine_put(p,expanded,&p->argv[p->argc++]))return 0;
    p->argv[p->argc]=NULL;
    for(unsigned i=0;i<11;i++) {
        char value[4097],entry[4097];
        if(!engine_expand(context,private_env_values[i],home,scratch,value))return 0;
        int n=snprintf(entry,sizeof(entry),"%s=%s",private_env_names[i],value);
        if(n<=0||(size_t)n>=sizeof(entry)||!engine_put(p,entry,&p->env[p->envc++]))return 0;
    }
    p->env[p->envc]=NULL;return 1;
}
static int engine_pipe_pair(int pair[2]) {
    if(pipe(pair)!=0)return 0;
    for(unsigned i=0;i<2;i++)if(fcntl(pair[i],F_SETFD,FD_CLOEXEC)<0)return 0;
    return 1;
}
static int engine_prepare_effect(struct engine_owner *owner,struct engine_prepared *p,int input) {
    if(posix_spawnattr_init(&p->attr)!=0)return engine_failure(owner,"coverage_unknown","private spawn attributes unavailable");p->attr_live=1;
    if(posix_spawnattr_setflags(&p->attr,POSIX_SPAWN_START_SUSPENDED|POSIX_SPAWN_SETPGROUP|POSIX_SPAWN_CLOEXEC_DEFAULT)!=0
        ||posix_spawnattr_setpgroup(&p->attr,0)!=0)return engine_failure(owner,"coverage_unknown","private suspended group attributes unavailable");
    if(posix_spawn_file_actions_init(&p->actions)!=0)return engine_failure(owner,"coverage_unknown","private file actions unavailable");p->actions_live=1;
    if(!engine_pipe_pair(p->output)||!engine_pipe_pair(p->errors)||(input&&!engine_pipe_pair(p->input)))return engine_failure(owner,"resource_limit","owned pipe preparation failed");
    if(posix_spawn_file_actions_addchdir(&p->actions,p->cwd)!=0
        ||posix_spawn_file_actions_adddup2(&p->actions,p->output[1],1)!=0||posix_spawn_file_actions_adddup2(&p->actions,p->errors[1],2)!=0)
        return engine_failure(owner,"coverage_unknown","private prepared cwd or output actions failed");
    int code=input?posix_spawn_file_actions_adddup2(&p->actions,p->input[0],0):posix_spawn_file_actions_addopen(&p->actions,0,"/dev/null",O_RDONLY,0);
    return code==0?1:engine_failure(owner,"coverage_unknown","private prepared stdin action failed");
}
static void engine_dispose_prepared(struct engine_prepared *p) {
    uint64_t at=private_cost.open_cpu;unsigned work=0;
    if(p->actions_live){p->actions_live=0;work++;if(posix_spawn_file_actions_destroy(&p->actions)!=0)private_fail("settlement_unknown","prepared file-action disposal unresolved");}
    if(p->attr_live){p->attr_live=0;work++;if(posix_spawnattr_destroy(&p->attr)!=0)private_fail("settlement_unknown","prepared attributes disposal unresolved");}
    for(unsigned i=0;i<2;i++) {
        int *fds[3]={&p->input[i],&p->output[i],&p->errors[i]};
        for(unsigned j=0;j<3;j++)if(*fds[j]>=0) {
            int fd=*fds[j];*fds[j]=-1;work++;
            if(close(fd)!=0)private_fail("settlement_unknown","prepared descriptor close unresolved");
        }
    }
    // At most eight adjacent owned disposals form one measured ordinary unit.
    // No no-op disposal mints another minimum rest interval.
    if(work)(void)private_end_unit(at);
}
static int engine_transfer_prepared(struct engine_owner *owner,struct engine_prepared *p) {
    owner->input=p->input[1];p->input[1]=-1;owner->output=p->output[0];p->output[0]=-1;owner->errors=p->errors[0];p->errors[0]=-1;
    int *kept[3]={&owner->input,&owner->output,&owner->errors};
    for(unsigned i=0;i<3;i++)if(*kept[i]>=0) {
        int flags=fcntl(*kept[i],F_GETFL);if(flags<0||fcntl(*kept[i],F_SETFL,flags|O_NONBLOCK)<0)
            return engine_failure(owner,"coverage_unknown","retained pipe nonblocking setup failed");
    }
    // No independent unit/rest occurs inside the measured native application
    // span. The retained suspended effect precedes all of these verdicts.
    for(unsigned i=0;i<2;i++) {
        int *fds[3]={&p->input[i],&p->output[i],&p->errors[i]};
        for(unsigned j=0;j<3;j++)if(*fds[j]>=0){int fd=*fds[j];*fds[j]=-1;if(close(fd)!=0)return engine_failure(owner,"source_changed","child-end descriptor close unresolved");}
    }
    if(p->actions_live){p->actions_live=0;if(posix_spawn_file_actions_destroy(&p->actions)!=0)return engine_failure(owner,"coverage_unknown","spawn file actions disposal failed");}
    if(p->attr_live){p->attr_live=0;if(posix_spawnattr_destroy(&p->attr)!=0)return engine_failure(owner,"coverage_unknown","spawn attributes disposal failed");}
    return 1;
}
static int engine_birth(struct engine_owner *owner,struct engine_prepared *p) {
    uint64_t inclusive=private_cost.open_cpu,wall=engine_now(owner,CLOCK_MONOTONIC);
    // Admission clocks precede the native effect. Reservations remain spent,
    // but an unknown beginning never causes an attempted creation.
    if(inclusive==UINT64_MAX||wall==UINT64_MAX||owner->failed) {
        engine_max(&owner->result->inclusive_max_ns,UINT64_MAX);
        engine_charge(owner,UINT64_MAX,UINT64_MAX,0);return 0;
    }
    if(engine_cancel_signal||wall>=owner->context->outer->deadline) {
        engine_failure(owner,engine_cancel_signal?"cancelled":"deadline","creation revoked before any native effect");
        uint64_t used=engine_interval(inclusive,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        engine_charge(owner,used,used,1);return 0;
    }
    if(owner->git_signature&&!private_git_sig_before_birth(owner->context,owner->git_signature)) {
        engine_failure(owner,private_error,private_detail);
        uint64_t used=engine_interval(inclusive,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        engine_charge(owner,used,used,1);return 0;
    }
    struct prepared_spawn_outcome raw;prepared_spawn(p->image,p->argv,p->env,&p->actions,&p->attr,1,&raw);
    owner->native=raw;
    if(raw.pid>0) {
        owner->ticket.pid=owner->ticket.pgid=raw.pid;owner->result->created++;
        owner->direct_custody=1;
        owner->result->group_gone=owner->result->retired_coverage=0;
    } else if(raw.attempted&&!raw.error) {
        owner->result->group_gone=owner->result->retired_coverage=0;
        engine_failure(owner,"settlement_unknown","native success reported without a usable owned PID");
    }
    int transferred=engine_transfer_prepared(owner,p);
    if(raw.pid>0) {
        struct image first;
        if(!control_observe(raw.pid,&first)||first.ppid!=getpid()||first.pgid!=raw.pid)
            engine_failure(owner,"coverage_unknown","suspended birth identity unavailable");
        else {
            owner->ticket.birth=first.birth;owner->members[0].state.pid=raw.pid;owner->members[0].state.birth=first.birth;
            uint64_t until=engine_now(owner,CLOCK_MONOTONIC)+25000000;int ack=0;
            do {
                if(!engine_sample_member(owner,0,raw.pid,&ack))break;
                if(ack)break;
                usleep(100);
            } while(engine_now(owner,CLOCK_MONOTONIC)<until);
            if(!ack||owner->members[0].user)engine_failure(owner,"coverage_unknown","helper was not born fully stopped before user work");
            owner->ticket.start=owner->members[0].state.start_absolute;owner->closed=ack;
        }
    }
    uint64_t end=engine_now(owner,CLOCK_THREAD_CPUTIME_ID),ended=engine_now(owner,CLOCK_MONOTONIC);
    uint64_t total=engine_interval(inclusive,end),whole_wall=engine_interval(wall,ended);
    uint64_t app=total!=UINT64_MAX&&raw.cpu!=UINT64_MAX&&total>=raw.cpu?total-raw.cpu:UINT64_MAX;
    engine_max(&owner->result->native_cpu_max_ns,raw.cpu);engine_max(&owner->result->native_wall_max_ns,raw.wall);
    engine_max(&owner->result->inclusive_max_ns,total);engine_max(&owner->result->ordinary_max_ns,app);
    if(!strcmp(owner->context->outer->case_name,"clock_failure")) {
        engine_max(&owner->result->inclusive_max_ns,UINT64_MAX);total=UINT64_MAX;
        engine_failure(owner,"clock_unknown","fixture injected unknown ending clock after retaining actual PID");
    }
    int measured=engine_charge(owner,total,total,0);
    total=private_cost.last_unit;
    app=total!=UINT64_MAX&&raw.cpu!=UINT64_MAX&&total>=raw.cpu?total-raw.cpu:UINT64_MAX;
    engine_max(&owner->result->inclusive_max_ns,total);engine_max(&owner->result->ordinary_max_ns,app);
    if(!raw.attempted||!raw.cpu_known||!raw.wall_known||!raw.pid||raw.pid<0||raw.error||!transferred)
        return engine_failure(owner,"coverage_unknown","prepared native operation or retained effect refused");
    if(raw.cpu>2000000||raw.wall>25000000||total>3000000||app>1000000)
        return engine_failure(owner,"resource_limit","unchanged native2ms wall25ms application1ms inclusive3ms gate");
    if(whole_wall==UINT64_MAX)return engine_failure(owner,"clock_unknown","inclusive birth wall clock unknown");
    if(!measured||owner->failed||private_failed)return 0;
    owner->result->members_max=owner->result->members_max<1?1:owner->result->members_max;
    if(!strcmp(owner->context->outer->case_name,"stale_incarnation")) {
        struct engine_ticket stale=owner->ticket;stale.birth++;
        if(engine_ticket_equal(&owner->ticket,&stale))return engine_failure(owner,"coverage_unknown","stale ticket accepted");
        return engine_failure(owner,"coverage_unknown","fixture stale original incarnation refused without signaling it");
    }
    return engine_stopped_corroboration(owner);
}
static uint64_t engine_timeval(const struct timeval *value) {
    if(value->tv_sec<0||value->tv_usec<0||value->tv_usec>=1000000)return UINT64_MAX;
    __uint128_t ns=(__uint128_t)value->tv_sec*1000000000+(uint64_t)value->tv_usec*1000;
    return ns>UINT64_MAX?UINT64_MAX:(uint64_t)ns;
}
static int engine_observe_wait4(struct engine_owner *owner) {
    if(owner->direct_reaped)return owner->wait_observed;
    struct rusage usage;int status=0;errno=0;
    pid_t joined=wait4(owner->ticket.pid,&status,WNOHANG,&usage);
    if(joined==0||(joined<0&&errno==EINTR))return 0;
    if(joined!=owner->ticket.pid) {
        owner->direct_custody=0;
        return engine_failure(owner,"settlement_unknown","original actual wait4 became unavailable");
    }
    owner->retained_wait=usage;owner->retained_status=status;
    owner->direct_reaped=owner->wait_observed=1;owner->direct_custody=0;retained_group=0;
    owner->result->reaped++;owner->wait_user=engine_timeval(&usage.ru_utime);owner->wait_system=engine_timeval(&usage.ru_stime);
    owner->wait_rss=usage.ru_maxrss<0?UINT64_MAX:(uint64_t)usage.ru_maxrss;
    owner->wait_read_ns=engine_now(owner,CLOCK_MONOTONIC);
    if(owner->wait4_reads==UINT64_MAX)return engine_failure(owner,"clock_unknown","actual wait4 rounding count overflow");
    owner->wait4_reads++;
    if(owner->stop_pending&&owner->stop_terminal_candidate){owner->stop_terminal=1;owner->stop_terminal_ns=owner->wait_read_ns;}
    return owner->wait_read_ns!=UINT64_MAX;
}
enum engine_terminal_ri_result { ENGINE_TERMINAL_RI_KNOWN,ENGINE_TERMINAL_RI_UNAVAILABLE,ENGINE_TERMINAL_RI_INVALID };
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static unsigned source_known_initial_ri_unavailable,source_known_initial_payment_interrupt,source_known_initial_ri_calls;
#endif
static int engine_initial_ri_read(pid_t pid,struct rusage_info_v1 *usage) {
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    source_known_initial_ri_calls++;
#endif
    return proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)usage);
}
static unsigned engine_initial_physical_basis(const struct engine_initial_physical_observation *p,int original_eligible) {
    if(!original_eligible||!p->own_wnowait_known||!p->own_wnowait_terminal
        ||!p->group_original_only||p->group_result<0||p->child_result!=0)return 0;
    if(p->bsd_known&&p->predicate_mask==255u)return 1;
    // For this SOURCE-ADMITTED never-continued one-slot startup, an original
    // unreaped positive PID pin + its OWN fresh WNOWAIT terminal result, old
    // token ESRCH, retained birth/start and exact census are independent proof.
    // BSD ESRCH supplies NO exit or absence inference. Any returned BSD row
    // must instead satisfy every original identity/zombie predicate above.
    if(!p->bsd_known&&p->bsd_result==0&&p->bsd_error==ESRCH&&p->predicate_mask==1u)return 2;
    return 0;
}
static enum engine_terminal_ri_result engine_initial_terminal_ri(struct engine_owner *owner) {
    struct engine_member *leader=&owner->members[0];struct proc_bsdinfo b;
    struct engine_initial_physical_observation *p=&owner->initial_physical;p->stage=2;
    char path[PROC_PIDPATHINFO_MAXSIZE];errno=0;
    p->audit_result=proc_pidpath_audittoken(&leader->token,path,sizeof(path));p->audit_error=p->audit_result<=0?errno:0;
    p->audit_path_expected=p->audit_result>0&&(size_t)p->audit_result<sizeof(path)
        &&strnlen(path,sizeof(path))<sizeof(path)&&!strcmp(path,owner->image);
    errno=0;p->bsd_result=proc_pidinfo(owner->ticket.pid,PROC_PIDTBSDINFO,0,&b,sizeof(b));p->bsd_error=p->bsd_result==(int)sizeof(b)?0:errno;
    if(p->audit_result<=0&&p->audit_error==ESRCH)p->predicate_mask|=1u;
    if(p->bsd_result==(int)sizeof(b)) {
        p->bsd_known=1;p->pid=b.pbi_pid;p->ppid=b.pbi_ppid;p->pgid=b.pbi_pgid;p->status=b.pbi_status;p->uid=b.pbi_uid;
        p->birth=(uint64_t)b.pbi_start_tvsec*1000000+b.pbi_start_tvusec;
        p->predicate_mask|=2u;
        if(p->pid==(uint32_t)owner->ticket.pid)p->predicate_mask|=4u;
        if(p->ppid==(uint32_t)getpid())p->predicate_mask|=8u;
        if(p->pgid==(uint32_t)owner->ticket.pgid)p->predicate_mask|=16u;
        if(p->status==SZOMB)p->predicate_mask|=32u;
        if(p->uid==geteuid())p->predicate_mask|=64u;
        if(p->birth==owner->ticket.birth)p->predicate_mask|=128u;
    }
    p->physical_basis=engine_initial_physical_basis(p,owner->post_exit_initial);
    if(!p->physical_basis)return ENGINE_TERMINAL_RI_INVALID;
    p->stage=3;
    owner->post_exit_ri_attempted=1;
    struct rusage_info_v1 usage;int read=engine_initial_ri_read(owner->ticket.pid,&usage);
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_initial_ri_unavailable){read=-1;errno=ENOTSUP;}
#endif
    if(read!=0)return ENGINE_TERMINAL_RI_UNAVAILABLE;
    if(usage.ri_proc_start_abstime!=owner->ticket.start||usage.ri_user_time<leader->user||usage.ri_system_time<leader->system)
        return ENGINE_TERMINAL_RI_INVALID;
    if(!usage.ri_proc_exit_abstime)return ENGINE_TERMINAL_RI_UNAVAILABLE;
    if(!engine_usage(owner,0,&usage))return ENGINE_TERMINAL_RI_INVALID;
    leader->phase=ENGINE_EXITED;leader->exit_proved_ns=engine_now(owner,CLOCK_MONOTONIC);
    return leader->exit_proved_ns!=UINT64_MAX&&leader->exit_proved_ns?ENGINE_TERMINAL_RI_KNOWN:ENGINE_TERMINAL_RI_INVALID;
}
static int engine_initial_predicate_record(struct engine_owner *owner) {
    const struct engine_initial_physical_observation *p=&owner->initial_physical;char line[1536];
    int n=snprintf(line,sizeof(line),"{\"kind\":\"initial_terminal_predicates\",\"stage\":%u,\"leader\":%d,\"expected_parent\":%d,\"expected_pgid\":%d,\"expected_birth\":%llu,\"expected_start\":%llu,\"expected_uid\":%u,\"expected_status\":%u,\"token_version\":%u,\"returned_BSD_required_mask\":255,\"predicate_mask\":%u,\"group_result\":%d,\"group_errno\":%d,\"child_result\":%d,\"child_errno\":%d,\"audit_result\":%d,\"audit_errno\":%d,\"audit_path_expected\":%u,\"bsd_result\":%d,\"bsd_errno\":%d,\"bsd_known\":%u,\"bsd_pid\":%u,\"bsd_parent\":%u,\"bsd_pgid\":%u,\"bsd_status\":%u,\"bsd_uid\":%u,\"bsd_birth\":%llu,\"own_WNOWAIT_known\":%u,\"own_WNOWAIT_terminal\":%u,\"physical_basis\":%u,\"source_valid\":%u,\"ri_attempted\":%u,\"ri_unavailable\":%u,\"attempt_retained\":%u,\"attempt_paid\":%u,\"debt_charged_ns\":%llu,\"debt_paid_ns\":%llu,\"runtime_approved\":false}\n",
        p->stage,owner->ticket.pid,getpid(),owner->ticket.pgid,(unsigned long long)owner->ticket.birth,(unsigned long long)owner->ticket.start,
        (unsigned)geteuid(),(unsigned)SZOMB,owner->members[0].token.val[7],p->predicate_mask,p->group_result,p->group_error,p->child_result,p->child_error,
        p->audit_result,p->audit_error,p->audit_path_expected,p->bsd_result,p->bsd_error,p->bsd_known,p->pid,p->ppid,p->pgid,p->status,p->uid,
        (unsigned long long)p->birth,p->own_wnowait_known,p->own_wnowait_terminal,p->physical_basis,owner->post_exit_initial_source_valid,owner->post_exit_ri_attempted,owner->post_exit_ri_unavailable,
        owner->post_exit_attempt_retained,owner->post_exit_attempt_paid,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    int recorded=engine_write_record(owner,line,n);
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    int paid=engine_charge(owner,used,used,1);engine_sum(&owner->result->post_cpu_ns,private_cost.last_unit);
    return recorded&&paid;
}
static int engine_initial_attempt_paid(struct engine_owner *owner) {
    uint64_t now;
    int paid=owner->post_exit_initial&&owner->post_exit_initial_source_valid&&owner->post_exit_ri_attempted&&owner->post_exit_attempt_retained
        &&!private_has_retained_cost()&&!private_cost.retained_overflow&&private_cost.debt_paid==private_cost.debt
        &&private_cost.open_cpu!=UINT64_MAX&&known_clock_ns(CLOCK_MONOTONIC,&now)&&now>=private_cost.rest_until;
    owner->post_exit_attempt_paid=(unsigned)paid;return paid;
}
static int engine_independent_post_exit(struct engine_owner *owner) {
    if(owner->post_exit_observed&&owner->post_exit_initial) {
        if(!engine_initial_attempt_paid(owner))return 0;
        return !owner->post_exit_ri_unavailable&&owner->conservation_known;
    }
    struct engine_member *leader=&owner->members[0];
    int initial=owner->source_admitted&&owner->startup_known&&owner->startup_charged&&owner->conservation_known
        &&owner->slots==1&&owner->closed&&owner->grant.roster_closed&&!owner->grants&&!owner->grant.t0
        &&!owner->child_created&&!owner->members[1].state.pid&&!owner->grant.unknown_mask
        &&!owner->coverage_lost&&!owner->pending_identity_lost&&!owner->stop_pending
        &&owner->observed_user[0]==0&&leader->user==0&&owner->retired_count==1
        &&leader->token_known&&leader->state.pid==owner->ticket.pid&&leader->state.birth==owner->ticket.birth
        &&leader->state.start_absolute&&leader->state.start_absolute==owner->ticket.start
        &&owner->context->basis->source_coverage==127&&owner->context->basis->command_coverage==65535;
    int original=0;
    for(unsigned i=0;i<owner->retired_count;i++)if(owner->retired[i].current&&owner->retired[i].pid==owner->ticket.pid
        &&owner->retired[i].birth==owner->ticket.birth&&owner->retired[i].start==owner->ticket.start
        &&!memcmp(&owner->retired[i].token,&leader->token,sizeof(leader->token)))original=1;
    initial=initial&&original;
    if(owner->startup_known&&owner->startup_charged&&!owner->grants&&!owner->grant.t0&&!initial)
        return engine_failure(owner,"coverage_unknown","charged initial terminal source lacks original validated eligibility");
    if(owner->post_exit_observed||owner->direct_reaped||(!owner->grant_accounted&&!initial)
        ||!owner->grant.roster_closed||owner->running||owner->accounting_pending)return 1;
    uint32_t numer=initial?owner->startup_timebase_numer:owner->grant.timebase_numer;
    uint32_t denom=initial?owner->startup_timebase_denom:owner->grant.timebase_denom;
    mach_timebase_info_data_t scale;
    uint64_t paid_wall;
    if(private_has_retained_cost()||private_cost.retained_overflow||private_cost.debt_paid!=private_cost.debt
        ||private_cost.open_cpu==UINT64_MAX||!known_clock_ns(CLOCK_MONOTONIC,&paid_wall)||paid_wall<private_cost.rest_until
        ||mach_timebase_info(&scale)!=KERN_SUCCESS||!numer||!denom||numer!=scale.numer||denom!=scale.denom)
        return engine_failure(owner,"coverage_unknown","independent post-exit RI requires original timebase and fully paid prior debt");
    int done=0;
    if(!known_exited(owner->ticket.pid,&done)||!done)
        return engine_failure(owner,"settlement_unknown","independent post-exit RI needs the original unreaped WNOWAIT identity");
    owner->post_exit_observed=1;owner->post_exit_initial=(unsigned)initial;int initial_roster_valid=1;
    if(initial) {
        struct engine_initial_physical_observation *p=&owner->initial_physical;p->stage=1;
        p->own_wnowait_known=p->own_wnowait_terminal=1; // This caller's immediately preceding successful original WNOWAIT.
        int group[3]={0},children[2]={0};
        errno=0;int n=proc_listpgrppids(owner->ticket.pgid,group,sizeof(group));p->group_result=n;p->group_error=n<0?errno:0;
        errno=0;int c=proc_listchildpids(owner->ticket.pid,children,sizeof(children));p->child_result=c;p->child_error=c<0?errno:0;
        p->group_original_only=n>=0&&n<=1&&(n==0||group[0]==owner->ticket.pid);
        if(n<0||c<0||n>1||c||((n==1)&&group[0]!=owner->ticket.pid)) {
            if(n>1||c>0||((n==1)&&group[0]!=owner->ticket.pid))owner->coverage_lost=1;
            initial_roster_valid=0;(void)engine_failure(owner,"coverage_unknown","initial terminal RI lacks fresh original closed child/group roster");
        }
    }
    uint64_t before,after,old_user,new_user,old_system,new_system,delta=UINT64_MAX;
    memcpy(owner->post_exit_before,owner->observed_raw,sizeof(owner->post_exit_before));
    int known=engine_safe_sum(owner->observed_raw[0],owner->observed_raw[1]>owner->observed_raw[2]?owner->observed_raw[1]:owner->observed_raw[2],&before)
        &&engine_safe_sum(owner->observed_user[0],owner->observed_user[1]>owner->observed_user[2]?owner->observed_user[1]:owner->observed_user[2],&old_user)
        &&engine_safe_sum(owner->observed_system[0],owner->observed_system[1]>owner->observed_system[2]?owner->observed_system[1]:owner->observed_system[2],&old_system);
    if(initial) {
        enum engine_terminal_ri_result result=initial_roster_valid?engine_initial_terminal_ri(owner):ENGINE_TERMINAL_RI_INVALID;
        owner->post_exit_initial_source_valid=result!=ENGINE_TERMINAL_RI_INVALID;
        owner->post_exit_ri_unavailable=result==ENGINE_TERMINAL_RI_UNAVAILABLE;
        if(result!=ENGINE_TERMINAL_RI_KNOWN)known=0;
    } else if(!engine_member_exit(owner,0))known=0;
    if(owner->members[1].state.pid&&!engine_member_exit(owner,1))known=0;
    uint64_t raw[3]={owner->members[0].state.raw_rusage_cpu,owner->members[1].state.raw_rusage_cpu,owner->members[0].state.raw_waited};
    memcpy(owner->post_exit_after,raw,sizeof(raw));
    if(!engine_safe_sum(raw[0],raw[1]>raw[2]?raw[1]:raw[2],&after)
        ||!engine_safe_sum(owner->members[0].user,owner->members[1].user>owner->members[0].child_user?owner->members[1].user:owner->members[0].child_user,&new_user)
        ||!engine_safe_sum(owner->members[0].system,owner->members[1].system>owner->members[0].child_system?owner->members[1].system:owner->members[0].child_system,&new_system)
        ||!known||after<before||new_system<old_system
        ||!engine_scaled_ticks(new_system-old_system,numer,denom,1,&delta))known=0;
    int justified=known&&new_user==old_user;
    if(initial&&((!known&&!owner->post_exit_ri_unavailable)||(known&&!justified)))owner->post_exit_initial_source_valid=0;
    if(!justified)(void)engine_failure(owner,"coverage_unknown","post-exit raw RI lacks stable user or original monotonic source proof");
    owner->post_exit_delta=delta;
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID)),cost;
    if(!engine_safe_sum(used,delta,&cost))cost=UINT64_MAX;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(initial&&source_known_initial_payment_interrupt){source_known_initial_payment_interrupt=0;source_known_payment_interrupts=1;}
#endif
    uint64_t booked=private_cost.closed_units;int paid=engine_charge(owner,used,cost,1);
    if(initial) {
        owner->post_exit_attempt_retained=private_cost.closed_units==booked+1;
        (void)engine_initial_attempt_paid(owner);
    }
    engine_sum(&owner->result->post_cpu_ns,used);engine_sum(&owner->result->late_kernel_ns,delta);
    if(justified&&cost!=UINT64_MAX&&private_cost.closed_units==booked+1)(void)engine_credit_member(owner,delta,1);
    else owner->conservation_known=0;
    engine_observed_counters(owner);
    int recorded=1;
    if(initial&&!owner->post_exit_initial_source_valid)recorded=engine_initial_predicate_record(owner);
    return paid&&justified&&recorded;
}
static int engine_independent_credit(struct engine_owner *owner,uint64_t *credit) {
    return owner->startup_known&&owner->startup_cpu!=UINT64_MAX&&owner->charged_member_cpu!=UINT64_MAX
        &&owner->justified_late_cpu!=UINT64_MAX&&owner->justified_post_cpu!=UINT64_MAX
        &&engine_safe_sum(owner->startup_cpu,owner->charged_member_cpu,credit)
        &&engine_safe_sum(*credit,owner->justified_late_cpu,credit)
        &&engine_safe_sum(*credit,owner->justified_post_cpu,credit);
}
static int engine_residual_obligation(struct engine_owner *owner,uint64_t pending_independent) {
    uint64_t raw,upper,credit=0;
    if(!owner->direct_reaped||!owner->wait_observed||owner->wait_user==UINT64_MAX||owner->wait_system==UINT64_MAX
        ||!engine_safe_sum(owner->wait_user,owner->wait_system,&raw)||!engine_safe_sum(raw,2000,&upper)) {
        owner->residual_required_known=0;return 0;
    }
    int known=engine_independent_credit(owner,&credit)&&pending_independent!=UINT64_MAX
        &&engine_safe_sum(credit,pending_independent,&credit);
    owner->raw_wait4_cpu=raw;owner->wait4_upper=upper;owner->residual_required_known=(unsigned)known;
    owner->independent_cpu_credit=known?credit:0;
    // Unknown prefix credit is never subtracted as a known zero or used for
    // certification. Cost-only recovery conservatively bills the FULL known
    // own wait4 upper total in that case; conservation still refuses unknown.
    owner->residual_raw=known&&raw>credit?raw-credit:known?0:raw;
    owner->residual_required_cost=known&&upper>credit?upper-credit:known?0:upper;
    return 1;
}
static int engine_residual_cost_covers(struct engine_owner *owner) {
    uint64_t now;
    int paid=owner->residual_required_known&&owner->residual_retained_cost>=owner->residual_required_cost
        &&!private_has_retained_cost()&&!private_cost.retained_overflow&&private_cost.debt_paid==private_cost.debt
        &&private_cost.open_cpu!=UINT64_MAX&&known_clock_ns(CLOCK_MONOTONIC,&now)&&now>=private_cost.rest_until;
    owner->residual_cost_paid=(unsigned)paid;return paid;
}
static void engine_residual_payment_retained(struct engine_owner *owner,uint64_t amount,uint64_t before,uint64_t whole_cost) {
    if(amount&&amount!=UINT64_MAX&&whole_cost!=UINT64_MAX&&whole_cost>=amount
        &&owner->last_charged_cost!=UINT64_MAX&&owner->last_charged_cost>=amount
        &&private_cost.closed_units==before+1&&private_cost.closed_units>owner->residual_last_debit_unit) {
        if(owner->residual_debit_entries==UINT64_MAX||!engine_safe_sum(owner->residual_retained_cost,amount,&owner->residual_retained_cost)) {
            owner->conservation_known=0;owner->residual_cost_paid=0;(void)engine_failure(owner,"clock_unknown","retained residual cost ledger overflow");return;
        }
        owner->residual_debit_entries++;owner->residual_last_debit_unit=private_cost.closed_units;
    }
    (void)engine_residual_cost_covers(owner);
}
static unsigned engine_reaped_for_teardown(const struct engine_owner *owner,unsigned *direct,unsigned *child) {
    *direct=owner->direct_reaped&&owner->wait_observed&&owner->wait4_reads==1;
    const struct engine_child_reap_proof *p=&owner->child_reap_proof;const struct engine_member *m=&owner->members[1];
    *child=*direct&&p->valid&&owner->command==12&&owner->verified_descendants==1&&owner->child_created
        &&!owner->coverage_lost&&!owner->pending_identity_lost&&!owner->rollup_source_failed
        &&owner->context->basis->source_coverage==127&&owner->context->basis->command_coverage==65535
        &&m->phase==ENGINE_EXITED&&m->exit_abstime&&m->token_known&&m->state.pid==p->pid&&m->state.birth==p->birth
        &&m->state.start_absolute==p->start&&m->token.val[7]==p->version
        &&owner->git_signature&&owner->git_signature->bound&&owner->git_signature->pid==p->pid&&owner->git_signature->pidversion==p->version
        &&p->exit_ns&&p->zero_ns&&p->ack_ns&&p->read_ns>=p->exit_ns&&p->read_ns>=p->zero_ns&&p->read_ns>=p->ack_ns
        &&owner->members[0].state.raw_waited>=p->rollup_ticks;
    // Accounting rejection alone does not erase independently proven physical
    // reaps. Neither expected slots nor a wait4 residual contributes a count.
    return *direct+*child;
}
static int engine_wait4_late(struct engine_owner *owner,uint64_t pending_independent,uint64_t *late) {
    uint64_t cpu,user,system,user_upper,system_upper,tick;
    mach_timebase_info_data_t scale;
    if(!engine_residual_obligation(owner,pending_independent))return 0;
    int matched=engine_actual(owner,&cpu,&user,&system)&&mach_timebase_info(&scale)==KERN_SUCCESS
        &&engine_scaled_ticks(1,scale.numer,scale.denom,1,&tick)
        &&engine_safe_sum(owner->wait_user,1000,&user_upper)&&engine_safe_sum(user_upper,tick,&user_upper)
        &&engine_safe_sum(owner->wait_system,1000,&system_upper)&&engine_safe_sum(system_upper,tick,&system_upper)
        &&owner->wait_user<=user&&user_upper>=user&&system_upper>=system;
    // Each actual timeval field is rounded UP by1us; one explicit Mach-tick
    // conversion allowance covers its RI comparison. User growth is refused.
    uint64_t cost=owner->residual_required_cost;
    if(matched) {
        uint64_t prior=system_upper-system;
        if(owner->grant_accounted&&owner->grant.wait4_subtree_ns)prior=0;
        if(prior>cost)cost=prior;
    } else (void)engine_failure(owner,"coverage_unknown","final raw user/system cutoffs are not reconciled; full known wait4 residual still billed");
    if(cost>owner->residual_payment_target)owner->residual_payment_target=cost;
    *late=owner->residual_payment_target>owner->residual_retained_cost?owner->residual_payment_target-owner->residual_retained_cost:0;
    return 1;
}
static int engine_conservation_failure(struct engine_owner *owner,const char *code,const char *detail) {
    (void)engine_revoke_provisional(owner);
    return engine_failure(owner,code,detail);
}
static int engine_conservation(struct engine_owner *owner) {
    if(owner->conservation_checked)return owner->conservation_ok;
    owner->conservation_checked=1;owner->conservation_ok=0;
    if(owner->result->terminal_conservation_checked==UINT_MAX)
        return engine_conservation_failure(owner,"clock_unknown","terminal conservation result count overflow");
    owner->result->terminal_conservation_checked++;
    unsigned reaped=engine_reaped_for_teardown(owner,&owner->teardown_direct_reaps,&owner->teardown_child_reaps);
    owner->exit_teardown_limit_ns=ENGINE_EXIT_TEARDOWN_CEILING_NS;
    if(reaped>2||!owner->exit_teardown_limit_ns||reaped>UINT64_MAX/owner->exit_teardown_limit_ns)
        return engine_conservation_failure(owner,"clock_unknown","actual reaped exit-teardown ceiling overflow");
    owner->exit_teardown_allowance_ns=reaped*owner->exit_teardown_limit_ns;
    mach_timebase_info_data_t scale;uint64_t tick_allowance,wait_allowance,credit,raw;
    if(!owner->startup_known||!owner->conservation_known||!owner->direct_reaped||!owner->wait_observed||owner->rollup_source_failed||owner->coverage_lost
        ||owner->wait_user==UINT64_MAX||owner->wait_system==UINT64_MAX
        ||mach_timebase_info(&scale)!=KERN_SUCCESS||!scale.numer||!scale.denom
        ||(owner->grant.timebase_numer&&(scale.numer!=owner->grant.timebase_numer||scale.denom!=owner->grant.timebase_denom))
        ||(owner->startup_timebase_numer&&(scale.numer!=owner->startup_timebase_numer||scale.denom!=owner->startup_timebase_denom))
        ||!engine_scaled_ticks(owner->counter_reads,scale.numer,scale.denom,1,&tick_allowance)
        ||owner->wait4_reads>UINT64_MAX/2000)
        return engine_conservation_failure(owner,"coverage_unknown","terminal conservation has missing source counters or rounding evidence");
    wait_allowance=owner->wait4_reads*2000;
    if(!engine_safe_sum(tick_allowance,wait_allowance,&owner->rounding_allowance)
        ||!engine_safe_sum(owner->wait_user,owner->wait_system,&raw)||!engine_safe_sum(raw,2000,&owner->wait4_upper)
        ||!engine_safe_sum(owner->startup_cpu,owner->charged_member_cpu,&credit)
        ||!engine_safe_sum(credit,owner->justified_late_cpu,&credit)
        ||!engine_safe_sum(credit,owner->justified_post_cpu,&credit)
        ||!engine_safe_sum(credit,owner->rounding_allowance,&owner->accounted_upper))
        return engine_conservation_failure(owner,"clock_unknown","terminal conservation checked sum overflow");
    owner->raw_wait4_cpu=raw;owner->independent_cpu_credit=credit;owner->residual_required_known=1;
    owner->residual_raw=raw>credit?raw-credit:0;
    owner->residual_required_cost=owner->wait4_upper>credit?owner->wait4_upper-credit:0;
    if(!engine_safe_sum(owner->accounted_upper,owner->exit_teardown_allowance_ns,&owner->conservation_upper))
        return engine_conservation_failure(owner,"clock_unknown","fixed exit-teardown and independent ceiling sum overflow");
    // Controller/formatting CPU and normalized 1ms floors are NOT helper CPU
    // credit. Only the independently retained charged helper components above
    // may explain the actual whole-subtree wait4 upper bound.
    // T is a SEPARATE fixed per-actual-reaped-process ceiling, never helper
    // CPU credit, rounding, a native class or a wait4-derived allowance.
    owner->conservation_excess=owner->wait4_upper>owner->conservation_upper?owner->wait4_upper-owner->conservation_upper:0;
    if(!engine_residual_cost_covers(owner))
        return engine_conservation_failure(owner,"resource_limit","terminal wait4 residual cost was not fully retained and paid");
    if(owner->conservation_excess)return engine_conservation_failure(owner,"resource_limit","actual terminal CPU exceeds independently charged helper conservation bound");
    if(owner->result->terminal_conservation_passed==UINT_MAX)return engine_conservation_failure(owner,"clock_unknown","terminal conservation pass count overflow");
    owner->result->terminal_conservation_passed++;owner->conservation_ok=1;owner->provisional_finalized=1;return 1;
}
static int engine_physical_exit(const struct engine_incarnation *member) {
    if(!member->pid||!engine_token_gone(&member->token))return 0;
    struct proc_bsdinfo b;int size=sizeof(b);errno=0;
    int result=proc_pidinfo(member->pid,PROC_PIDTBSDINFO,0,&b,size);
    if(result==size) {
        uint64_t birth=(uint64_t)b.pbi_start_tvsec*1000000+b.pbi_start_tvusec;
        if(birth!=member->birth)return 1;
        if(b.pbi_status!=SZOMB)return 0;
    } else if(errno!=ESRCH)return 0;
    struct rusage_info_v1 usage;errno=0;
    if(proc_pid_rusage(member->pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)==0)
        return member->start&&usage.ri_proc_start_abstime==member->start&&usage.ri_proc_exit_abstime!=0;
    return result!=size&&errno==ESRCH;
}
static int engine_original_group_gone(struct engine_owner *owner) {
    if(owner->coverage_lost)return 0;
    // A discovered child lacking a retained identity never becomes absent
    // merely because a later group census is empty.
    if(owner->members[1].state.pid&&(!owner->members[1].token_known||!owner->members[1].state.start_absolute))return 0;
    for(unsigned i=0;i<owner->retired_count;i++)if(owner->retired[i].current&&!engine_physical_exit(&owner->retired[i]))return 0;
    int pids[3]={0};int count=proc_listpgrppids(owner->ticket.pgid,pids,sizeof(pids));
    if(count<0||count>2)return 0;
    // A nonempty numeric census after reap is not permission to infer either
    // original settlement or a replacement generation. Never signal it again.
    return count==0;
}
static void engine_quarantine_provisional(struct engine_owner *owner) {
    // A deadline/accounting refusal revokes an unfinalized measured rebate,
    // but cannot erase a separately retained physical child-reap proof.
    // Actual identity/roster loss retains its explicit source/coverage flags.
    if(owner->provisional_count&&!owner->provisional_finalized)(void)engine_revoke_provisional(owner);
}
static int engine_quarantine(struct engine_owner *owner,const char *detail) {
    owner->settlement_quarantined=1;owner->retiring=1;private_cost.custody_hold=1;
    if(!owner->controller_parent)owner->controller_parent=private_cost.original_parent?private_cost.original_parent:getppid();
    (void)engine_failure(owner,"settlement_unknown",detail);
    owner->result->phase="quarantined";owner->result->outcome="quarantined";
    owner->result->group_gone=owner->result->retired_coverage=0;
    engine_current=owner;dispose_private_engine=engine_emergency_dispose;
    // No endpoint/absence is minted for unresolved original work. Close the
    // controller's measured unit, with unknown helper/late cost kept unknown.
    uint64_t now=UINT64_MAX,cpu=UINT64_MAX;
    (void)known_clock_ns(CLOCK_MONOTONIC,&now);(void)known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu);
    if(owner->grant.t0&&!owner->grant.roster_closed) {
        owner->grant.cpu_close=cpu;owner->grant.actual_cpu=UINT64_MAX;
        (void)engine_window_cost(owner,now,0);
    }
    // Unknown terminal provenance cannot keep a provisional measured rebate
    // while this same owner holds custody. Retain/pay arrears before the raw
    // quarantine disposition; failure remains owned by the shared barrier.
    engine_quarantine_provisional(owner);
    engine_max(&owner->result->late_kernel_ns,UINT64_MAX);
    // The attempt flag precedes every evidence write: unknown writes are not
    // replayed at atexit. A successful persisted receipt links these unresolved
    // identities to run/nonce/input. Decision66074 keeps THIS controller alive
    // with its actual FDs/unreaped handle; this record never replaces custody.
    if(!owner->quarantine_recorded) {
        owner->quarantine_recorded=1;int durable=owner->ledger>=0;
        if(owner->ledger>=0&&!engine_record(owner,"quarantined"))durable=0;
        for(unsigned i=0;i<owner->retired_count;i++) {
            const struct engine_incarnation *m=&owner->retired[i];char line[768];
            int n=snprintf(line,sizeof(line),"{\"phase\":\"quarantine_identity\",\"input_sha256\":\"%s\",\"run_id\":\"%s\",\"nonce\":\"%s\",\"pid\":%d,\"pgid\":%d,\"birth\":%llu,\"start\":%llu,\"current\":%u,\"token\":[%u,%u,%u,%u,%u,%u,%u,%u],\"direct_reaped\":%u,\"unknown_child_pid\":%d,\"unknown_child_token\":%u,\"kill_attempted\":%u,\"kill_result\":%d,\"conditional_source_only\":true}\n",
                owner->context->input_sha256,owner->context->outer->run,owner->context->outer->nonce,m->pid,owner->ticket.pgid,
                (unsigned long long)m->birth,(unsigned long long)m->start,m->current,m->token.val[0],m->token.val[1],m->token.val[2],m->token.val[3],
                m->token.val[4],m->token.val[5],m->token.val[6],m->token.val[7],owner->direct_reaped,
                owner->members[1].state.pid,owner->members[1].token_known,m->kill_attempted,m->kill_result);
            if(n<=0||(size_t)n>=sizeof(line)||owner->record_count>=ENGINE_RECORD_MAX
                ||owner->ledger_bytes>ENGINE_CAPTURE_MAX||(size_t)n>ENGINE_CAPTURE_MAX-owner->ledger_bytes){durable=0;break;}
            owner->record_count++;owner->ledger_bytes+=(size_t)n;
            if(owner->ledger>=0) {if(write(owner->ledger,line,(size_t)n)!=n){durable=0;break;}}
            else {durable=0;if(write(STDERR_FILENO,line,(size_t)n)!=n)break;}
        }
        if(owner->ledger<0||fsync(owner->ledger)!=0||owner->context->outer->output.fd<0
            ||fsync(owner->context->outer->output.fd)!=0)durable=0;
        owner->quarantine_durable=(unsigned)durable;
        if(!durable)(void)engine_failure(owner,"settlement_unknown","quarantine evidence write or persistence failed; no retry and no durable-custody claim");
        char receipt[256];int count=snprintf(receipt,sizeof(receipt),
            "private_quarantine evidence_persisted=%s same_controller_holds_custody=true created=%u reaped=%u group_gone=false retired_coverage=false\n",
            durable?"true":"false",(unsigned)owner->result->created,(unsigned)owner->result->reaped);
        if(count<=0||(size_t)count>=sizeof(receipt)||write(STDERR_FILENO,receipt,(size_t)count)!=count)
            (void)engine_failure(owner,"settlement_unknown","quarantine persistence-status receipt is incomplete");
        uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        engine_max(&owner->result->cleanup_max_ns,used);
        if(!owner->grant.t0||owner->grant.roster_closed)engine_sum(&owner->result->post_cpu_ns,used);
        (void)engine_charge(owner,used,UINT64_MAX,1);
    }
    return 0;
}
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
// Compile-only injected observation, not a second deadline or runtime option.
static unsigned source_known_settlement_deadline_unknown;
#endif
static int engine_settlement_live(struct engine_owner *owner) {
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_settlement_deadline_unknown)
        return engine_quarantine(owner,"source-known injected unknown original deadline observation");
#endif
    uint64_t now;
    if(!known_clock_ns(CLOCK_MONOTONIC,&now))return engine_quarantine(owner,"settlement clock is unknown; original custody retained");
    if(now>=owner->context->outer->deadline)return engine_quarantine(owner,"original settlement deadline reached; original custody retained");
    return 1;
}
static int engine_settlement_poll_unit(struct engine_owner *owner) {
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    if(used==UINT64_MAX)
        return engine_quarantine(owner,"settlement control CPU is unknown or overflowing");
    // Keep the ORIGINAL open member window and C start. Poll-unit debt is
    // paid now; its CPU is subtracted only from the final payment, not Q or
    // the original-window verdict, so every observed nanosecond is paid once.
    uint64_t booked=private_cost.closed_units;int paid=engine_charge(owner,used,used,1);
    // Include the actual ending-clock/bookkeeping tail retained by the shared
    // charge, not only the earlier caller sample. Subtract this from PAYMENT
    // once; the full original controller interval remains in Q.
    uint64_t funded=private_cost.last_unit;
    if(private_cost.closed_units!=booked+1||funded==UINT64_MAX
        ||!engine_safe_sum(owner->settlement_prepaid_cpu,funded,&owner->settlement_prepaid_cpu))
        return engine_quarantine(owner,"settlement prepaid controller tail is unknown or overflowing");
    if(!paid)return engine_quarantine(owner,"settlement control unit or original deadline refused");
    return engine_settlement_live(owner);
}
static int engine_retire(struct engine_owner *owner,int *exit_code) {
    if(owner->settlement_quarantined)return 0;
    if(owner->direct_reaped&&owner->result->group_gone&&owner->result->retired_coverage)return !owner->failed;
    owner->result->phase="settling";owner->retiring=1;
    if(!engine_settlement_live(owner))return 0;
    // Observe the original unreaped handle before settlement. A naturally
    // finished command is reaped without signalling its retired incarnation.
    // Failed STOP remains pending until the independent terminal conjunction;
    // cancellation may only kill retained identities, never issue another STOP.
    if(owner->ticket.pid<=0)return 1;
    int done=owner->direct_reaped!=0;
    if(!done&&!known_exited(owner->ticket.pid,&done)) {
        owner->direct_custody=0;retained_group=0;
        return engine_quarantine(owner,"settlement lacks original unreaped handle");
    }
    if(done&&!owner->direct_reaped&&!owner->failed) {
        int closed=0;
        if(!engine_roster(owner,1,&closed)||!closed)
            engine_failure(owner,"coverage_unknown","normal completion lacks original terminal roster closure");
    }
    if(!owner->direct_reaped&&(!done||owner->failed)) {
        for(unsigned pass=0;pass<2;pass++)for(unsigned i=0;i<owner->retired_count;i++) {
            struct engine_incarnation *member=&owner->retired[i];
            if(!member->current||((member->pid==owner->ticket.pid)!=(pass==1))
                ||(member->pid==owner->ticket.pid&&done)||engine_physical_exit(member))continue;
            if(member->kill_attempted)continue;
            member->kill_attempted=1;member->kill_result=engine_token_signal(&member->token,SIGKILL);
            if(member->kill_result&&member->kill_result!=ESRCH)engine_failure(owner,"settlement_unknown","retained incarnation kill unresolved");
        }
        if(!done&&!owner->group_kill_attempted) {
            owner->group_kill_attempted=1;errno=0;int raw=kill(-owner->ticket.pgid,SIGKILL);
            owner->group_kill_result=raw?errno:0;
            if(owner->group_kill_result&&owner->group_kill_result!=ESRCH)
                engine_failure(owner,"settlement_unknown","final unreaped group kill unresolved");
        }
    }
    while(!owner->direct_reaped) {
        if(!engine_settlement_live(owner))return 0;
        if(!engine_direct_exited(owner,&done)) {
            owner->direct_custody=0;retained_group=0;
            return engine_quarantine(owner,"actual direct wait ownership became unavailable");
        }
        if(done) {
            if(!strcmp(owner->context->outer->case_name,"reap_delay")&&!owner->case_fired) {
                owner->case_fired=1;engine_failure(owner,"settlement_unknown","fixture refused completion while original handle remained unreaped");
                return engine_quarantine(owner,"source-known delayed reap retains the original unreaped handle");
            }
            if(!engine_independent_post_exit(owner)&&!engine_initial_attempt_paid(owner))
                return engine_quarantine(owner,"post-exit independent RI or paid observation refused before own wait4");
            if(engine_observe_wait4(owner))break;
            if(!owner->direct_custody&&!owner->direct_reaped)return engine_quarantine(owner,"actual direct wait4 unavailable");
        }
        if(!engine_settlement_poll_unit(owner))return 0;
        // This is bounded observation of the already consumed original effect,
        // never a fresh deadline, helper grant, signal retry or source work.
        usleep(1000);
    }
    struct rusage usage=owner->retained_wait;int status=owner->retained_status;
    *exit_code=WIFEXITED(status)?WEXITSTATUS(status):-1;
    if(owner->accounting_pending) {
        (void)engine_finish_grant(owner);
    }
    // Once the original roster is physically closed, separate the own-reap
    // bookkeeping from GS2, group checks and ledger formatting. An open grant
    // keeps its original epoch until the full fallback Q is charged below.
    if(owner->grant.roster_closed&&!owner->running) {
        uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        (void)engine_charge(owner,used,used,1);
    }
    if(owner->command==12&&owner->git_signature) {
        if(owner->grant.t0&&!owner->grant.roster_closed)
            return engine_quarantine(owner,"signature custody cannot perform new hash work before original grant closure");
        // GS2 is impossible before the actual direct wait4 above. These reads,
        // rehashes, identity-only unlink and receipts remain in this existing
        // retirement C/post span. A failed observation retains FD quarantine.
        if(!private_git_sig_gs2(owner->git_signature,owner->direct_reaped,
            WIFEXITED(status)&&!engine_cancel_signal,owner->signature_buffer,owner->signature_capacity)) {
            owner->git_signature->quarantined=1;
            engine_failure(owner,private_error,private_detail);
        }
        uint64_t gs2_used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        (void)engine_charge(owner,gs2_used,gs2_used,1);
    }
    uint64_t late=UINT64_MAX,open_independent=0;
    if(owner->grant.t0&&!owner->grant.roster_closed&&!owner->grant_accounted) {
        uint64_t tick_end;
        if(!engine_actual_delta(owner,owner->grant_tick_before,&open_independent,&tick_end))open_independent=UINT64_MAX;
    }
    if(!engine_wait4_late(owner,open_independent,&late))engine_failure(owner,"coverage_unknown","final raw user/system cutoffs are not reconciled");
    if(!engine_original_group_gone(owner)) {
        // An unknown child/token cannot improve through an empty census or an
        // unbounded sleep loop. Preserve original handles and paid raw evidence
        // in explicit quarantine immediately, without claiming group absence.
        return engine_quarantine(owner,"original member/group retirement remains unknown after actual leader reap");
    }
    owner->result->group_gone=1;
    // The implicit verifier is physically reaped by Git, not by a second Q2
    // wait4. Count it only after its retained original exit/RI/token endpoint,
    // reconciled leader rollup and actual leader wait4/group settlement.
    if(owner->child_created&&!owner->child_settled) {
        if(owner->members[1].phase==ENGINE_EXITED&&owner->members[1].exit_abstime
            &&owner->members[1].token_known&&engine_member_exit(owner,1)) {
            owner->child_settled=1;owner->result->reaped++;
        } else engine_failure(owner,"coverage_unknown","implicit verifier lacks retained positive terminal settlement");
    }
    owner->result->retired_coverage=!(owner->git_signature&&owner->git_signature->quarantined)
        &&(!owner->child_created||owner->child_settled);
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID)),cost;
    if(owner->grant.t0&&!owner->grant.roster_closed) {
        owner->grant.cpu_close=engine_now(owner,CLOCK_THREAD_CPUTIME_ID);
        owner->grant.actual_cpu=open_independent;
        // An unfinished original grant includes teardown/late kernel work.
        // The final wait4 reconciliation cannot disappear merely because the
        // signal/window closure failed. Unknown stays unknown and refuses.
        if(late==UINT64_MAX||owner->grant.actual_cpu==UINT64_MAX
            ||!engine_safe_sum(owner->grant.actual_cpu,late,&owner->grant.actual_cpu))owner->grant.actual_cpu=UINT64_MAX;
        engine_sum(&owner->result->late_kernel_ns,late);
        engine_window_cost(owner,engine_now(owner,CLOCK_MONOTONIC),0);owner->running=0;
        uint64_t prepaid=owner->settlement_prepaid_cpu;
        uint64_t actual=owner->grant.controller>=prepaid?owner->grant.controller-prepaid:UINT64_MAX;
        uint64_t outstanding=owner->grant.charge>=prepaid?owner->grant.charge-prepaid:UINT64_MAX;
        if(owner->grant.charge>1000000)engine_failure(owner,"resource_limit","original open settlement Q exceeds one millisecond");
        uint64_t booked=private_cost.closed_units;(void)engine_charge(owner,actual,outstanding,1);
        engine_residual_payment_retained(owner,late,booked,outstanding);
        if(private_cost.closed_units==booked+1&&owner->grant.actual_cpu!=UINT64_MAX) {
            // The full fallback cost contains the final wait4 residual, but
            // only this independent raw RI delta can fund conservation's RHS.
            if(!owner->grant_accounted)(void)engine_credit_member(owner,open_independent,0);
            owner->grant_accounted=1;
        } else owner->conservation_known=0;
    } else {
        if(!engine_safe_sum(used,late,&cost))cost=UINT64_MAX;
        engine_sum(&owner->result->post_cpu_ns,used);engine_sum(&owner->result->late_kernel_ns,late);
        uint64_t booked=private_cost.closed_units;(void)engine_charge(owner,used,cost,1);
        engine_residual_payment_retained(owner,late,booked,cost);
        // This residual is still fully charged, but comes from the SAME final
        // wait4 as the conservation LHS. It is never independent RHS credit.
        if(private_cost.closed_units!=booked+1||cost==UINT64_MAX)owner->conservation_known=0;
    }
    (void)engine_conservation(owner);
    if(usage.ru_maxrss<0||(uint64_t)usage.ru_maxrss>16u*1024u*1024u)engine_failure(owner,"resource_limit","actual wait4 RSS exceeds existing helper limit");
    engine_max(&owner->result->cleanup_max_ns,used);
    uint64_t logged=private_cost.open_cpu;
    (void)engine_record(owner,"reaped");
    uint64_t log_used=engine_interval(logged,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    (void)engine_charge(owner,log_used,log_used,1);
    owner->result->phase="settled";return !owner->failed;
}
static int engine_hold_member_terminal(struct engine_owner *owner,unsigned index) {
    struct engine_member *member=&owner->members[index];
    if(!member->state.pid||!member->token_known||!member->state.start_absolute)return 0;
    // The same admitted final-image and exact RI/audit continuity rules apply.
    // No empty census repairs a missing slot or observed exec discontinuity.
    return engine_member_exit(owner,index);
}
static int engine_hold_late(struct engine_owner *owner,uint64_t *late) {
    return engine_wait4_late(owner,0,late);
}
static int engine_hold_original_bound(const struct engine_owner *owner) {
    if(owner->slots<1||owner->slots>2||owner->ticket.pid<=0||owner->ticket.pgid!=owner->ticket.pid
        ||!owner->ticket.birth||!owner->ticket.start||!owner->ticket.sequence
        ||(!owner->direct_custody&&!owner->direct_reaped)||(!owner->source_admitted&&!owner->grant.source_complete)
        ||owner->context->basis->source_coverage!=127||owner->context->basis->command_coverage!=65535
        ||owner->retired_count<owner->slots||owner->retired_count>8)return 0;
    for(unsigned i=0;i<owner->slots;i++) {
        const struct engine_member *m=&owner->members[i];unsigned bound=0;
        if(m->state.pid<=0||!m->state.birth||!m->state.start_absolute||!m->token_known
            ||(i==0&&(m->state.pid!=owner->ticket.pid||m->state.birth!=owner->ticket.birth||m->state.start_absolute!=owner->ticket.start)))return 0;
        for(unsigned j=0;j<owner->retired_count;j++) {
            const struct engine_incarnation *r=&owner->retired[j];
            if(r->current&&r->pid==m->state.pid&&r->birth==m->state.birth&&r->start==m->state.start_absolute
                &&!memcmp(&r->token,&m->token,sizeof(r->token)))bound=1;
        }
        if(!bound)return 0;
    }
    return 1;
}
static int engine_hold_unknown_cost(struct engine_owner *owner,const char *detail) {
    owner->hold_conservative_sleep=1;
    (void)engine_failure(owner,"clock_unknown",detail);
    if(owner->hold_unknown_units<UINT64_MAX)owner->hold_unknown_units++;
    else private_cost.retained_overflow=1;
    // Raw counters remain in the retained owner. Defer all persistence/status
    // I/O to the post-payment lane below; even equality of debt fields cannot
    // stand in for the successful payment observation that lane requires.
    engine_sum(&owner->result->post_cpu_ns,UINT64_MAX);engine_max(&owner->result->cleanup_max_ns,UINT64_MAX);
    // Same canonical unknown3ms mandatory-settlement charge, never unknown-as-
    // zero. The shared closed-unit owner retains ANY failed scheduling cost,
    // including this raw unknown. Sleeping below is never reported as payment.
    owner->hold_prior_payment_pending=1;
    (void)engine_charge(owner,UINT64_MAX,UINT64_MAX,1);
    return 0;
}
static __attribute__((unused)) int engine_hold_pay_prior(struct engine_owner *owner) {
    // Field equality alone is insufficient: a failed ending-clock read can
    // follow debt_paid assignment. Require the actual payment call to finish
    // successfully before ANY new terminal/roster/wait4/group/status work.
    int recovering=owner->hold_prior_payment_pending||private_has_retained_cost()||private_cost.retained_overflow;
    owner->hold_prior_payment_pending=1;
    if(!private_pay_pending()||private_has_retained_cost()||private_cost.retained_overflow||private_cost.debt_paid!=private_cost.debt)
        return engine_hold_unknown_cost(owner,"prior quarantine debt payment is unproven; observation withheld");
    owner->hold_prior_payment_pending=0;
    if(!recovering)return 1;
    owner->hold_conservative_sleep=1;
    uint64_t used=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
    engine_sum(&owner->result->post_cpu_ns,used);engine_max(&owner->result->cleanup_max_ns,used);
    if(!engine_charge(owner,used,used,1))owner->hold_prior_payment_pending=1;
    return 0; // Booking/payment is this round's only work; revalidate next round.
}
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static unsigned source_known_hold_clock_unknown;
#endif
static int engine_hold_observe(struct engine_owner *owner) {
    if(!owner->settlement_quarantined||owner->quarantine_settled)return owner->quarantine_settled;
    private_cost.custody_hold=1;owner->hold_conservative_sleep=0;
    if(owner->controller_parent&&getppid()!=owner->controller_parent)owner->owner_lost=1;
    if(engine_owner_loss_signal)owner->owner_lost=1;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_hold_clock_unknown)return engine_hold_unknown_cost(owner,"source-known injected unknown hold entry clock");
#endif
    uint64_t wall=0,begin=0;
    if(!known_clock_ns(CLOCK_MONOTONIC,&wall)||!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&begin)
        ||private_cost.open_cpu==UINT64_MAX||begin<private_cost.open_cpu
        ||wall<owner->hold_last_wall||begin<owner->hold_last_cpu) {
        return engine_hold_unknown_cost(owner,"quarantine entry clock unknown or reversed; original owner stays alive");
    }
    owner->hold_last_wall=wall;owner->hold_last_cpu=begin;
    if(!engine_hold_pay_prior(owner))return 0;
    if(owner->hold_unknown_units&&!owner->hold_unknown_recorded) {
        owner->hold_unknown_recorded=1;
        if(owner->ledger>=0){(void)engine_record(owner,"hold_clock_unknown");(void)fsync(owner->ledger);}
    }
    if(owner->owner_lost&&!owner->owner_lost_recorded) {
        owner->owner_lost_recorded=1;
        // This is a new observed owner-loss event, never a retry of the old
        // quarantine write and never a release or replacement admission.
        int recorded=owner->ledger>=0&&engine_record(owner,"owner_lost")&&fsync(owner->ledger)==0;
        char line[256];int n=snprintf(line,sizeof(line),"private_quarantine_owner_lost pid=%d parent=%d recorded=%s outcome=quarantined same_controller_alive=true\n",getpid(),owner->controller_parent,recorded?"true":"false");
        if(n>0&&(size_t)n<sizeof(line))(void)write(STDERR_FILENO,line,(size_t)n);
    }
    int initial_ready=engine_initial_attempt_paid(owner);
    int ready=!owner->coverage_lost&&!owner->pending_identity_lost&&engine_hold_original_bound(owner);
    for(unsigned i=0;i<owner->slots&&ready;i++) {
        if(initial_ready&&owner->post_exit_ri_unavailable&&i==0)continue; // Original WNOWAIT/BSD/token proof; actual wait4 still follows.
        if(owner->hold_terminal_samples==UINT64_MAX)return engine_hold_unknown_cost(owner,"terminal observation counter overflow");
        owner->hold_terminal_samples++;
        if(!engine_hold_member_terminal(owner,i))ready=0;
    }
    if(ready&&!owner->direct_reaped&&!initial_ready) {
        if(owner->hold_roster_checks==UINT64_MAX)return engine_hold_unknown_cost(owner,"roster observation counter overflow");
        owner->hold_roster_checks++;int closed=0;
        if(!engine_roster(owner,1,&closed)||!closed)ready=0;
    }
    if(ready&&!owner->direct_reaped) {
        int done=0;
        if(!known_exited(owner->ticket.pid,&done)||!done)ready=0;
        else {
            if(owner->hold_wait4_calls==UINT64_MAX)return engine_hold_unknown_cost(owner,"wait4 observation counter overflow");
            owner->hold_wait4_calls++;
            if(!engine_independent_post_exit(owner)&&!engine_initial_attempt_paid(owner))ready=0;
            else if(engine_observe_wait4(owner))owner->hold_reap_normal=WIFEXITED(owner->retained_status)&&!engine_cancel_signal;
            else ready=0;
        }
    }
    if(ready) {
        if(owner->hold_group_checks==UINT64_MAX)return engine_hold_unknown_cost(owner,"group observation counter overflow");
        owner->hold_group_checks++;
        if(!owner->direct_reaped||!engine_original_group_gone(owner))ready=0;
    }
    uint64_t late=0;
    if(ready&&!owner->hold_late_charged&&!engine_hold_late(owner,&late))ready=0;
    // Retired/current incarnation absence is checked by the same existing
    // original-group routine. A late unknown child remains held forever until
    // there is real continuity evidence, never a timeout/census-based release.
    uint64_t ended=0,used=UINT64_MAX;
    if(!known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&ended)||private_cost.open_cpu==UINT64_MAX
        ||ended<begin||ended<private_cost.open_cpu||ended<owner->hold_last_cpu)
        return engine_hold_unknown_cost(owner,"quarantine ending clock unknown or reversed; original owner stays alive");
    used=ended-private_cost.open_cpu;owner->hold_last_cpu=ended;
    if(owner->hold_rounds==UINT_MAX||used==UINT64_MAX||UINT64_MAX-owner->hold_cpu_total<used) {
        ready=0;(void)engine_failure(owner,"clock_unknown","quarantine observation cost unknown or overflowing");
    } else {owner->hold_rounds++;owner->hold_cpu_total+=used;if(used>owner->hold_max_cpu)owner->hold_max_cpu=used;}
    engine_sum(&owner->result->post_cpu_ns,used);engine_max(&owner->result->cleanup_max_ns,used);
    uint64_t cost=used;int charge_late=ready&&!owner->hold_late_charged;
    if(charge_late) {
        if(!engine_safe_sum(used,late,&cost))cost=UINT64_MAX;
        // The raw original unknown stays unknown in aggregate metrics; this
        // independently observed final reconciliation is charged once.
    }
    uint64_t booked=private_cost.closed_units;
    if(!engine_charge(owner,used,cost,1))ready=0;
    if(charge_late&&cost!=UINT64_MAX&&private_cost.closed_units==booked+1) {
        // This exact late cost now belongs to the shared retained/booked debt,
        // even when scheduling/payment failed. Do not add it a second time.
        engine_sum(&owner->result->late_kernel_ns,late);owner->hold_late_charged=1;
        engine_residual_payment_retained(owner,late,booked,cost);
    }
    uint64_t paid_wall=0;
    if(!known_clock_ns(CLOCK_MONOTONIC,&paid_wall)||paid_wall<wall||paid_wall<owner->hold_last_wall)
        return engine_hold_unknown_cost(owner,"quarantine payment clock unknown or reversed; original owner stays alive");
    owner->hold_last_wall=paid_wall;
    if(!ready)return 0;
    (void)engine_conservation(owner); // Physical settled-refusal never upgrades the resource verdict.
    if(owner->child_created&&!owner->child_settled){owner->child_settled=1;owner->result->reaped++;}
    // The result already sent remains the immutable prompt refusal. The ledger
    // records physical settlement; it never upgrades that result to success.
    if(!owner->hold_settled_recorded) {
        owner->hold_settled_recorded=1;
        (void)engine_record(owner,owner->owner_lost?"settled_refusal_owner_lost":"settled_refusal");
        if(owner->ledger>=0)(void)fsync(owner->ledger);
        uint64_t cost=engine_interval(private_cost.open_cpu,engine_now(owner,CLOCK_THREAD_CPUTIME_ID));
        engine_sum(&owner->result->post_cpu_ns,cost);engine_max(&owner->result->cleanup_max_ns,cost);
        if(!engine_charge(owner,cost,cost,1))return 0;
    }
    if(private_cost.retained_overflow||private_has_retained_cost()||private_cost.debt_paid!=private_cost.debt)return 0;
    owner->quarantine_settled=1;owner->result->group_gone=owner->result->retired_coverage=1;
    owner->result->phase="settled";owner->result->outcome="refused";
    engine_current=NULL;dispose_private_engine=NULL;return 1;
}
static void engine_quarantine_hold(struct engine_owner *owner) {
    if(!owner||!owner->settlement_quarantined)return;
    private_cost.custody_hold=1;
    // Exactly one existing-wire refusal attempt precedes the lasting hold.
    if(!owner->refusal_attempted)(void)private_emit(owner->context->outer,owner->context->input_sha256,owner->result);
    while(!owner->quarantine_settled) {
        if(engine_hold_observe(owner))break;
        // Ordinary observation already paid its9x debt. On missing clocks no
        // settlement is inferred; sleeping prevents an unknown-clock busy loop.
        usleep(owner->hold_conservative_sleep?27000:10000);
    }
}
static void engine_emergency_dispose(void) {
    struct engine_owner *owner=engine_current;if(!owner)return;
    if(!owner->settlement_quarantined) {
        int status=-1;(void)engine_retire(owner,&status);
        if(!owner->result->group_gone||!owner->result->retired_coverage)
            (void)engine_quarantine(owner,"atexit retains unresolved original settlement");
    }
    if(owner->settlement_quarantined)engine_quarantine_hold(owner);
}
static int engine_capture_drain(struct engine_owner *owner) {
    while(owner->output>=0||owner->errors>=0) {
        uint64_t at=private_cost.open_cpu;
        for(unsigned i=0;i<64;i++) {
            if(!engine_pipe_io(owner))return 0;
            if(owner->output<0&&owner->errors<0)break;
            if(engine_now(owner,CLOCK_THREAD_CPUTIME_ID)-at>=400000)break;
        }
        if(!private_end_unit(at))return engine_failure(owner,private_error,private_detail);
    }
    return 1;
}
static void engine_close_transferred(struct engine_owner *owner) {
    uint64_t at=private_cost.open_cpu;unsigned work=0;int *fds[3]={&owner->input,&owner->output,&owner->errors};
    for(unsigned i=0;i<3;i++)if(*fds[i]>=0) {
        int fd=*fds[i];*fds[i]=-1;work++;
        if(close(fd)!=0)engine_failure(owner,"settlement_unknown","retained pipe close unresolved");
    }
    if(work)(void)private_end_unit(at);
}
static int engine_command(struct engine_owner *owner,struct engine_prepared *prepared,unsigned form,
    const char *home,const char *scratch,const unsigned char *input,size_t input_len,int *status) {
    memset(prepared,0,sizeof(*prepared));for(unsigned i=0;i<2;i++)prepared->input[i]=prepared->output[i]=prepared->errors[i]=-1;
    *status=-1;
    if(engine_current)return engine_failure(owner,"settlement_unknown","a previous physical owner remains retained");
    if(form>=15)return engine_failure(owner,"input_invalid","only fifteen fixed direct forms are admitted");
    if(!owner->git_signature||!owner->git_signature->ready||owner->git_signature->quarantined)
        return engine_failure(owner,"review_unbound","B scratch and selected-case custody are unavailable before birth");
    owner->command=form;owner->slots=private_forms[form].members;
    owner->input=owner->output=owner->errors=-1;owner->stdin_bytes=input;owner->stdin_len=input_len;
    if(input_len>ENGINE_INPUT_MAX){engine_failure(owner,"resource_limit","public input exceeds existing two-MiB limit");goto prepared_done;}
    if(owner->result->attempts>256-owner->slots){engine_failure(owner,"resource_limit","creation reservations exhausted without refund");goto prepared_done;}
    if(!engine_prepare_argv(owner->context,form,home,scratch,prepared)||!engine_prepare_effect(owner,prepared,input_len!=0)) {
        engine_failure(owner,"resource_limit","fixed argv environment or effect preparation refused");goto prepared_done;
    }
    if(!private_revalidate_context(owner->context)){engine_failure(owner,private_error,private_detail);goto prepared_done;}
    owner->source_admitted=1;
    // Consumption itself belongs to the paid preparation unit, before the
    // native effect. Failed/unknown creations never refund these reservations.
    owner->result->attempts+=(uint16_t)owner->slots;owner->ticket.sequence=owner->result->attempts;
    strcpy(owner->image,prepared->image);strcpy(owner->descendant,prepared->descendant);
    if(!private_end_unit(private_cost.open_cpu)){engine_failure(owner,private_error,private_detail);goto prepared_done;}
    engine_current=owner;dispose_private_engine=engine_emergency_dispose;
    int born=engine_birth(owner,prepared);
    while(born&&!owner->failed) {
        if(owner->direct_reaped||owner->accounting_pending)break;
        int done=0;if(!engine_direct_exited(owner,&done)||done)break;
        if(owner->foreground) {
            uint64_t now=engine_now(owner,CLOCK_MONOTONIC);
            if(now<owner->hold_until){usleep(1000);continue;}
            if(!engine_stopped_corroboration(owner))break;owner->foreground=0;
        }
        if(!engine_grant(owner))break;
    }
    if(owner->ticket.pid>0)engine_retire(owner,status);
    if(owner->result->group_gone&&owner->result->retired_coverage){engine_current=NULL;dispose_private_engine=NULL;}
prepared_done:
    engine_dispose_prepared(prepared);
    if(!owner->failed&&!private_revalidate_context(owner->context))engine_failure(owner,private_error,private_detail);
    if(!owner->failed)(void)engine_capture_drain(owner);
    if(!owner->settlement_quarantined)engine_close_transferred(owner);
    if(private_failed&&!owner->failed)engine_failure(owner,private_error,private_detail);
    if(form==12&&owner->verified_descendants!=1)return engine_failure(owner,"coverage_unknown","signature did not observe exactly one expected verifier incarnation");
    return !owner->failed;
}
static int engine_output_text(struct engine_owner *owner,const char *expected) {
    size_t n=owner->out_len;while(n&&(owner->capture[n-1]=='\n'||owner->capture[n-1]=='\r'))n--;
    return n==strlen(expected)&&!memcmp(owner->capture,expected,n);
}
static int engine_output_sha40(struct engine_owner *owner,char sha[41]) {
    size_t n=owner->out_len;if(n==41&&owner->capture[40]=='\n')n--;
    if(n!=40)return 0;memcpy(sha,owner->capture,40);sha[40]=0;return hex_value(sha,40);
}
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
static int engine_blob_pin(struct engine_owner *owner,const char *expected) {
    CC_SHA1_CTX hash;unsigned char value[CC_SHA1_DIGEST_LENGTH];char header[64],hex[41];
    int n=snprintf(header,sizeof(header),"blob %zu",owner->out_len);
    if(n<=0||(size_t)n+1>=sizeof(header)||!CC_SHA1_Init(&hash)||!CC_SHA1_Update(&hash,header,(CC_LONG)n+1))return 0;
    size_t offset=0;
    while(offset<owner->out_len) {
        size_t count=owner->out_len-offset;if(count>4096)count=4096;
        if(!CC_SHA1_Update(&hash,owner->capture+offset,(CC_LONG)count))return 0;
        offset+=count;if(!private_inner_checkpoint())return 0;
    }
    if(!CC_SHA1_Final(value,&hash))return 0;
    for(unsigned i=0;i<sizeof(value);i++)snprintf(hex+2*i,3,"%02x",value[i]);
    return !strcmp(hex,expected);
}
#pragma clang diagnostic pop
// Fixed public Git t7510 corpus adopted by the signed builder fixture. The
// SHA-1 commit relation (including its separate gpgsig-sha256 header) is the
// explicit source/corpus-equivalence premise. Actual object/type/tree/blob,
// ancestry, last-touch and Git cryptographic statuses are still checked below.
// Adapted from Git public presigned objects:
// https://github.com/git/git/blob/v2.54.0/t/t7510-signed-commit.sh
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
static const char *engine_selected_commit(const struct private_engine_context *context) {
    return !strcmp(context->outer->case_name,"corrupt_proof")?context->basis->fixture.corrupt:context->basis->fixture.valid;
}
static int engine_corpus_binding(const struct private_engine_context *context,char tree[41]) {
    const struct private_fixture *f=&context->basis->fixture;
    if(strcmp(f->ancestor,"2ac709c836778e8203ecc9232daa5153a166cc0f")
        ||strcmp(f->valid,"cbceced299d68bdfce3b95f182698f280f1da743")
        ||strcmp(f->corrupt,"0cf6ea6f9b0da82362299e1ccad64e4a5d028de3")
        ||strcmp(f->blob,"d00491fd7e5bb6fa28c517a0bb32b8b506539d4d")
        ||strcmp(f->signer,"73D758744BE721698EC54E8713B6F51ECDDE430D")
        ||f->payload.len!=218||strcmp(f->payload.sha,"58c4ecb71bef6681cc31d9513ac85bbbd130d4d15438e5c57802a983f6091fb3")
        ||f->valid_signature.len!=228||strcmp(f->valid_signature.sha,"d716c2d0807ca1339c774486d55d31768fa9f953df90872f2ceef4cb4568c792")
        ||f->corrupt_signature.len!=228||strcmp(f->corrupt_signature.sha,"cd9777c930d18c7b5b3603ad58a6f63bd64fca702d87bc4e518fca5c1cf0a17b"))
        return private_fail("review_unbound","selected fixture differs from the bound presigned corpus relation");
    unsigned char payload[4096];size_t length=0;
    if(!private_read_bound_ref(context,&f->payload,1,payload,sizeof(payload),&length))return 0;
    uint64_t at=private_cost.open_cpu;
    // Parse the canonical bounded unsigned commit header instead of accepting
    // a caller-provided expected tree. The pinned payload hash binds every
    // remaining header/message byte; the known corpus has this single parent.
    int valid=length==f->payload.len&&length>=95&&!memcmp(payload,"tree ",5)&&payload[45]=='\n'
        &&!memcmp(payload+46,"parent ",7)&&payload[93]=='\n'&&!memcmp(payload+53,f->ancestor,40);
    if(valid){memcpy(tree,payload+5,40);tree[40]=0;valid=hex_value(tree,40);}
    if(!private_end_unit(at))return 0;
    return valid?1:private_fail("review_unbound","unsigned corpus payload tree or parent binding differs");
}
static int engine_signature_semantic(struct engine_owner *owner,int status) {
    const struct private_fixture *fixture=&owner->context->basis->fixture;
    if(status!=0||owner->verified_descendants!=1||!owner->child_created||!owner->child_settled
        ||!owner->git_signature||!owner->git_signature->bound||!owner->git_signature->settled
        ||owner->git_signature->quarantined||owner->git_signature->selected!=private_sig_selected(owner->context))
        return engine_failure(owner,"signature_unexpected","signature requires actual verifier, status and GS1/GS2 settlement");
    int corrupt=!strcmp(owner->context->outer->case_name,"corrupt_proof");
    // The actual Git status below is the reviewed source path's cryptographic
    // result, conditional on exact Git/GPG source/corpus binding. BSD child
    // status is corroboration when retained; Git may already have reaped it.
    // Absence is recorded, never converted to an invented zero/one or a new
    // mandatory racy snapshot. A retained contradictory status still refuses.
    int child_status=(int)owner->members[1].exit_status;
    if(owner->members[1].exit_status_known&&(!WIFEXITED(child_status)||WEXITSTATUS(child_status)!=(corrupt?1:0)))
        return engine_failure(owner,"signature_unexpected","observed verifier exit contradicts the selected cryptographic outcome");
    size_t n=owner->out_len;if(n&&owner->capture[n-1]=='\n')n--;
    if(n<3||owner->capture[1]!=0||memchr(owner->capture,'\n',n))
        return engine_failure(owner,"signature_unexpected","Git cryptographic status framing differs");
    const unsigned char *third=memchr(owner->capture+2,0,n-2);
    if(!third||memchr(third+1,0,n-(size_t)(third+1-owner->capture)))
        return engine_failure(owner,"signature_unexpected","Git cryptographic status fields differ");
    size_t first_len=(size_t)(third-owner->capture)-2,primary_len=n-(size_t)(third+1-owner->capture);
    int fingerprint=first_len==40&&!memcmp(owner->capture+2,fixture->signer,40);
    int primary=primary_len==40&&!memcmp(third+1,fixture->signer,40);
    if(corrupt) {
        // B is Git's actual bad-cryptographic-signature status. E/N, a missing
        // verifier, parse/resource/custody errors or generic nonzero are refused.
        // BADSIG need not expose the VALIDSIG-only fingerprint/primary fields.
        if(owner->capture[0]!='B'||(first_len&&!fingerprint)||(primary_len&&!primary))
            return engine_failure(owner,"signature_unexpected","corrupt fixture did not produce actual bad-signature status");
        owner->result->observed_proof="invalid";
    } else {
        if((owner->capture[0]!='G'&&owner->capture[0]!='U')||(!fingerprint&&!primary))
            return engine_failure(owner,"signature_unexpected","valid fixture lacks G/U and pinned signer or primary");
        owner->result->observed_proof="valid";
    }
    return 1;
}
static int engine_semantic(struct engine_owner *owner,unsigned form,int status) {
    const struct private_fixture *fixture=&owner->context->basis->fixture;char text[4097],sha[41];
    if(status!=0)return engine_failure(owner,"signature_unexpected","fixed public-only command returned unexpected status");
    switch(form) {
        case 0:return engine_output_text(owner,"false");
        case 1:return engine_output_text(owner,owner->context->outer->repo.path);
        // Both fixed refs name the valid commit, including the corrupt case.
        case 2:case 3:return engine_output_sha40(owner,sha)&&!strcmp(sha,fixture->valid);
        case 4:return engine_output_text(owner,"commit");
        case 5:return owner->out_len==0;
        case 6: {
            if(!private_token_text(fixture->record,text,sizeof(text)))return 0;
            char expected[4097];int n=snprintf(expected,sizeof(expected),"100644 blob %s\t%s",fixture->blob,text);
            return n>0&&(size_t)n<sizeof(expected)&&owner->out_len==(size_t)n+1&&!memcmp(owner->capture,expected,(size_t)n)&&owner->capture[n]==0;
        }
        case 7:return engine_blob_pin(owner,fixture->blob);
        case 8:return engine_output_sha40(owner,sha)&&!strcmp(sha,engine_selected_commit(owner->context));
        case 9:return engine_output_sha40(owner,sha)&&!strcmp(sha,fixture->blob);
        case 10: {
            char tree[41];return engine_corpus_binding(owner->context,tree)&&engine_output_sha40(owner,sha)&&!strcmp(sha,tree);
        }
        case 11:return private_token_text(fixture->origin,text,sizeof(text))&&engine_output_text(owner,text);
        case 12:return engine_signature_semantic(owner,status);
        case 13: {
            char fingerprint[46];int n=snprintf(fingerprint,sizeof(fingerprint),":%s:",fixture->signer);
            if(n<=0||(size_t)n>=sizeof(fingerprint))return 0;
            for(size_t i=0;i+(size_t)n<=owner->out_len;i++)if(!memcmp(owner->capture+i,fingerprint,(size_t)n))return 1;
            return 0;
        }
        case 14:return 1; // Exact source-reviewed public import still must return actual status0.
        default:return 0;
    }
}
static int engine_public_home(const struct private_engine_context *context,char home[4097],char scratch[4097]) {
    int root=context->outer->output.fd;
    if(mkdirat(root,"h",0700)!=0||mkdirat(root,"t",0700)!=0)return private_fail("source_changed","fresh private home or scratch claim unresolved");
    int n=snprintf(home,4097,"%s/h",context->outer->output.path),m=snprintf(scratch,4097,"%s/t",context->outer->output.path);
    if(n<=0||m<=0||n>=4097||m>=4097)return private_fail("review_unbound","private home path exceeds fixed bound");
    int dir=openat(root,"h",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(dir<0)return private_fail("source_changed","owned public home custody unavailable");
    const struct private_ref *refs[2]={&context->basis->gpg_conf,&context->basis->common_conf};const char *names[2]={"gpg.conf","common.conf"};
    for(unsigned i=0;i<2;i++) {
        unsigned char bytes[4096];size_t length=0;
        if(!private_read_bound_ref(context,refs[i],0,bytes,sizeof(bytes),&length)){private_close(dir);return 0;}
        int fd=openat(dir,names[i],O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        if(fd<0){private_close(dir);return private_fail("source_changed","exclusive private config creation failed");}
        size_t offset=0;
        while(offset<length) {ssize_t wrote=write(fd,bytes+offset,length-offset);if(wrote<=0){private_close(fd);private_close(dir);return private_fail("source_changed","private config write unresolved");}offset+=(size_t)wrote;}
        int flushed=fsync(fd)==0;private_close(fd);if(!flushed){private_close(dir);return private_fail("source_changed","private config fsync unresolved");}
        if(!private_end_unit(private_cost.open_cpu)){private_close(dir);return 0;}
    }
    int flushed=fsync(dir)==0;private_close(dir);if(!flushed)return private_fail("source_changed","private home publication unresolved");return 1;
}
static int engine_original_claim(const struct private_engine_context *context,struct private_engine_result *result) {
    char claim[512];int n=snprintf(claim,sizeof(claim),"{\"incarnation\":%llu,\"input_sha256\":\"%s\",\"nonce\":\"%s\",\"run_id\":\"%s\"}",
        (unsigned long long)context->outer->incarnation,context->input_sha256,context->outer->nonce,context->outer->run);
    if(n<=0||(size_t)n>=sizeof(claim))return private_fail("input_invalid","original claim bound");
    int fd=openat(context->outer->output.fd,"claim.json",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    if(fd<0) {
        if(errno==EEXIST){result->phase="claim";result->outcome="quarantined";result->group_gone=result->retired_coverage=0;return private_fail("settlement_unknown","original claim exists; no replacement effect admitted");}
        return private_fail("source_changed","exclusive original claim effect unresolved");
    }
    size_t offset=0;
    while(offset<(size_t)n) {ssize_t wrote=write(fd,claim+offset,(size_t)n-offset);if(wrote<=0){private_close(fd);return private_fail("source_changed","original claim write unresolved");}offset+=(size_t)wrote;}
    int flushed=fsync(fd)==0;private_close(fd);
    if(!flushed||fsync(context->outer->output.fd)!=0)return private_fail("source_changed","original claim fsync unresolved");
    result->phase="claim";return private_end_unit(private_cost.open_cpu);
}
static int engine_signature_boundary_available(const struct private_engine_context *context) {
    const struct private_ref *selected=!strcmp(context->outer->case_name,"corrupt_proof")
        ?&context->basis->fixture.corrupt_signature:&context->basis->fixture.valid_signature;
    if(private_forms[15].argc!=5||strcmp(private_forms[15].argv[3],"{git_signature_temp}"))
        return private_fail("review_unbound","current Commands lacks the unexpanded B matcher");
    if(selected->len>=ENGINE_INPUT_MAX)
        return private_fail("review_unbound","selected B bytes plus EOF observation exceed the existing input buffer");
    if(engine_signature_storage.initialized)
        return private_fail("review_unbound","the original B custody state cannot be rebound");
    char tree[41];
    // This is the admitted private conditional source/corpus boundary, never
    // a product approved-lifetime literal or a final qualification receipt.
    return engine_corpus_binding(context,tree);
}
static int engine_signature_prepare_record(const struct private_engine_context *context,
    const struct private_git_sig_temp *signature,int ledger,size_t *bytes,unsigned *records) {
    if(ledger<0||!signature->initialized||!signature->ready||signature->quarantined
        ||signature->context!=context||signature->scratch_fd<0||signature->signature_fd>=0
        ||signature->selected!=private_sig_selected(context)||signature->len!=signature->selected->len
        ||strcmp(signature->sha256,signature->selected->sha))
        return private_fail("review_unbound","prebirth signature receipt lacks retained original custody");
    const char *selected=!strcmp(context->outer->case_name,"corrupt_proof")?"corrupt":"valid";
    char line[512];int n=snprintf(line,sizeof(line),
        "{\"phase\":\"git_sig_prepare\",\"input_sha256\":\"%s\",\"recorded_umask\":%u,\"scratch_dev\":%llu,\"scratch_ino\":%llu,"
        "\"scratch_mode\":%u,\"scratch_uid\":%u,\"selected_case\":\"%s\",\"selected_len\":%llu,\"selected_sha256\":\"%s\"}\n",
        context->input_sha256,(unsigned)signature->recorded_umask,
        (unsigned long long)signature->scratch_stat.st_dev,(unsigned long long)signature->scratch_stat.st_ino,
        (unsigned)(signature->scratch_stat.st_mode&07777),(unsigned)signature->scratch_stat.st_uid,
        selected,(unsigned long long)signature->len,signature->sha256);
    if(n<=0||(size_t)n>=sizeof(line)||*records>=ENGINE_RECORD_MAX||*bytes>ENGINE_CAPTURE_MAX
        ||(size_t)n>ENGINE_CAPTURE_MAX-*bytes)
        return private_fail("resource_limit","prebirth signature receipt exceeds existing ledger reservation");
    // Reserve this one record before its write effect. Partial/failed writes
    // consume the attempt and never receive an automatic retry or a birth.
    *bytes+=(size_t)n;(*records)++;
    if(write(ledger,line,(size_t)n)!=n)
        return private_fail("settlement_unknown","prebirth signature receipt write effect unresolved; no retry");
    if(fsync(ledger)!=0||fsync(context->outer->output.fd)!=0)
        return private_fail("settlement_unknown","prebirth signature receipt persistence unavailable");
    return 1;
}
static int engine_signature_umask_allowed(void) {
    // Reject before creating a home/scratch whose permissions the mask could
    // strip. Prepare later retains and rechecks this same controller property.
    mode_t mask=umask(0);umask(mask);
    return (mask&0600)?private_fail("review_unbound","controller umask strips required Git signature permissions"):1;
}
// Test-only low-level seams exercise this SAME functional flow with scripted
// public outputs and physical-effect records. They are absent from the private
// probe executable and cannot be selected by descriptor, argv or environment.
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
struct private_source_known_effects {
    int (*current)(const struct private_engine_context *);
    int (*command)(struct engine_owner *,struct engine_prepared *,unsigned,
        const char *,const char *,const unsigned char *,size_t,int *);
};
static const struct private_source_known_effects *private_source_known_effects;
#endif
static int engine_pretest_current(const struct private_engine_context *context) {
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(private_source_known_effects&&private_source_known_effects->current)
        return private_source_known_effects->current(context);
#endif
    return private_revalidate_context(context);
}
static int engine_fixed_command(struct engine_owner *owner,struct engine_prepared *prepared,unsigned form,
    const char *home,const char *scratch,const unsigned char *input,size_t length,int *status) {
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(private_source_known_effects&&private_source_known_effects->command)
        return private_source_known_effects->command(owner,prepared,form,home,scratch,input,length,status);
#endif
    return engine_command(owner,prepared,form,home,scratch,input,length,status);
}
static int engine_metric_bound(const struct private_engine_metric *m,uint64_t bound) {
    return m->observed&&m->known&&m->value<=bound;
}
static int engine_complete_result(const struct private_engine_context *context,struct private_engine_result *result) {
    int corrupt=!strcmp(context->outer->case_name,"corrupt_proof");
    if(private_failed||!result->engine_entered||result->semantic_forms!=32767u||result->commands_completed!=15
        ||result->attempts!=16||result->created!=16||result->reaped!=16||result->members_max!=2
        ||!result->group_gone||!result->retired_coverage||engine_current
        ||result->terminal_conservation_checked!=15||result->terminal_conservation_passed!=15
        ||strcmp(result->observed_proof,corrupt?"invalid":"valid"))
        return private_fail("coverage_unknown","complete fixed case lacks actual semantic, verifier or physical settlement coverage");
    if(!engine_metric_bound(&result->ordinary_max_ns,1000000)||!engine_metric_bound(&result->controller_max_ns,1000000)
        ||!engine_metric_bound(&result->cleanup_max_ns,1000000)||!engine_metric_bound(&result->native_cpu_max_ns,2000000)
        ||!engine_metric_bound(&result->native_wall_max_ns,25000000)||!engine_metric_bound(&result->inclusive_max_ns,3000000)
        ||!engine_metric_bound(&result->Q_max_ns,1000000)||!engine_metric_bound(&result->W_ns,UINT64_MAX-1)
        ||!engine_metric_bound(&result->A_ns,UINT64_MAX-1)||!engine_metric_bound(&result->C_ns,UINT64_MAX-1)
        ||!engine_metric_bound(&result->post_cpu_ns,UINT64_MAX-1)||!engine_metric_bound(&result->late_kernel_ns,UINT64_MAX-1)
        ||!engine_metric_bound(&result->helper_group_rss_peak_bytes,16u*1024u*1024u)
        ||!engine_metric_bound(&result->probe_rss_peak_bytes,96u*1024u*1024u)
        ||private_has_retained_cost()||private_cost.debt!=private_cost.debt_paid||private_cost.ordinary_max>1000000)
        return private_fail("resource_limit","fixed case resource, clock or paid-debt conjunction is incomplete");
    result->phase="settled";result->outcome=corrupt?"expected_rejection":"completed";
    result->error_code=result->error_detail=NULL;return 1;
}
int private_command_engine(const struct private_engine_context *context,struct private_engine_result *result) {
    memset(result,0,sizeof(*result));result->phase="validate";result->outcome="refused";result->observed_proof="not_reached";
    result->engine_entered=1;
    result->group_gone=result->retired_coverage=1;
    if(!context||!context->outer||!context->basis||!context->input_sha256||!hex_value(context->input_sha256,64)
        ||context->basis->source_coverage!=127||context->basis->command_coverage!=65535||private_failed) {
        private_fail("review_unbound","command engine requires the internal completely bound conditional context");goto refused;
    }
    if(!engine_owner_priority()||!engine_pretest_current(context)||!engine_signature_boundary_available(context))goto refused;
    if(!engine_signature_umask_allowed())goto refused;
    if(!engine_original_claim(context,result))goto refused;
    char home[4097],scratch[4097];if(!engine_public_home(context,home,scratch))goto refused;
    uint64_t declared=ENGINE_CAPTURE_MAX+ENGINE_INPUT_MAX+sizeof(engine_storage)+sizeof(struct engine_prepared)+sizeof(engine_signature_storage);
    if(engine_current||declared>ENGINE_WORK_LIMIT||private_cost.metadata>ENGINE_WORK_LIMIT-declared) {
        private_fail("resource_limit","declared engine owner exceeds existing work-credit limit");goto refused;
    }
    unsigned char *capture=malloc(ENGINE_CAPTURE_MAX),*input=malloc(ENGINE_INPUT_MAX);
    if(!capture||!input){free(capture);free(input);private_fail("resource_limit","existing work-credit capture/input allocation failed");goto refused;}
    engine_max(&result->metadata_bytes,private_cost.metadata+declared);
    if(!private_git_sig_prepare(context,&engine_signature_storage,scratch)||!private_end_unit(private_cost.open_cpu)) {
        (void)private_git_sig_release(&engine_signature_storage);
        free(input);free(capture);(void)private_end_unit(private_cost.open_cpu);goto refused;
    }
    struct sigaction ignore,old_pipe,old_term,old_child,old_hup;memset(&ignore,0,sizeof(ignore));ignore.sa_handler=SIG_IGN;sigemptyset(&ignore.sa_mask);
    int pipe_set=sigaction(SIGPIPE,&ignore,&old_pipe)==0;ignore.sa_handler=engine_cancel_handler;
    int term_set=sigaction(SIGTERM,&ignore,&old_term)==0;engine_cancel_signal=0;
    ignore.sa_handler=engine_owner_loss_handler;int hup_set=sigaction(SIGHUP,&ignore,&old_hup)==0;engine_owner_loss_signal=0;
    ignore.sa_handler=SIG_DFL;
    int child_set=sigaction(SIGCHLD,&ignore,&old_child)==0;
    int ledger=openat(context->outer->output.fd,"ledger.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    const unsigned order[15]={13,14,0,1,2,3,4,5,6,7,8,9,10,11,12};int all=pipe_set&&term_set&&hup_set&&child_set&&ledger>=0;
    if(!all)private_fail("source_changed","private signal disposition or exclusive raw-ledger setup failed");
    size_t ledger_bytes=0;unsigned record_count=0;
    if(all)all=engine_signature_prepare_record(context,&engine_signature_storage,ledger,&ledger_bytes,&record_count);
    // File/directory persistence and every formatter/write/FD operation above
    // are ordinary prebirth work. Close/pay the full unit before any helper.
    if(!private_end_unit(private_cost.open_cpu))all=0;
    for(unsigned i=0;i<15&&all;i++) {
        unsigned form=order[i];result->phase=i<2?"public_keys":form==12?"git_signature":"git_reads";
        size_t length=0;
        if(form>=13&&!private_read_bound_ref(context,&context->basis->fixture.keys,1,input,ENGINE_INPUT_MAX,&length)){all=0;break;}
        if(engine_current){private_fail("settlement_unknown","prior original effect remains retained");all=0;break;}
        struct engine_owner *owner=&engine_storage;memset(owner,0,sizeof(*owner));
        owner->context=context;owner->result=result;owner->capture=capture;owner->ledger=ledger;
        owner->allocations_owned=1;owner->controller_parent=private_cost.original_parent?private_cost.original_parent:getppid();
        owner->git_signature=&engine_signature_storage;owner->signature_buffer=input;owner->signature_capacity=ENGINE_INPUT_MAX;
        owner->ledger_bytes=ledger_bytes;owner->record_count=record_count;
        struct engine_prepared prepared;int status=-1;
        if(!engine_fixed_command(owner,&prepared,form,home,scratch,input,length,&status)||!engine_semantic(owner,form,status)) {
            if(!private_failed)engine_failure(owner,"signature_unexpected","fixed fixture semantic relationship differs");all=0;
        } else {result->semantic_forms|=1u<<form;result->commands_completed++;}
        ledger_bytes=owner->ledger_bytes;record_count=owner->record_count;
    }
    // Restoration and releases happen only after physical settlement. An
    // ambiguous original effect keeps its fixed owner and borrowed buffers;
    // atexit may use only its retained audit identities, never a reused PGID.
    if(engine_current||engine_signature_storage.quarantined) {result->phase="quarantined";result->outcome="quarantined";goto refused;}
    if(hup_set&&sigaction(SIGHUP,&old_hup,NULL)!=0)all=private_fail("settlement_unknown","private owner-loss disposition restore unresolved");
    if(child_set&&sigaction(SIGCHLD,&old_child,NULL)!=0)all=private_fail("settlement_unknown","private child disposition restore unresolved");
    if(term_set&&sigaction(SIGTERM,&old_term,NULL)!=0)all=private_fail("settlement_unknown","private cancel disposition restore unresolved");
    if(pipe_set&&sigaction(SIGPIPE,&old_pipe,NULL)!=0)all=private_fail("settlement_unknown","private pipe disposition restore unresolved");
    if(ledger>=0){if(fsync(ledger)!=0)all=private_fail("source_changed","raw ledger fsync unresolved");private_close(ledger);}
    if(!private_git_sig_release(&engine_signature_storage)) {
        result->phase="quarantined";result->outcome="quarantined";goto refused;
    }
    free(input);free(capture);if(!private_end_unit(private_cost.open_cpu))all=0;
    if(!all||!engine_pretest_current(context))goto refused;
    engine_max(&result->ordinary_max_ns,private_cost.ordinary_max);
    engine_max(&result->debt_charged_ns,private_cost.debt);engine_max(&result->debt_paid_ns,private_cost.debt_paid);
    if(engine_complete_result(context,result))return 1;
refused:
    result->error_code=private_error;result->error_detail=private_detail;return 0;
}


// END COMMAND_ENGINE

// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// One retained copy of each shared pretest document. No admission from syntax.
struct private_prepared_inner {
    unsigned char source_lock[32768],basis_bytes[49152];
    char pin_tokens[5][8195];
    char sdk_token[8195];
    struct private_ref pin_refs[5];struct private_basis basis;
    struct private_engine_context context;
};
static struct private_prepared_inner private_prepared;
#define PRIVATE_INNER_WORKING_CAPACITY (PRIVATE_CODEC_STACK_CAPACITY+sizeof(private_ref_bindings)+PRIVATE_METADATA_CAPACITY+sizeof(private_guards)+sizeof(private_prepared)+PRIVATE_CORROBORATION_CAPACITY)
_Static_assert(PRIVATE_INNER_WORKING_CAPACITY<=78u*1024u*1024u,"private physical working capacity");
static int private_pin_ref(const struct private_file *pin,struct private_ref *ref,char token[8195]) {
    size_t n=0;token[n++]='"';
    for(const unsigned char *p=(const unsigned char *)pin->path;*p;p++) {
        unsigned c=*p;if(c=='"'||c=='\\'){if(n+2>=8195)return 0;token[n++]='\\';token[n++]=(char)c;}
        else if(c<32)return 0;
        else {if(n+1>=8195)return 0;token[n++]=(char)c;}
    }
    token[n++]='"';token[n]=0;memset(ref,0,sizeof(*ref));ref->path=(struct private_slice){(const unsigned char *)token,n};ref->len=pin->len;memcpy(ref->sha,pin->sha,65);return 1;
}
static int private_guard_matches_pin(const struct private_guard *g,const struct private_file *pin) {
    const struct stat *s=&g->original;
    return g->root==pin->root&&(uint64_t)s->st_dev==pin->dev&&(uint64_t)s->st_ino==pin->ino
        &&s->st_mtimespec.tv_sec==pin->mtime_sec&&(uint32_t)s->st_mtimespec.tv_nsec==pin->mtime_nsec
        &&g->len==pin->len&&!strcmp(g->sha,pin->sha);
}
static int private_prepare_inner(struct private_input *outer,const char *input_hash,size_t descriptor_len) {
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    private_cost.metadata=PRIVATE_INNER_WORKING_CAPACITY;
    memset(&private_prepared,0,sizeof(private_prepared));memset(&private_guards,0,sizeof(private_guards));
    private_prepared.context=(struct private_engine_context){outer,&private_prepared.basis,input_hash};
    private_guards.outer=outer;private_guards.basis=&private_prepared.basis;private_guards.input_hash=input_hash;private_guards.stage=1;
    uint64_t logical=(uint64_t)descriptor_len+outer->source_lock.len+outer->build.len+8192u;
    if(logical>PRIVATE_EVIDENCE_MAX)return private_fail("resource_limit","logical descriptor basis source-lock result reservation exceeds128KiB");
    private_cost.metadata=PRIVATE_INNER_WORKING_CAPACITY;
    if(private_cost.metadata>78u*1024u*1024u)return private_fail("resource_limit","private working-storage reservation");
    const struct private_file *pins[5]={&outer->source_lock,&outer->build,&outer->git,&outer->gpg,&outer->keys};
    for(unsigned i=0;i<5;i++)if(!private_pin_ref(pins[i],&private_prepared.pin_refs[i],private_prepared.pin_tokens[i]))return private_fail("input_invalid","canonical pretest path binding");
    if(!private_end_unit(at))return 0;
    size_t lengths[2]={0,0};
    if(!private_read_bound_ref(&private_prepared.context,&private_prepared.pin_refs[0],0,private_prepared.source_lock,sizeof(private_prepared.source_lock),&lengths[0])
        ||!private_read_bound_ref(&private_prepared.context,&private_prepared.pin_refs[1],0,private_prepared.basis_bytes,sizeof(private_prepared.basis_bytes),&lengths[1]))return 0;
    for(unsigned i=0;i<2;i++) {
        struct private_guard *g=private_guard_open(&private_prepared.context,&private_prepared.pin_refs[i],0);
        if(!g||!private_guard_matches_pin(g,pins[i]))return private_fail("review_unbound","pretest document FilePin identity differs");
    }
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    private_ref_binding_count=0;private_ref_hook=private_register_ref;private_ref_root=0;
    int lock=private_canonical(private_prepared.source_lock,lengths[0])&&private_source_lock_validate(private_prepared.source_lock,lengths[0],outer,&private_prepared.basis.source);
    int parsed=lock&&private_basis_read(private_prepared.basis_bytes,lengths[1],&private_prepared.basis,outer);
    private_ref_hook=NULL;private_ref_root=0;
    int paid=private_end_unit(at);
    if(!paid)return 0;
    if(!parsed)return private_fail("review_unbound","strict pretest source-lock basis or cross-binding validation failed");
    // Compiler/SDK/source/map/archive payloads were checked offline under the
    // signed source-review premise. Runtime opens only admitted candidate,
    // config, prepared-document and explicit fixture inputs; no catalogue scan.
    if(!private_actual_platform(&private_prepared.basis.platform)||!private_actual_config(&private_prepared.context)||!private_actual_fixture(&private_prepared.context))return 0;
    for(unsigned i=2;i<5;i++) {
        struct private_guard *g=private_guard_ref(&private_prepared.context,&private_prepared.pin_refs[i],pins[i]->root);
        if(!g)return 0;
        at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        int identity=private_guard_matches_pin(g,pins[i])&&(i==4?!(g->original.st_mode&077):(g->original.st_mode&0100)!=0);
        if(!private_end_unit(at))return 0;
        if(!identity)return private_fail("review_unbound","actual tool or public-key FilePin identity differs");
    }
    for(unsigned i=0;i<private_prepared.basis.node_count;i++) {
        const struct private_node *node=&private_prepared.basis.nodes[i];
        if(node->kind<2) {if(!private_actual_macho(&private_prepared.context,node))return 0;}
        else if(!private_guard_ref(&private_prepared.context,&node->file,0)
            ||!private_guard_ref(&private_prepared.context,&node->member_index,0))return 0;
    }
    for(unsigned i=0;i<private_prepared.basis.link_count;i++)
        if(!private_guard_ref(&private_prepared.context,&private_prepared.basis.links[i].file,0))return 0;
    // All live selected input/tool/image/platform predicates above were checked.
    // Offline source/build reachability remains the explicitly bound conditional
    // private-test premise, separate from product runtime qualification.
    private_guards.stage=2;
    return private_revalidate_context(&private_prepared.context);
}

static int private_same_pin(const struct private_file *a,const struct private_file *b) {
    return a->root==b->root&&a->dev==b->dev&&a->ino==b->ino&&a->len==b->len&&a->mtime_sec==b->mtime_sec
        &&a->mtime_nsec==b->mtime_nsec&&!strcmp(a->path,b->path)&&!strcmp(a->sha,b->sha);
}
static int private_pretest(struct private_input *v,const char *self) {
    uint64_t now=clock_ns(CLOCK_MONOTONIC);
    if(private_cost.entry>UINT64_MAX-30000000000ULL||v->deadline<=now||v->deadline>private_cost.entry+30000000000ULL)return private_fail("deadline","deadline is not the original same-boot thirty-second window");
    private_cost.deadline=v->deadline;
    struct utsname host;char build[129];size_t len=sizeof(build);
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int known=uname(&host)==0&&sysctlbyname("kern.osversion",build,&len,NULL,0)==0&&len>0&&len<=sizeof(build)&&build[len-1]==0;
    int match=known&&!strcmp(v->arch,"aarch64")&&(!strcmp(host.machine,"arm64")||!strcmp(host.machine,"aarch64"))&&!strcmp(v->os_build,build);
    if(!private_end_unit(at))return 0;if(!match)return private_fail("review_unbound","exact current OS build or architecture differs");
    char actual_image[PROC_PIDPATHINFO_MAXSIZE]={0};at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int image_len=proc_pidpath(getpid(),actual_image,sizeof(actual_image));
    int image_known=image_len>0&&(size_t)image_len<sizeof(actual_image)&&memchr(actual_image,0,sizeof(actual_image))!=NULL;
    if(!private_end_unit(at))return 0;
    if(!image_known||strcmp(actual_image,self))return private_fail("review_unbound","actual current probe image differs from canonical invocation");
    int exe=private_open_walk(-1,actual_image,1,O_RDONLY);if(exe<0)return private_fail("review_unbound","canonical current probe image unavailable");
    struct stat exe_before,exe_after;at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int valid=fstat(exe,&exe_before)==0&&S_ISREG(exe_before.st_mode)&&exe_before.st_uid==geteuid()&&exe_before.st_nlink==1
        &&exe_before.st_size>0&&(uint64_t)exe_before.st_size<=64*1024*1024&&(exe_before.st_mode&0022)==0&&(exe_before.st_mode&06000)==0;
    if(!private_end_unit(at)||!valid){private_close(exe);return private_fail("review_unbound","current probe custody unavailable");}
    char hash[65];valid=private_hash_fd(exe,(uint64_t)exe_before.st_size,hash);
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);valid=valid&&fstat(exe,&exe_after)==0&&exe_before.st_dev==exe_after.st_dev&&exe_before.st_ino==exe_after.st_ino
        &&exe_before.st_size==exe_after.st_size&&exe_before.st_ctimespec.tv_sec==exe_after.st_ctimespec.tv_sec&&exe_before.st_ctimespec.tv_nsec==exe_after.st_ctimespec.tv_nsec;
    if(!private_end_unit(at)){private_close(exe);return 0;}
    if(!valid||strcmp(hash,v->exe)){private_close(exe);return private_fail("review_unbound","current probe image digest differs");}
    if(!private_named_current(exe,-1,actual_image,1,&exe_before)){private_close(exe);return 0;}
    if(!private_close(exe))return 0;
    const struct private_file *projections[]={&v->review,&v->closure,&v->config,&v->commands,&v->manifest};
    for(unsigned i=0;i<sizeof(projections)/sizeof(projections[0]);i++)if(!private_same_pin(&v->build,projections[i]))
        return private_fail("review_unbound","six projection FilePins must be exactly identical");
    if(v->build.root||strcmp(v->build.path,"provenance/pretest-basis.json")||v->source_lock.root)
        return private_fail("review_unbound","basis or source lock belongs to wrong exact candidate root");
    struct private_file *files[]={&v->source_lock,&v->build};
    if(v->source_lock.len>32768||v->build.len>49152)return private_fail("resource_limit","source lock or basis size ceiling");
    uint64_t total=v->source_lock.len+v->build.len;
    // Shared Basis is counted once; the bounded actual result allowance is8KiB.
    // The later builder must charge its final qualification envelope as well.
    if(total>PRIVATE_EVIDENCE_MAX-PRIVATE_INPUT_MAX-8192u)return private_fail("resource_limit","combined prepared evidence reservation exceeds128KiB");
    if(v->git.root||v->gpg.root||v->keys.root!=1||!v->keys.len)return private_fail("review_unbound","tool or key belongs to wrong exact root");
    if(v->git.len>64u*1024u*1024u||v->gpg.len>64u*1024u*1024u||v->keys.len>2u*1024u*1024u)
        return private_fail("resource_limit","existing tool/key allocation ceiling");
    if(!private_dir_open(&v->prefix)||!private_dir_open(&v->repo)||!private_dir_open(&v->output))return 0;
    if((v->prefix.dev==v->repo.dev&&v->prefix.ino==v->repo.ino)||(v->output.dev==v->prefix.dev&&v->output.ino==v->prefix.ino)
        ||(v->output.dev==v->repo.dev&&v->output.ino==v->repo.ino))return private_fail("review_unbound","candidate fixture and output roots must be distinct");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);DIR *out=fdopendir(dup(v->output.fd));int empty=out!=NULL;
    if(out){for(unsigned i=0;i<3;i++){errno=0;struct dirent *e=readdir(out);if(!e){if(errno)empty=0;break;}if(strcmp(e->d_name,".")&&strcmp(e->d_name,".."))empty=0;}closedir(out);}
    if(!private_end_unit(at))return 0;if(!empty)return private_fail("review_unbound","output is not a fresh owned empty directory");
    // The shared documents are read/hash/parsed exactly once by their retained
    // owner below. Actual tools/keys/closure stay prebirth until corroborated.
    (void)files;
    if(!private_root_current(&v->prefix)||!private_root_current(&v->repo)||!private_root_current(&v->output))return 0;
    // Outer custody only. The caller must still refuse the missing inner tuple
    // after closing its original input-path fence; this is never a source premise.
    return 1;
}
static const char *private_metric_text(const struct private_engine_metric *metric,char bytes[32]) {
    if(!metric->observed||!metric->known)return "null";
    snprintf(bytes,32,"%llu",(unsigned long long)metric->value);return bytes;
}
static int private_json_quoted(const char *text,char *bytes,size_t capacity) {
    size_t used=0;if(capacity<3)return 0;bytes[used++]='"';
    for(const unsigned char *p=(const unsigned char *)text;*p;p++) {
        unsigned c=*p;if(c=='"'||c=='\\'){if(capacity-used<3)return 0;bytes[used++]='\\';bytes[used++]=(char)c;}
        else if(c<32){
            const char *escape=c==8?"\\b":c==9?"\\t":c==10?"\\n":c==12?"\\f":c==13?"\\r":NULL;
            if(escape){if(capacity-used<3)return 0;bytes[used++]=escape[0];bytes[used++]=escape[1];}
            else {if(capacity-used<7)return 0;int n=snprintf(bytes+used,capacity-used,"\\u%04x",c);if(n!=6)return 0;used+=6;}
        }
        else {if(capacity-used<2)return 0;bytes[used++]=(char)c;}
    }
    if(capacity-used<2)return 0;bytes[used++]='"';bytes[used]=0;return 1;
}
static void private_result_cost(struct private_engine_result *result) {
    engine_max(&result->ordinary_max_ns,private_cost.ordinary_max);
    engine_max(&result->cleanup_max_ns,private_cost.close_max);
    engine_max(&result->debt_charged_ns,private_cost.debt);engine_max(&result->debt_paid_ns,private_cost.debt_paid);
    engine_max(&result->metadata_bytes,private_cost.metadata);
    if(private_cost.rss_known)engine_max(&result->probe_rss_peak_bytes,private_cost.rss_peak);
    if(private_failed) {
        result->error_code=private_error;result->error_detail=private_detail;
        if(strcmp(result->outcome,"quarantined"))result->outcome=result->created?"failed":"refused";
    }
}
static int private_format(const struct private_input *v,const char *input_hash,const struct private_engine_result *result,char *bytes,size_t capacity) {
    const struct private_engine_metric *metrics[]={&result->A_ns,&result->C_ns,&result->Q_max_ns,&result->W_ns,
        &result->cleanup_max_ns,&result->controller_max_ns,&result->debt_charged_ns,&result->debt_paid_ns,
        &result->helper_group_rss_peak_bytes,&result->inclusive_max_ns,&result->late_kernel_ns,
        &result->native_cpu_max_ns,&result->native_wall_max_ns,&result->ordinary_max_ns,&result->post_cpu_ns,&result->probe_rss_peak_bytes};
    char numbers[16][32];const char *m[16];for(unsigned i=0;i<16;i++)m[i]=private_metric_text(metrics[i],numbers[i]);
    char error[4096]="null";
    if(result->error_code) {
        char code[256],detail[3075];
        if(!private_json_quoted(result->error_code,code,sizeof(code))||!private_json_quoted(result->error_detail?result->error_detail:"",detail,sizeof(detail)))return -1;
        int n=snprintf(error,sizeof(error),"{\"code\":%s,\"detail\":%s}",code,detail);if(n<0||(size_t)n>=sizeof(error))return -1;
    }
    uint64_t metadata=result->metadata_bytes.observed&&result->metadata_bytes.known?result->metadata_bytes.value:private_cost.metadata;
    // Exact canonical field order. All values come from this invocation's
    // retained result; unknown/unobserved observations stay null, never zero.
    return snprintf(bytes,capacity,"{\"availability\":\"not_applicable_private_test\",\"basis\":{\"kind\":\"conditional_on_bound_source_review\",\"review_sha256\":\"%s\",\"whole_command_lifetime_proof\":false},\"binding\":{\"build\":\"%s\",\"closure\":\"%s\",\"commands\":\"%s\",\"config\":\"%s\",\"git\":\"%s\",\"gpg\":\"%s\",\"source_lock\":\"%s\",\"source_review\":\"%s\"},\"case\":\"%s\",\"error\":%s,\"incarnation\":%llu,\"input_sha256\":\"%s\",\"kind\":\"private_candidate_result\",\"metrics\":{\"A_ns\":%s,\"C_ns\":%s,\"Q_max_ns\":%s,\"W_ns\":%s,\"attempts\":%u,\"cleanup_max_ns\":%s,\"controller_max_ns\":%s,\"debt_charged_ns\":%s,\"debt_paid_ns\":%s,\"helper_group_rss_peak_bytes\":%s,\"inclusive_max_ns\":%s,\"late_kernel_ns\":%s,\"members_max\":%u,\"metadata_bytes\":%llu,\"native_cpu_max_ns\":%s,\"native_wall_max_ns\":%s,\"ordinary_max_ns\":%s,\"post_cpu_ns\":%s,\"probe_rss_peak_bytes\":%s},\"nonce\":\"%s\",\"observed_proof\":\"%s\",\"outcome\":\"%s\",\"phase\":\"%s\",\"probe\":{\"exe_sha256\":\"%s\",\"source_sha256\":\"%s\"},\"profile\":\"read-helper-native/1\",\"run_id\":\"%s\",\"runtime_approved\":false,\"settlement\":{\"created\":%u,\"group_gone\":%s,\"reaped\":%u,\"retired_coverage\":%s}}\n",
        v->review.sha,v->build.sha,v->closure.sha,v->commands.sha,v->config.sha,v->git.sha,v->gpg.sha,v->source_lock.sha,v->review.sha,v->case_name,
        error,(unsigned long long)v->incarnation,input_hash,m[0],m[1],m[2],m[3],(unsigned)result->attempts,m[4],m[5],m[6],m[7],m[8],m[9],m[10],
        (unsigned)result->members_max,(unsigned long long)metadata,m[11],m[12],m[13],m[14],m[15],v->nonce,result->observed_proof,result->outcome,result->phase,
        v->exe,v->source,v->run,(unsigned)result->created,result->group_gone?"true":"false",(unsigned)result->reaped,result->retired_coverage?"true":"false");
}
static int private_sample_rss(void) {
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct proc_taskinfo info;
    int got=proc_pidinfo(getpid(),PROC_PIDTASKINFO,0,&info,sizeof(info));
    if(got==(int)sizeof(info)){private_cost.rss_known=1;if(info.pti_resident_size>private_cost.rss_peak)private_cost.rss_peak=info.pti_resident_size;}
    int paid=private_end_unit(at);if(!paid)return 0;
    if(got!=(int)sizeof(info))return private_fail("coverage_unknown","probe resident-memory observation unavailable");
    if(info.pti_resident_size>96*1024*1024)return private_fail("resource_limit","probe resident-memory ceiling exceeded");return 1;
}
static int private_emit(const struct private_input *v,const char *input_hash,struct private_engine_result *result) {
    struct engine_owner *held=engine_current&&engine_current->result==result&&engine_current->settlement_quarantined?engine_current:NULL;
    int can_emit=!engine_current||held;
    if(held&&held->refusal_attempted)return held->refusal_complete;
    if(held)held->refusal_attempted=1;
    if(!private_terminal.control_known) {
        private_terminal.control_known=1;private_terminal.control_ok=!private_failed&&!result->error_code;
    }
    private_terminal.started=1;
    (void)private_sample_rss();private_result_cost(result);
    // Keep the one stdout schema and its original pre-publication cutoff.
    // Actual persistence and all later costs are independent raw observations.
    uint64_t format_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    char bytes[8192];int len=private_format(v,input_hash,result,bytes,sizeof(bytes));
    private_terminal.format_start=format_start;private_terminal.format_end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    private_terminal.format_paid=(unsigned)private_end_unit(format_start);private_terminal.format_inclusive=private_cost.last_unit;
    if(len<=0||(size_t)len>=sizeof(bytes))private_fail("settlement_unknown","private result formatter did not produce one bounded result");
    uint64_t write_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);int close_paid=1;
    if(can_emit&&(private_terminal.format_paid||held)&&len>0&&(size_t)len<sizeof(bytes)&&v->output.fd>=0) {
        private_terminal.persistence_attempted=1;
        int fd=openat(v->output.fd,"result.json",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        private_terminal.persist_open=fd>=0;
        if(fd<0){private_terminal.persistence_errno=errno;private_fail("settlement_unknown","exclusive private result creation failed; no retry");}
        else {
            ssize_t n=write(fd,bytes,(size_t)len);private_terminal.persist_write=n==len;
            if(n!=len){private_terminal.persistence_errno=errno;private_fail("settlement_unknown","private result file write incomplete; no retry");}
            private_terminal.persist_fsync=fsync(fd)==0;
            if(!private_terminal.persist_fsync){private_terminal.persistence_errno=errno;private_fail("settlement_unknown","private result file fsync failed; no retry");}
            int actual_closed=0;close_paid=private_close_observed(fd,&actual_closed);private_terminal.persist_close=(unsigned)actual_closed;
            if(!actual_closed){private_terminal.persistence_errno=errno;private_fail("settlement_unknown","private result file close failed; no retry");}
            private_terminal.persist_dir_fsync=fsync(v->output.fd)==0;
            if(!private_terminal.persist_dir_fsync){private_terminal.persistence_errno=errno;private_fail("settlement_unknown","private result directory fsync failed; no retry");}
        }
    }
    private_terminal.persist_ok=private_terminal.persist_open&&private_terminal.persist_write&&private_terminal.persist_fsync
        &&private_terminal.persist_close&&private_terminal.persist_dir_fsync;
    private_terminal.persist_unknown=private_terminal.persist_open&&!private_terminal.persist_ok;
    private_terminal.persistence_paid=(unsigned)private_end_unit(write_start)&&close_paid;
    write_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);private_terminal.write_start=write_start;
    int flags=fcntl(STDOUT_FILENO,F_GETFL);
    if(can_emit&&(private_terminal.format_paid||held)&&len>0&&(size_t)len<sizeof(bytes)&&flags>=0&&fcntl(STDOUT_FILENO,F_SETFL,flags|O_NONBLOCK)==0) {
        ssize_t n=write(STDOUT_FILENO,bytes,(size_t)len);private_terminal.stdout_complete=n==len;
        if(n!=len)private_terminal.stdout_errno=errno;
        if(fcntl(STDOUT_FILENO,F_SETFL,flags)!=0){private_terminal.stdout_errno=errno;private_terminal.stdout_complete=0;}
    } else private_terminal.stdout_errno=errno;
    if(!private_terminal.stdout_complete)private_fail("settlement_unknown","private result stdout emission incomplete; no retry");
    private_terminal.write_end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    private_terminal.write_paid=(unsigned)private_end_unit(write_start);private_terminal.write_inclusive=private_cost.last_unit;
    if(held)held->refusal_complete=private_terminal.stdout_complete;
    // No final receipt here: a held owner may still have a long settlement,
    // and ordinary output-close/exit-tail work has not happened yet.
    return private_terminal.stdout_complete&&private_terminal.persist_ok&&private_terminal.format_paid
        &&private_terminal.persistence_paid&&private_terminal.write_paid&&!private_failed;
}
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
static unsigned source_known_tail_over_cpu;
#endif
static void private_finish_diagnostics(const char *phase,unsigned created,unsigned reaped,int output_closed) {
    private_terminal.started=1;
#ifdef DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS
    if(source_known_tail_over_cpu) {
        uint64_t until=clock_ns(CLOCK_THREAD_CPUTIME_ID)+1100000;
        while(clock_ns(CLOCK_THREAD_CPUTIME_ID)<until){}
    }
#endif
    fprintf(stderr,"private_candidate_exit_tail output_closed=%s close_max_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu ordinary_max_ns=%llu\n",
        output_closed?"true":"false",(unsigned long long)private_cost.close_max,(unsigned long long)private_cost.debt,
        (unsigned long long)private_cost.debt_paid,(unsigned long long)private_cost.ordinary_max);
    (void)private_end_unit(private_cost.open_cpu);private_terminal.tail_complete=1;
    _Static_assert(11u*2304u+4096u+256u<=32768u,"bounded complete raw diagnostics stay within32KiB");
    // Fixed code histogram covers every failure call, including failures after
    // the legacy first stdout error. Detail samples are bounded and explicitly
    // marked when truncated; counts/overflow never silently imply completeness.
    for(unsigned i=0;i<11;i++)if(private_errors[i].count) {
        char first[967],last[967],line[2304];
        int quoted=private_json_quoted(private_errors[i].first,first,sizeof(first))&&private_json_quoted(private_errors[i].last,last,sizeof(last));
        int n=quoted?snprintf(line,sizeof(line),"{\"code\":\"%s\",\"count\":%llu,\"details_truncated\":%s,\"first_detail\":%s,\"kind\":\"private_candidate_error\",\"last_detail\":%s}\n",
            private_error_codes[i],(unsigned long long)private_errors[i].count,private_errors[i].truncated?"true":"false",first,last):-1;
        if(n<=0||(size_t)n>=sizeof(line)||write(STDERR_FILENO,line,(size_t)n)!=n)private_terminal.diagnostics_ok=0;
        (void)private_end_unit(private_cost.open_cpu);
    }
    // This final fixed raw receipt follows the PAID complete tail and error
    // diagnostics. Its own minimal formatting/write is the explicit observation
    // cutoff, witnessed externally by the parent wait4, not a recursive claim.
    char counts[512];size_t used=0;
    for(unsigned i=0;i<11;i++) {
        int n=snprintf(counts+used,sizeof(counts)-used,"%s%llu",i?",":"",(unsigned long long)private_errors[i].count);
        if(n<=0||(size_t)n>=sizeof(counts)-used){private_terminal.diagnostics_ok=0;counts[0]=0;break;}used+=(size_t)n;
    }
    char final[4096];int final_len=snprintf(final,sizeof(final),"private_candidate_terminal profile=read-helper-native/1 stage=%s metrics_cutoff=pre_result_emission metadata_kind=declared_peak_capacity terminal_cutoff=after_paid_tail_and_error_diagnostics_before_final_receipt runtime_approved=false helper_created=%u helper_reaped=%u result_complete=%s control_verdict=%s persistence_verdict=%s terminal_cost_verdict=%s tail_complete=%s persistence_attempted=%s persistence_open=%s persistence_write=%s persistence_fsync=%s persistence_close=%s persistence_directory_fsync=%s persistence_errno=%d stdout_errno=%d persistence_paid=%s formatter_cpu_start_ns=%llu formatter_cpu_end_ns=%llu formatter_inclusive_ns=%llu write_cpu_start_ns=%llu write_cpu_end_ns=%llu write_inclusive_ns=%llu close_max_cpu_ns=%llu probe_rss_peak_bytes=%llu rss_known=%s ordinary_max_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu retained_units=%llu retained_overflow=%u format_paid=%s write_paid=%s error_count=%llu error_codes=cancelled,clock_unknown,coverage_unknown,deadline,input_invalid,resource_limit,review_unbound,settlement_unknown,signature_unexpected,source_changed,other error_counts=%s errors_complete=%s diagnostics_complete=%s parent_actual_reap_required=true\n",
        phase,created,reaped,private_terminal.stdout_complete?"true":"false",private_terminal.control_known?(private_terminal.control_ok?"passed":"failed"):"unknown",
        private_terminal.persistence_attempted?(private_terminal.persist_unknown?"unknown":private_terminal.persist_ok?"persisted":"not_persisted"):"unknown",private_terminal.cost_unknown?"unknown":private_terminal.cost_ok?"passed":"failed",private_terminal.tail_complete?"true":"false",private_terminal.persistence_attempted?"true":"false",
        private_terminal.persist_open?"true":"false",private_terminal.persist_write?"true":"false",private_terminal.persist_fsync?"true":"false",private_terminal.persist_close?"true":"false",private_terminal.persist_dir_fsync?"true":"false",private_terminal.persistence_errno,private_terminal.stdout_errno,
        private_terminal.persistence_paid?"true":"false",(unsigned long long)private_terminal.format_start,(unsigned long long)private_terminal.format_end,(unsigned long long)private_terminal.format_inclusive,
        (unsigned long long)private_terminal.write_start,(unsigned long long)private_terminal.write_end,(unsigned long long)private_terminal.write_inclusive,(unsigned long long)private_cost.close_max,
        (unsigned long long)private_cost.rss_peak,private_cost.rss_known?"true":"false",(unsigned long long)private_cost.ordinary_max,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,
        (unsigned long long)private_cost.retained_units,private_cost.retained_overflow,private_terminal.format_paid?"true":"false",private_terminal.write_paid?"true":"false",(unsigned long long)private_error_count,counts,
        private_errors_overflow?"false":"true",private_terminal.diagnostics_ok?"true":"false");
    if(final_len<=0||(size_t)final_len>=sizeof(final)||write(STDERR_FILENO,final,(size_t)final_len)!=final_len) {
        private_terminal.diagnostics_ok=0;
        private_fail("settlement_unknown","final raw receipt write incomplete; no retry or completeness claim");
    }
}
static int private_early_refusal(int fd,const char *code,const char *detail) {
    private_terminal.control_known=1;private_terminal.control_ok=0;private_terminal.started=1;
    private_fail(code,detail);if(fd>=0)(void)private_close(fd);
    fprintf(stderr,"private candidate refused: %s %s\n",code,detail);
    private_finish_diagnostics("prebirth",0,0,1);return 81;
}
static int private_dispatch(int outer_ready,int inner_ready,int original_ready,
    const struct private_engine_context *context,struct private_engine_result *result) {
    if(!outer_ready||!inner_ready||!original_ready)return 0;
    return private_command_engine(context,result);
}
static int private_candidate(const char *self,const char *path,const char *expected) {
    memset(&private_cost,0,sizeof(private_cost));private_failed=0;private_error="input_invalid";private_detail="strict private input decoding failed";private_cost.entry=clock_ns(CLOCK_MONOTONIC);private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);private_cost.original_parent=getppid();private_reset_diagnostics();
    if(private_cost.entry>UINT64_MAX-30000000000ULL)return private_early_refusal(-1,"clock_unknown","entry overflow");
    private_cost.deadline=private_cost.entry+30000000000ULL;
    uint64_t priority_at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int priority=engine_owner_priority();
    if(!private_end_unit(priority_at)||!priority)return private_early_refusal(-1,"review_unbound","private owner priority");
    if(!hex_value(expected,64))return private_early_refusal(-1,"input_invalid","expected digest");
    int fd=private_open_walk(-1,path,1,O_RDONLY);if(fd<0)return private_early_refusal(-1,"input_invalid","descriptor custody");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);struct stat before,after;int valid=fstat(fd,&before)==0&&S_ISREG(before.st_mode)
        &&before.st_uid==geteuid()&&before.st_nlink==1&&(before.st_mode&077)==0&&before.st_size>0&&before.st_size<=PRIVATE_INPUT_MAX;
    if(!private_end_unit(at)||!valid)return private_early_refusal(fd,"input_invalid","descriptor identity");
    unsigned char bytes[PRIVATE_INPUT_MAX+1];size_t count=0;
    while(count<(size_t)before.st_size){at=clock_ns(CLOCK_THREAD_CPUTIME_ID);ssize_t n=read(fd,bytes+count,(size_t)before.st_size-count>4096?4096:(size_t)before.st_size-count);
        int paid=private_end_unit(at);if(n<=0||!paid)return private_early_refusal(fd,"source_changed","descriptor read");count+=(size_t)n;}
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);valid=fstat(fd,&after)==0&&before.st_dev==after.st_dev&&before.st_ino==after.st_ino&&before.st_size==after.st_size
        &&before.st_mtimespec.tv_sec==after.st_mtimespec.tv_sec&&before.st_mtimespec.tv_nsec==after.st_mtimespec.tv_nsec
        &&before.st_ctimespec.tv_sec==after.st_ctimespec.tv_sec&&before.st_ctimespec.tv_nsec==after.st_ctimespec.tv_nsec;
    static char hash[65];private_hash(bytes,count,hash);static struct private_input input;
    valid=valid&&!strcmp(hash,expected)&&private_decode(bytes,count,&input);
    // Working-copy capacity is distinct from the logical128KiB evidence sum.
    // All concrete guard/typed capacities stay inside the existing78MiB work.
    private_cost.metadata=PRIVATE_INNER_WORKING_CAPACITY;
    if(!private_end_unit(at)||!valid)return private_early_refusal(fd,"input_invalid","changed hash or noncanonical schema");
    static struct private_engine_result result;
    memset(&result,0,sizeof(result));result.phase="validate";result.outcome="refused";result.observed_proof="not_reached";
    result.group_gone=result.retired_coverage=1;
    int outer=private_pretest(&input,self);
    int inner=outer?private_prepare_inner(&input,hash,count):0;
    int original=private_named_current(fd,-1,path,1,&before);
    int completed=private_dispatch(outer,inner,original,&private_prepared.context,&result);
    private_terminal.control_known=1;private_terminal.control_ok=completed&&!private_failed;private_terminal.started=1;
    // Borrowed guards and the original outer roots outlive their engine use.
    // Unresolved ownership retains them for the exact atexit disposal owner.
    if(!engine_current&&!engine_signature_storage.quarantined) {
        private_guards_dispose();
        for(unsigned i=0;i<2;i++){struct private_dir *d=i==0?&input.prefix:&input.repo;if(d->fd>=0){private_close(d->fd);d->fd=-1;}}
        (void)private_close(fd);
    } else {result.phase="quarantined";result.outcome="quarantined";result.group_gone=result.retired_coverage=0;}
    int emitted=private_emit(&input,hash,&result);
    if(engine_current&&engine_current->settlement_quarantined) {
        struct engine_owner *held=engine_current;engine_quarantine_hold(held);
        // All releases follow the same controller's actual terminal proof.
        engine_close_transferred(held);
        struct private_git_sig_temp *sig=held->git_signature;
        if(sig&&sig->initialized) {
            int *fds[]={&sig->signature_fd,&sig->transient_fd,&sig->scratch_fd};
            for(unsigned i=0;i<3;i++)if(*fds[i]>=0){int owned=*fds[i];*fds[i]=-1;(void)private_close(owned);}
            // GS2 semantic/quarantine flags remain unchanged: closing owned
            // descriptors after group proof cannot turn a failed proof valid.
        }
        if(held->allocations_owned){free(held->signature_buffer);free(held->capture);held->signature_buffer=held->capture=NULL;held->allocations_owned=0;(void)private_end_unit(private_cost.open_cpu);}
        private_guards_dispose();
        for(unsigned i=0;i<2;i++){struct private_dir *d=i==0?&input.prefix:&input.repo;if(d->fd>=0){private_close(d->fd);d->fd=-1;}}
        (void)private_close(fd);
    }
    int output_closed=input.output.fd<0;
    if(!engine_current&&input.output.fd>=0) {
        int actual_closed=0;(void)private_close_observed(input.output.fd,&actual_closed);
        input.output.fd=-1;output_closed=actual_closed;
    }
    private_finish_diagnostics(result.phase,result.created,result.reaped,output_closed);
    return completed&&emitted&&!private_failed&&private_terminal.cost_ok&&private_terminal.diagnostics_ok&&(!strcmp(input.case_name,"positive")||!strcmp(input.case_name,"corrupt_proof"))?0:81;
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
    if (!strcmp(argv[1],"private-candidate")) {require(argc==4,"private candidate argc");return private_candidate(argv[0],argv[2],argv[3]);}
    require(0,"unknown mode");return 81;
}
