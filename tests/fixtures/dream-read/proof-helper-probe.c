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
    uint64_t birth, cpu, rss, waited;
    uint32_t status;
    char path[PROC_PIDPATHINFO_MAXSIZE];
};
static int observe(pid_t pid, struct image *out) {
    struct proc_bsdinfo b; struct proc_taskinfo t;struct rusage_info_v1 usage;
    memset(out, 0, sizeof(*out));
    if (proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, &b, sizeof(b)) != (int)sizeof(b)
        || proc_pidinfo(pid, PROC_PIDTASKINFO, 0, &t, sizeof(t)) != (int)sizeof(t)
        || proc_pidpath(pid, out->path, sizeof(out->path)) <= 0
        ||proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)!=0) return 0;
    out->pid = pid; out->ppid = b.pbi_ppid; out->pgid = b.pbi_pgid;
    out->birth = (uint64_t)b.pbi_start_tvsec * 1000000ULL + b.pbi_start_tvusec;
    out->status = b.pbi_status;
    out->cpu = t.pti_total_user + t.pti_total_system;
    out->rss = t.pti_resident_size;
    out->waited=usage.ri_child_user_time+usage.ri_child_system_time;
    return out->birth > 0 && out->pgid > 0;
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
    // Retain the actual returned identity before any verdict can reject its effect.
    if (result != 0 || pid <= 0) { errno = result; require(0, "suspended creation"); }
    if (own_group) retained_group=pid;else retained_child=pid;
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
    usleep(20000);
    require(observe(pid, &after) && after.birth == before.birth && after.cpu == before.cpu
        && access(first, F_OK) == -1 && errno == ENOENT, "no pre-admission target CPU");
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
    require(proc_listchildpids(pid, members, sizeof(members)) == (int)sizeof(pid_t)
        && members[0] == child_pid, "bounded exact child relationship");
    uint64_t peak_rss = after.rss + child.rss;
    require(kill(-pid, SIGCONT) == 0, "group resume");
    uint64_t child_cpu = 0;
    until = clock_ns(CLOCK_MONOTONIC) + 2000000000ULL;
    while (!exited(pid)) {
        struct image c, p;
        if (observe(child_pid, &c)) { if (c.cpu > child_cpu) child_cpu = c.cpu; }
        if (observe(pid, &p) && observe(child_pid, &c) && p.rss + c.rss > peak_rss) peak_rss = p.rss + c.rss;
        require(clock_ns(CLOCK_MONOTONIC) < until, "native completion"); usleep(100);
    }
    struct rusage usage; joined_group(pid, &usage);
    FILE *f = fopen(report, "r"); uint64_t waited_cpu = 0; pid_t reported = 0;
    require(f && fscanf(f, "%d\n%llu", &reported, (unsigned long long *)&waited_cpu) == 2
        && reported == child_pid, "actual descendant wait4 report"); fclose(f);
    require(waited_cpu >= 12000000 && child_cpu > 0 && ru_cpu(&usage) >= waited_cpu,
        "leader wait4 descendant CPU rollup");
    printf("{\"created_suspended\":true,\"cloexec_default\":true,\"leader\":%d,\"child\":%d,\"pgid\":%d,\"leader_birth\":%llu,\"child_birth\":%llu,\"members\":2,\"sampled_child_cpu_ns\":%llu,\"waited_child_cpu_ns\":%llu,\"leader_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true}\n",
        pid, child_pid, pid, (unsigned long long)before.birth, (unsigned long long)child.birth,
        (unsigned long long)child_cpu, (unsigned long long)waited_cpu, (unsigned long long)ru_cpu(&usage),
        (unsigned long long)peak_rss, (unsigned long long)create_cpu, (unsigned long long)create_wall);
    return 0;
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
    uint64_t leader_peak_cpu=leader.cpu,child_peak_cpu=0,peak_rss=leader.rss,waited_peak=leader.waited;
    uint64_t child_birth=0;pid_t child_pid=0;unsigned samples=0,max_members=1;
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+5000000000ULL;
    while (!exited(pid)) {
        require(kill(-pid,SIGCONT)==0,"Git group resume");usleep(250);kill(-pid,SIGSTOP);
        if (exited(pid)) break;
        int members[3]={0};int count=proc_listchildpids(pid,members,sizeof(members));
        require(count>=0 && count<=(int)sizeof(pid_t),"Git group bound two");
        if (observe(pid,&leader)) {
            require(leader.birth>0&&leader.pgid==pid,"Git leader identity retained");
            if (leader.cpu>leader_peak_cpu) leader_peak_cpu=leader.cpu;
            if(leader.waited>waited_peak)waited_peak=leader.waited;
            uint64_t rss=leader.rss;
            if (count==(int)sizeof(pid_t)&&observe(members[0],&child)) {
                require(child.ppid==pid&&child.pgid==pid,"GPG exact direct descendant");
                require(strcmp(child.path,gpg)==0,"only provisioned GPG descendant image");
                if (strcmp(child.path,gpg)==0) {
                    require(!child_pid || (child_pid==child.pid&&child_birth==child.birth),"one stable GPG identity");
                    child_pid=child.pid;child_birth=child.birth;max_members=2;
                    if (child.cpu>child_peak_cpu) child_peak_cpu=child.cpu;
                    rss+=child.rss;
                }
            }
            if (rss>peak_rss) peak_rss=rss;
        }
        samples++;require(samples<=512&&clock_ns(CLOCK_MONOTONIC)<until,"bounded Git profile completion");usleep(9000);
    }
    struct rusage_info_v1 final_usage;
    if(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&final_usage)==0) {
        uint64_t waited=final_usage.ri_child_user_time+final_usage.ri_child_system_time;
        if(waited>waited_peak)waited_peak=waited;
    }
    struct rusage usage;joined_group(pid,&usage);
    require(child_pid>0&&child_birth>0&&child_peak_cpu>0&&max_members==2,"observed real GPG CPU before wait");
    require(waited_peak>=child_peak_cpu&&ru_cpu(&usage)>=child_peak_cpu+leader_peak_cpu,"real Git waited descendant rollup without loss");
    printf("{\"git_pid\":%d,\"gpg_pid\":%d,\"gpg_birth\":%llu,\"max_members\":%u,\"samples\":%u,\"sampled_git_cpu_ns\":%llu,\"sampled_gpg_cpu_ns\":%llu,\"rolled_gpg_cpu_ns\":%llu,\"git_wait4_cpu_ns\":%llu,\"group_peak_rss_bytes\":%llu,\"create_cpu_ns\":%llu,\"create_wall_ns\":%llu,\"reaped\":true}\n",pid,child_pid,
        (unsigned long long)child_birth,max_members,samples,(unsigned long long)leader_peak_cpu,
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
