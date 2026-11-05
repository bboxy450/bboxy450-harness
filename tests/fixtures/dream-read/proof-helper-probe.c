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
    if (clock_gettime(id, &t)) _exit(80);
    return (uint64_t)t.tv_sec * 1000000000ULL + (uint64_t)t.tv_nsec;
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
        require(clock_ns(CLOCK_MONOTONIC)<until&&clock_ns(CLOCK_THREAD_CPUTIME_ID)-cpu<=1000000,"bounded actual control stop acknowledgement");
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
    printf("{\"created_suspended\":true,\"cloexec_default\":true,\"audit_token_signals\":true,\"running_child_stop_applied\":true,\"stale_pid_version_control\":true,\"mach_timebase_numer\":%u,\"mach_timebase_denom\":%u,\"sampled_child_cpu_raw\":%llu,\"initial_native_cpu_ns\":%llu,\"confirmed_native_cpu_ns\":%llu,\"native_bootstrap_reconciliation_ns\":%llu,\"stop_confirm_wall_ns\":%llu,\"held_user_cpu_delta_ns\":0,\"held_kernel_cpu_ns\":%llu,\"held_kernel_paid_debt_ns\":%llu,\"threads\":1,\"confirmed_runnable\":0,\"stale_audit_error\":%d,\"leader\":%d,\"child\":%d,\"pgid\":%d,\"leader_birth\":%llu,\"child_birth\":%llu,\"members\":2,\"sampled_child_cpu_ns\":%llu,\"waited_child_cpu_ns\":%llu,\"leader_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true}\n",
        timebase.numer,timebase.denom,(unsigned long long)child_raw_cpu,(unsigned long long)initial_cpu,(unsigned long long)before.cpu,(unsigned long long)bootstrap_cpu,(unsigned long long)confirm_wall,
        (unsigned long long)held_kernel_cpu,(unsigned long long)(9*held_kernel_cpu),stale_error,pid, child_pid, pid, (unsigned long long)before.birth, (unsigned long long)child.birth,
        (unsigned long long)child_cpu, (unsigned long long)waited_cpu, (unsigned long long)ru_cpu(&usage),
        (unsigned long long)peak_rss, (unsigned long long)create_cpu, (unsigned long long)create_wall);
    return 0;
}
static void child_image_phase(const struct image *child,const char *git,const char *gpg,
    int *phase,unsigned *observations,uint64_t *peak) {
    if(strcmp(child->path,git)==0) {
        require(*phase==0&&++*observations<=16&&child->cpu<=1000000,"bounded monotonic approved Git fork-copy phase");
        if(child->cpu>*peak)*peak=child->cpu;
        fprintf(stderr,"git_fork_copy pid=%d birth=%llu path=%s cpu_ns=%llu user_ns=%llu system_ns=%llu status=%u runnable=%u thread_state=%d flags=%u observation=%u\n",child->pid,(unsigned long long)child->birth,child->path,(unsigned long long)child->cpu,
            (unsigned long long)child->user,(unsigned long long)child->system,child->status,child->runnable,child->thread_state,child->flags,*observations);
    }else {
        require(strcmp(child->path,gpg)==0,"only exact Git fork-copy or provisioned GPG descendant image");
        *phase=1;
    }
}
struct exec_fence {
    int phase,fork_known,gpg_known;
    unsigned preexec_observations;
    uint64_t preexec_peak_cpu;
    audit_token_t fork_token,gpg_token;
};
static struct image stopped_signature_child(pid_t leader,pid_t pid,uint64_t birth,
    const char *git,const char *gpg,struct exec_fence *fence) {
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000ULL;struct image current;
    uint64_t cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    for (;;) {
        require(control_observe(pid,&current)&&current.birth==birth&&current.ppid==leader&&current.pgid==leader,
            "signature child exact birth and ownership during exec");
        require(proc_pidpath(pid,current.path,sizeof(current.path))>0,"bounded control exec image observation");
        require(current.threads==1&&current.runnable<=1,"single-threaded child through exec fence");
        child_image_phase(&current,git,gpg,&fence->phase,&fence->preexec_observations,&fence->preexec_peak_cpu);
        if(fence->phase==0&&!fence->fork_known) {
            fence->fork_token=control_token(pid,birth);fence->fork_known=1;
            fprintf(stderr,"git_fork_stop_token pid=%u version=%u\n",fence->fork_token.val[5],fence->fork_token.val[7]);
            require(proc_signal_with_audittoken(&fence->fork_token,SIGSTOP)==0,"one fork-task STOP receipt");
        }
        if(fence->phase==1&&!fence->gpg_known) {
            require(fence->fork_known,"observed fork audit token before GPG exec");
            fence->gpg_token=control_token(pid,birth);
            require(fence->gpg_token.val[5]==(unsigned)pid&&fence->gpg_token.val[7]!=fence->fork_token.val[7],"new kernel PID-version on exact approved exec");
            struct image exact;require(control_observe(pid,&exact)&&exact.birth==birth&&proc_pidpath(pid,exact.path,sizeof(exact.path))>0&&strcmp(exact.path,gpg)==0,"new token bound to exact GPG image");
            // This recipient is the newly observed exec-task incarnation. The
            // old task's successful STOP receipt never certifies this task.
            require(proc_signal_with_audittoken(&fence->gpg_token,SIGSTOP)==0,"one fresh STOP for newly known GPG task");
            fence->gpg_known=1;
            fprintf(stderr,"git_exec_stop_token pid=%u old_version=%u new_version=%u\n",fence->gpg_token.val[5],fence->fork_token.val[7],fence->gpg_token.val[7]);
        }
        if(current.status==4&&current.runnable==0) {
            if(fence->phase==1) {
                audit_token_t actual=control_token(pid,birth);
                require(memcmp(&actual,&fence->gpg_token,sizeof(actual))==0,"stable current GPG token at actual stop acknowledgement");
            }
            return current;
        }
        require(clock_ns(CLOCK_MONOTONIC)<until,"bounded exec-task actual stop acknowledgement");
        require(clock_ns(CLOCK_THREAD_CPUTIME_ID)-cpu_start<=1000000,"ordinary exec-task control observation <=1ms CPU");
    }
}
static int git_capability(char **argv) {
    const char *git = argv[2], *gpg = argv[3], *cwd = argv[4], *home = argv[5],
        *commit = argv[6], *tmp = argv[7], *trace = argv[8], *output = argv[9];
    require(chdir(cwd) == 0, "explicit repository cwd");
    char gp[4096], go[4096], gh[4096], tm[4096], tr[4096], hm[4096];
    snprintf(gp,sizeof(gp),"gpg.program=%s",gpg);snprintf(go,sizeof(go),"gpg.openpgp.program=%s",gpg);
    snprintf(gh,sizeof(gh),"GNUPGHOME=%s",home);snprintf(hm,sizeof(hm),"HOME=%s",home);
    snprintf(tm,sizeof(tm),"TMPDIR=%s",tmp);snprintf(tr,sizeof(tr),"GIT_TRACE2_EVENT=%s",trace);
    char *args[]={(char *)git,"--no-pager","--no-optional-locks","-c","core.fsmonitor=false",
        "-c","core.hooksPath=/dev/null","-c","credential.helper=","-c","protocol.allow=never",
        "-c","gpg.format=openpgp","-c",gp,"-c",go,"log","-1","--format=%G?%x00%GF%x00%GP",(char *)commit,NULL};
    char *env[]={"PATH=/usr/bin:/bin:/opt/homebrew/bin","LANG=C","LC_ALL=C",hm,gh,tm,tr,
        "GIT_CONFIG_NOSYSTEM=1","GIT_CONFIG_GLOBAL=/dev/null","GIT_TERMINAL_PROMPT=0",
        "GIT_ASKPASS=/usr/bin/false","SSH_ASKPASS=/usr/bin/false",NULL};
    int out=open(output,O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC,0600);require(out>=0,"private Git output");
    int old=dup(1);require(old>=0 && dup2(out,1)>=0,"Git output binding");close(out);
    uint64_t create_cpu=0,create_wall=0;pid_t pid=suspended(git,args,env,1,&create_cpu,&create_wall);
    require(dup2(old,1)>=0,"restore probe output");close(old);
    struct image leader, child;require(observe(pid,&leader)&&leader.pgid==pid&&leader.status==4
        &&strcmp(leader.path,git)==0,"suspended exact Git image");
    leader=confirmed_stop(pid,leader.birth,NULL,1);
    audit_token_t leader_token=control_token(pid,leader.birth);
    uint64_t leader_birth=leader.birth,leader_peak_cpu=leader.cpu,child_peak_cpu=0,peak_rss=leader.rss,waited_peak=leader.waited;
    uint64_t child_birth=0,child_start_absolute=0,late_kernel_cpu=0,late_kernel_debt=0;
    pid_t child_pid=0;unsigned samples=0,max_members=1,confirmed_stops=0;struct exec_fence fence;memset(&fence,0,sizeof(fence));
    uint64_t prior_total=leader_peak_cpu,controller_at=clock_ns(CLOCK_THREAD_CPUTIME_ID),max_grant_charge=0,max_ordinary_cpu=0,shared_paid_debt=0;
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+5000000000ULL;
    while (!exited(pid)) {
        uint64_t control_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        uint64_t resume_at=clock_ns(CLOCK_MONOTONIC);
        require(kill(-pid,SIGCONT)==0,"Git group resume");
        while(clock_ns(CLOCK_MONOTONIC)-resume_at<50000){}
        kill(-pid,SIGSTOP);
        if (exited(pid)) break;
        leader=control_stop_ack(pid,leader_birth);
        uint64_t all_stopped_at=leader.observed_at;
        int members[3]={0};int count=proc_listchildpids(pid,members,sizeof(members));
        require(count>=0 && count<=1,"Git group bound two");
        int group[3]={0};int group_count=proc_listpgrppids(pid,group,sizeof(group));
        require(group_count>=1&&group_count<=2,"actual private PGID member bound");
        for(int i=0;i<group_count;i++) require(group[i]==pid||(count==1&&group[i]==members[0]),"only leader and direct GPG in exact PGID");
        // The original STOP may predate fork. Stop the freshly observed exact
        // group before spending any time on the new member's image/audit reads.
        if(count==1&&child_pid==0)require(kill(-pid,SIGSTOP)==0,"one immediate fresh-roster pinned-group STOP");
        if(count==1&&control_observe(members[0],&child)) {
            require(child.ppid==pid&&child.pgid==pid&&child.threads==1&&child.runnable<=1,"actual control child ownership");
            require(!child_pid||(child_pid==child.pid&&child_birth==child.birth),"stable control child generation");
            child=stopped_signature_child(pid,child.pid,child.birth,git,gpg,&fence);
            if(child.observed_at>all_stopped_at)all_stopped_at=child.observed_at;
        }else if(count==1) {
            struct rusage_info_v1 dead;
            require(members[0]==child_pid&&child_start_absolute>0&&proc_pid_rusage(child_pid,RUSAGE_INFO_V1,(rusage_info_t *)&dead)==0
                &&dead.ri_proc_start_abstime==child_start_absolute&&dead.ri_proc_exit_abstime>0,"exact control child closed before stop");
            all_stopped_at=clock_ns(CLOCK_MONOTONIC);
        }
        uint64_t active_end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        int child_live=0;
        if (observe(pid,&leader)) {
            require(leader.birth==leader_birth&&leader.pgid==pid,"Git leader identity retained");
            audit_token_t current_leader=control_token(pid,leader_birth);
            require(memcmp(&current_leader,&leader_token,sizeof(leader_token))==0&&strcmp(leader.path,git)==0,"leader cannot exec or change task incarnation");
            require(leader.threads==1&&leader.runnable<=1,"single-threaded actual Git");
            if (leader.cpu>leader_peak_cpu) leader_peak_cpu=leader.cpu;
            if(leader.waited>waited_peak)waited_peak=leader.waited;
            uint64_t rss=leader.rss;
            if (count==1&&observe(members[0],&child)) {
                require(child.ppid==pid&&child.pgid==pid,"GPG exact direct descendant");
                require(child.threads==1&&child.runnable<=1,"single-threaded actual GPG");
                require(!child_pid || (child_pid==child.pid&&child_birth==child.birth),"one stable Git-to-GPG identity");
                require(child.status==4&&child.runnable==0,"actual stopped child retained through ordinary corroboration");
                int grandchildren[2]={0};require(proc_listchildpids(child.pid,grandchildren,sizeof(grandchildren))==0,"GPG cannot add unobserved descendants");
                child_pid=child.pid;child_birth=child.birth;child_start_absolute=child.start_absolute;max_members=2;child_live=1;
                if (child.cpu>child_peak_cpu) child_peak_cpu=child.cpu;
                rss+=child.rss;
            } else if(count==1) {
                struct rusage_info_v1 dead;
                require(members[0]==child_pid&&child_start_absolute>0&&proc_pid_rusage(child_pid,RUSAGE_INFO_V1,(rusage_info_t *)&dead)==0
                    &&dead.ri_proc_start_abstime==child_start_absolute&&dead.ri_proc_exit_abstime>0,"same observed GPG birth exited before stop");
            }
            if (rss>peak_rss) peak_rss=rss;
        }
        uint64_t stopped_leader_cpu=leader.cpu,stopped_child_cpu=child_live?child.cpu:0;
        require(all_stopped_at>=resume_at,"ordered actual all-member stop acknowledgement");
        uint64_t conservative_charge=2*(all_stopped_at-resume_at);
        samples++;require(samples<=512&&clock_ns(CLOCK_MONOTONIC)<until,"bounded Git profile completion");
        uint64_t pre_rest_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);usleep(9000);
        struct image held;
        int held_ok=observe(pid,&held);
        if(!held_ok||held.birth!=leader_birth||held.status!=4||held.threads!=1||held.runnable!=0||(held.thread_state!=TH_STATE_STOPPED&&held.thread_state!=TH_STATE_WAITING)||held.user!=leader.user||held.cpu!=stopped_leader_cpu)
            fprintf(stderr,"git_rest before_cpu_ns=%llu before_user_ns=%llu before_system_ns=%llu before_raw=%llu after_ok=%d after_cpu_ns=%llu after_user_ns=%llu after_system_ns=%llu after_raw=%llu before_ri_ns=%llu after_ri_ns=%llu status=%u threads=%u runnable=%u sample=%u before_thread_state=%d after_thread_state=%d before_thread_user_raw=%llu after_thread_user_raw=%llu before_thread_system_raw=%llu after_thread_system_raw=%llu\n",
                (unsigned long long)stopped_leader_cpu,(unsigned long long)leader.user,(unsigned long long)leader.system,(unsigned long long)leader.raw_cpu,held_ok,
                (unsigned long long)held.cpu,(unsigned long long)held.user,(unsigned long long)held.system,(unsigned long long)held.raw_cpu,
                (unsigned long long)leader.rusage_cpu,(unsigned long long)held.rusage_cpu,held.status,held.threads,held.runnable,samples,leader.thread_state,held.thread_state,
                (unsigned long long)leader.thread_user_raw,(unsigned long long)held.thread_user_raw,(unsigned long long)leader.thread_system_raw,(unsigned long long)held.thread_system_raw);
        require(held_ok&&held.birth==leader_birth&&held.status==4&&held.threads==1&&held.runnable==0&&(held.thread_state==TH_STATE_STOPPED||held.thread_state==TH_STATE_WAITING)&&held.user==leader.user&&held.cpu>=stopped_leader_cpu,
            "actual Git user work remains stopped through rest");
        uint64_t late=held.cpu-stopped_leader_cpu;
        if(held.cpu>leader_peak_cpu)leader_peak_cpu=held.cpu;
        if(held.waited>waited_peak)waited_peak=held.waited;
        if(child_live) {
            require(observe(members[0],&held)&&held.birth==child_birth&&held.status==4&&held.threads==1&&held.runnable==0&&(held.thread_state==TH_STATE_STOPPED||held.thread_state==TH_STATE_WAITING)&&held.user==child.user&&held.cpu>=stopped_child_cpu,
                "actual GPG user work remains stopped through rest");
            late+=held.cpu-stopped_child_cpu;if(held.cpu>child_peak_cpu)child_peak_cpu=held.cpu;
        }
        require(late<=1000000&&late_kernel_cpu<=UINT64_MAX-late&&late_kernel_debt<=UINT64_MAX-9*late,"late kernel reconciliation bounds");
        late_kernel_cpu+=late;late_kernel_debt+=9*late;
        uint64_t total=leader_peak_cpu+(child_peak_cpu>waited_peak?child_peak_cpu:waited_peak);
        uint64_t controller_end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        require(total>=prior_total&&controller_end>=controller_at&&controller_end>=pre_rest_cpu,"monotonic helper/controller charge");
        uint64_t actual_helper_delta=total-prior_total;
        uint64_t helper_charge=actual_helper_delta>conservative_charge?actual_helper_delta:conservative_charge;
        uint64_t active_controller_cpu=active_end-control_start;
        uint64_t ordinary_controller_cpu=controller_end-active_end+control_start-controller_at;
        if(ordinary_controller_cpu>max_ordinary_cpu)max_ordinary_cpu=ordinary_controller_cpu;
        require(ordinary_controller_cpu<=1000000,"separately paid ordinary stopped-corroboration unit <=1ms");
        uint64_t charge=helper_charge+active_controller_cpu;
        if(charge>max_grant_charge)max_grant_charge=charge;
        if(charge>1000000)fprintf(stderr,"git_grant charge_ns=%llu helper_delta_ns=%llu controller_ns=%llu late_kernel_ns=%llu sample=%u\n",
            (unsigned long long)charge,(unsigned long long)(total-prior_total),(unsigned long long)active_controller_cpu,(unsigned long long)late,samples);
        require(charge<=1000000,"full original helper grant plus controller and late reconciliation <=1ms");
        uint64_t post=late+ordinary_controller_cpu;require(post<=1000000&&shared_paid_debt<=UINT64_MAX-9*post,"fresh shared reconciliation debt bounds");
        shared_paid_debt+=9*post;prior_total=total;controller_at=controller_end;
        // This is fresh debt after the observation. Earlier stopped rest is
        // never banked as credit against a counter discovered later.
        if(post)usleep((useconds_t)((9*post+999)/1000));
        confirmed_stops++;
    }
    struct rusage_info_v1 final_usage;
    if(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&final_usage)==0) {
        uint64_t waited=absolute_ns(final_usage.ri_child_user_time+final_usage.ri_child_system_time);
        if(waited>waited_peak)waited_peak=waited;
    }
    struct rusage usage;joined_group(pid,&usage);
    uint64_t final_total=ru_cpu(&usage);if(final_total<prior_total)final_total=prior_total;
    uint64_t closing_charge=final_total-prior_total+clock_ns(CLOCK_THREAD_CPUTIME_ID)-controller_at;
    if(closing_charge>max_grant_charge)max_grant_charge=closing_charge;
    require(closing_charge<=1000000,"final reaped CPU/controller reconciliation <=1ms");
    usleep((useconds_t)((9*closing_charge+999)/1000));
    require(child_pid>0&&child_birth>0&&child_peak_cpu>0&&max_members==2&&fence.fork_known&&fence.gpg_known&&fence.phase==1&&fence.preexec_observations>0,"observed finite actual Git-fork-to-GPG transition and GPG CPU/token before wait");
    require(waited_peak>=child_peak_cpu&&ru_cpu(&usage)>=child_peak_cpu+leader_peak_cpu,"real Git waited descendant rollup without loss");
    printf("{\"git_pid\":%d,\"gpg_pid\":%d,\"gpg_birth\":%llu,\"max_members\":%u,\"max_threads_per_process\":1,\"samples\":%u,\"confirmed_stopped_rests\":%u,\"git_fork_before_gpg_exec\":true,\"old_fork_pidversion\":%u,\"new_gpg_pidversion\":%u,\"exec_task_stop_applied\":true,\"preexec_observations\":%u,\"preexec_peak_cpu_ns\":%llu,\"max_full_grant_charge_ns\":%llu,\"max_stopped_corroboration_cpu_ns\":%llu,\"shared_reconciliation_paid_debt_ns\":%llu,\"stopped_rest_user_cpu_delta_ns\":0,\"late_kernel_cpu_ns\":%llu,\"late_kernel_paid_debt_ns\":%llu,\"exact_pgid_membership\":true,\"sampled_git_cpu_ns\":%llu,\"sampled_gpg_cpu_ns\":%llu,\"rolled_gpg_cpu_ns\":%llu,\"git_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true}\n",pid,child_pid,
        (unsigned long long)child_birth,max_members,samples,confirmed_stops,fence.fork_token.val[7],fence.gpg_token.val[7],fence.preexec_observations,(unsigned long long)fence.preexec_peak_cpu,(unsigned long long)max_grant_charge,(unsigned long long)max_ordinary_cpu,(unsigned long long)shared_paid_debt,(unsigned long long)late_kernel_cpu,(unsigned long long)late_kernel_debt,(unsigned long long)leader_peak_cpu,
        (unsigned long long)child_peak_cpu,(unsigned long long)waited_peak,(unsigned long long)ru_cpu(&usage),(unsigned long long)peak_rss,
        (unsigned long long)create_cpu,(unsigned long long)create_wall);
    return 0;
}
int main(int argc,char **argv) {
    require(atexit(dispose_owned)==0,"owned cleanup registration");
    require(argc>=2,"mode required");
    if (!strcmp(argv[1],"cpu-child")) {require(argc==3,"child argc");cpu_child(argv[2]);}
    if (!strcmp(argv[1],"leader")) {require(argc==6,"leader argc");leader(argv[0],argv[2],argv[3],argv[4],argv[5]);}
    if (!strcmp(argv[1],"native")) {require(argc==3,"native argc");return native_capability(argv[0],argv[2]);}
    if (!strcmp(argv[1],"git")) {require(argc==10,"Git argc");return git_capability(argv);}
    require(0,"unknown mode");return 81;
}
