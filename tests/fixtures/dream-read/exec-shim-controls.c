// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Private process driver: no signing identity, builder or runtime approval.
#define _DARWIN_C_SOURCE 1
#define DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS 1
#define main canonical_probe_main
#include "proof-helper-probe.c"
#undef main
static struct dream_shim_config test_config;
static void put(const char *s,uint32_t *offset) {
    size_t n=strlen(s)+1;require(n<=4097&&test_config.used+n<=sizeof(test_config.bytes),"test config bound");
    *offset=test_config.used;memcpy(test_config.bytes+test_config.used,s,n);test_config.used+=(uint32_t)n;
}
static int escape_control(const char *mode,const char *ledger) {
    int ready[2],release[2];require(!pipe(ready)&&!pipe(release),"escape control pipes");
    pid_t child=fork();require(child>=0,"escape control owned child");
    if(!child){close(ready[0]);close(release[1]);if(setsid()<0)_exit(82);char byte='r';if(write(ready[1],&byte,1)!=1)_exit(83);if(read(release[0],&byte,1)!=1)_exit(84);_exit(0);}
    close(ready[1]);close(release[0]);char byte;require(read(ready[0],&byte,1)==1,"escape control ready");close(ready[0]);
    struct proc_bsdinfo b;require(proc_pidinfo(child,PROC_PIDTBSDINFO,0,&b,sizeof(b))==(int)sizeof(b),"escape control real birth");
    uint64_t birth=(uint64_t)b.pbi_start_tvsec*1000000+b.pbi_start_tvusec;
    struct private_engine_result result={0};struct engine_owner owner={.shim_active=1,.direct_custody=1,.ticket={.pid=child,.pgid=getpgrp()},.result=&result};
    owner.ledger=open(ledger,O_RDWR|O_CREAT|O_EXCL|O_CLOEXEC,0600);require(owner.ledger>=0,"escape control ledger");
    owner.retired_count=1;owner.retired[0]=(struct engine_incarnation){.pid=child,.birth=birth,.current=1};
    require(known_control_token(child,birth,&owner.retired[0].token),"escape control original token");
    int positive=!strcmp(mode,"escaped-live")||!strcmp(mode,"alive-after-reap");
    if(!positive){require(write(release[1],"x",1)==1,"escape control exit release");int status;require(waitpid(child,&status,0)==child&&WIFEXITED(status)&&WEXITSTATUS(status)==0,"escape control actual child reap");}
    if(!strcmp(mode,"cleanup-eio")||!strcmp(mode,"cleanup-esrch")) {
        source_known_observation_selected=ENGINE_OBS_ESCAPED_CLEANUP_BSD;
        source_known_observation_forced_domain=ENGINE_OBSERVATION_UNIX;
        source_known_observation_forced_code=!strcmp(mode,"cleanup-eio")?EIO:ESRCH;
    }
    if(strcmp(mode,"alive-after-reap"))engine_cleanup_escaped(&owner);
    int cleanup_failed=owner.failed;
    if(positive){
        if(!strcmp(mode,"alive-after-reap")){owner.direct_reaped=owner.wait_observed=1;owner.direct_custody=0;require(!engine_settle_proven(&owner)&&!owner.escaped_kill_attempts,"positive postreap hit refuses without signal");}
        require(kill(child,SIGKILL)==0||errno==ESRCH,"escape control exact owned cleanup");int status;require(waitpid(child,&status,0)==child,"escape control positive child reap");
        require(owner.failed&&!strcmp(result.error_code,"coverage_unknown"),"positive same-birth escaped live process refuses");
    }else{
        require(!cleanup_failed&&!owner.obs_terminal_takeovers&&!owner.escaped_kill_attempts&&!private_failed,"postreap escape read settles without terminal-proof route");
        owner.direct_reaped=owner.wait_observed=1;owner.direct_custody=0;
        if(!strcmp(mode,"census-eio")||!strcmp(mode,"census-esrch")) {
            source_known_observation_selected=ENGINE_OBS_GROUP_CENSUS;source_known_observation_forced_domain=ENGINE_OBSERVATION_UNIX;
            source_known_observation_forced_code=!strcmp(mode,"census-eio")?EIO:ESRCH;
        }
        require(engine_settle_proven(&owner)&&result.group_gone&&!owner.failed&&!private_failed,"best-effort postreap census settles without refusal");
    }
    close(release[1]);require(close(owner.ledger)==0,"escape control ledger close");
    printf("{\"escape_control\":\"%s\",\"refused\":%s,\"settled\":%s,\"terminal_takeovers\":%llu,\"kill_attempts\":%u,\"runtime_approved\":false}\n",mode,owner.failed?"true":"false",result.group_gone?"true":"false",(unsigned long long)owner.obs_terminal_takeovers,owner.escaped_kill_attempts);
    return 0;
}
int main(int argc,char **argv) {
    if(argc==4&&!strcmp(argv[1],"escape-control"))return escape_control(argv[2],argv[3]);
    require(argc>=12,"driver shim git gpg home scratch sha len cwd stdout form args...");
    const char *shim=argv[1],*git=argv[2],*gpg=argv[3],*home=argv[4],*scratch=argv[5];
    unsigned form=(unsigned)strtoul(argv[10],NULL,10);require(form<15,"driver form");
    test_config.magic=DREAM_SHIM_MAGIC;test_config.form=form;test_config.signature_len=strtoull(argv[7],NULL,10);
    require(strlen(argv[6])==64,"driver signature digest");strcpy(test_config.signature_sha,argv[6]);
    const char *exec_path=getenv("DREAM_SHIM_TEST_EXEC_PATH");
    if(exec_path)strcpy(test_config.git_exec_path,exec_path);else {strcpy(test_config.git_exec_path,git);char *last=strrchr(test_config.git_exec_path,'/');require(last!=NULL,"driver git path");*last=0;last=strrchr(test_config.git_exec_path,'/');require(last!=NULL,"driver git prefix");strcpy(last,"/libexec/git-core");}
    strcpy(test_config.git,git);strcpy(test_config.gpg,gpg);strcpy(test_config.shim,shim);strcpy(test_config.home,home);strcpy(test_config.scratch,scratch);
    require(lstat(scratch,&test_config.scratch_stat)==0,"driver scratch identity");
    char *args[65]={0},*env[18]={0};unsigned count=(unsigned)argc-10;require(count<=64,"driver argc");
    args[0]=(char *)shim;put(shim,&test_config.argv[0]);test_config.argc=count;
    for(unsigned i=1;i<count;i++){args[i]=argv[10+i];put(args[i],&test_config.argv[i]);}
    char dynamic[3][4107];snprintf(dynamic[0],sizeof(dynamic[0]),"HOME=%s",home);snprintf(dynamic[1],sizeof(dynamic[1]),"GNUPGHOME=%s",home);snprintf(dynamic[2],sizeof(dynamic[2]),"TMPDIR=%s",scratch);
    const char *values[]={"GIT_CONFIG_NOSYSTEM=1","GIT_CONFIG_GLOBAL=/dev/null","GIT_TERMINAL_PROMPT=0","GIT_ASKPASS=/usr/bin/false","SSH_ASKPASS=/usr/bin/false","LANG=C","LC_ALL=C","PATH=/usr/bin:/bin",dynamic[0],dynamic[1],dynamic[2]};
    for(unsigned i=0;i<11;i++){put(values[i],&test_config.env[i]);env[i]=(char *)values[i];}test_config.envc=11;
    if(getenv("DREAM_SHIM_TEST_BAD_ENV"))env[5]="LANG=fr";
    char derived_parameters[32768],derived_exec[4120],derived_path[4140];
    if(count==6&&!strcmp(args[1],"--keyid-format=long")) {
        // Test paths contain no quote/exclamation. The shim itself also covers
        // Git's quoting for those bytes; exact values are witnessed separately.
        require(!strchr(shim,'\'')&&!strchr(shim,'!'),"driver simple quoted fixture path");
        int n=snprintf(derived_parameters,sizeof(derived_parameters),"GIT_CONFIG_PARAMETERS='core.fsmonitor'='false' 'core.hooksPath'='/dev/null' 'credential.helper'='' 'protocol.allow'='never' 'gpg.format'='openpgp' 'gpg.program'='%s' 'gpg.openpgp.program'='%s'",shim,shim);require(n>0&&(size_t)n<sizeof(derived_parameters),"driver parameters");
        snprintf(derived_exec,sizeof(derived_exec),"GIT_EXEC_PATH=%s",test_config.git_exec_path);
        snprintf(derived_path,sizeof(derived_path),"PATH=%s:/usr/bin:/bin",test_config.git_exec_path);
        env[7]=derived_path;env[11]=derived_parameters;env[12]=derived_exec;env[13]="GIT_OPTIONAL_LOCKS=0";env[14]="GIT_PAGER=cat";env[15]="GIT_PREFIX=";
    }

    char config_name[4097];snprintf(config_name,sizeof(config_name),"%s/../config-XXXXXX",home);
    int writable=mkstemp(config_name);require(writable>=0&&fchmod(writable,0600)==0,"driver exclusive config");
    require(write(writable,&test_config,sizeof(test_config))==(ssize_t)sizeof(test_config)&&close(writable)==0,"driver config bytes");
    int cfg=open(config_name,O_RDONLY|O_NOFOLLOW|O_CLOEXEC);require(cfg>=0&&unlink(config_name)==0,"driver readonly unlinked config");
    int config_copy=fcntl(cfg,F_DUPFD_CLOEXEC,20);close(cfg);require(config_copy>=0,"driver config pin");
    int sockets[2];require(socketpair(AF_UNIX,SOCK_DGRAM,0,sockets)==0,"driver log sockets");
    int client=fcntl(sockets[1],F_DUPFD_CLOEXEC,20);close(sockets[1]);require(client>=0,"driver log pin");
    int out=open(argv[9],O_CREAT|O_EXCL|O_WRONLY|O_CLOEXEC,0600);require(out>=0,"driver output");
    posix_spawn_file_actions_t actions;posix_spawnattr_t attr;require(!posix_spawn_file_actions_init(&actions)&&!posix_spawnattr_init(&attr),"driver spawn preparation");
    require(!posix_spawnattr_setflags(&attr,POSIX_SPAWN_SETPGROUP|POSIX_SPAWN_START_SUSPENDED|POSIX_SPAWN_CLOEXEC_DEFAULT)
        &&!posix_spawnattr_setpgroup(&attr,0)&&(getenv("DREAM_SHIM_TEST_BAD_QOS")||!posix_spawnattr_set_qos_class_np(&attr,QOS_CLASS_BACKGROUND)),"driver background birth");
    require(!posix_spawn_file_actions_addchdir(&actions,argv[8])&&!posix_spawn_file_actions_addopen(&actions,0,getenv("DREAM_SHIM_TEST_INPUT")?getenv("DREAM_SHIM_TEST_INPUT"):"/dev/null",O_RDONLY,0)
        &&!posix_spawn_file_actions_adddup2(&actions,out,1)&&!posix_spawn_file_actions_adddup2(&actions,2,2)
        &&!posix_spawn_file_actions_adddup2(&actions,config_copy,3)&&!posix_spawn_file_actions_adddup2(&actions,client,4),"driver owned FD actions");
    pid_t pid=0;require(!posix_spawn(&pid,shim,&actions,&attr,args,env)&&pid>0,"driver actual shim birth");retained_group=pid;
    posix_spawn_file_actions_destroy(&actions);posix_spawnattr_destroy(&attr);close(config_copy);close(client);close(out);
    struct private_input outer={0};outer.deadline=clock_ns(CLOCK_MONOTONIC)+30000000000ULL;
    struct private_engine_context context={.outer=&outer};struct private_engine_result result={0};
    struct engine_owner observer={.shim_active=1,.ticket={.pid=pid,.pgid=pid},.direct_custody=1,.command=form,.context=&context,.result=&result};
    if(getenv("DREAM_SHIM_TEST_FAILED_LIVE_READ")) {
        source_known_observation_selected=ENGINE_OBS_LIVE_RUSAGE;
        source_known_observation_forced_domain=ENGINE_OBSERVATION_UNIX;source_known_observation_forced_code=EIO;
        require(engine_shim_sample_member(&observer,0,pid)&&observer.live_read_failures==1
            &&source_known_observation_hits==1&&!observer.sample_known&&!observer.members[0].state.start_absolute&&!observer.failed&&!private_failed,"supplied failed RI read is ignored on original live process");
        require(engine_sample_gap(&observer,UINT64_MAX)&&!observer.failed,"sample gap is not a gate");
    }
    int events=-1;if(getenv("DREAM_SHIM_TEST_FORK_BACKSTOP")) {
        events=kqueue();require(events>=0,"driver fork queue");struct kevent change;EV_SET(&change,(uintptr_t)pid,EVFILT_PROC,EV_ADD|EV_ENABLE|EV_CLEAR,NOTE_FORK|NOTE_EXIT,0,NULL);
        require(kevent(events,&change,1,NULL,0,NULL)==0,"driver original leader watch before continuation");
    }
    require(kill(pid,SIGCONT)==0,"driver initial continuation");
    uint64_t live_cpu=0;unsigned live_kill=0;
    if(events>=0)for(;;) {
        struct kevent note;struct timespec wait={0,1000000};int got=kevent(events,NULL,0,&note,1,&wait);require(got>=0,"driver observed fork wake");
        if(!got)continue;
        int admitted=engine_event_note(&observer,&note,clock_ns(CLOCK_MONOTONIC));
        if(observer.leader_fork_events==1) {char ack='a';require(send(sockets[0],&ack,1,0)==1,"driver first fork receipt ACK");}
        if(!admitted){require(observer.group_kill_attempted&&observer.leader_fork_events==2,"observed second fork causes production kill");break;}
        require(!(note.fflags&NOTE_EXIT),"extra-fork control must reach second observed note");
    }
    if(events>=0)close(events);
    if(getenv("DREAM_SHIM_TEST_CPU_BACKSTOP"))for(;;) {
        int done=0;require(known_exited(pid,&done),"driver terminal observation");if(done)break;
        struct rusage_info_v1 raw;if(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&raw)==0) {
            uint64_t sum=raw.ri_user_time+raw.ri_system_time;require(known_absolute_ns(sum,&live_cpu),"driver live CPU conversion");
            observer.members[0].user=raw.ri_user_time;observer.members[0].system=raw.ri_system_time;observer.sample_known=1;
            if(!engine_live_budget(&observer)){require(observer.live_over_cap&&observer.group_kill_attempted,"production successful over-cap read kills group");live_kill=1;break;}
        }
        usleep(1000);
    }
    struct rusage usage;int status;require(wait4(pid,&status,0,&usage)==pid,"driver original wait4");retained_group=0;
    unsigned receipts=0,gs1=0;int held=-1;struct dream_shim_record signature={0};
    for(unsigned i=0;i<3;i++) {
        struct dream_shim_record record;struct iovec io={&record,sizeof(record)};char rights[CMSG_SPACE(sizeof(int))];struct msghdr msg={0};
        msg.msg_iov=&io;msg.msg_iovlen=1;msg.msg_control=rights;msg.msg_controllen=sizeof(rights);
        ssize_t got=recvmsg(sockets[0],&msg,MSG_DONTWAIT);if(got<0){require(errno==EAGAIN||errno==EWOULDBLOCK||errno==ECONNRESET,"driver log empty");break;}
        require(got==(ssize_t)sizeof(record)&&record.magic==DREAM_SHIM_MAGIC&&record.priority==4,"driver checked shim receipt");receipts++;
        if(record.form==15){gs1++;signature=record;struct cmsghdr *c=CMSG_FIRSTHDR(&msg);require(c&&c->cmsg_type==SCM_RIGHTS&&held<0,"driver GS1 descriptor");memcpy(&held,CMSG_DATA(c),sizeof(held));}
    }
    unsigned gs2=0;if(held>=0){
        if(getenv("DREAM_SHIM_TEST_UNLINK")){char path[4097];int length=snprintf(path,sizeof(path),"%s/%s",scratch,signature.name);require(length>0&&(size_t)length<sizeof(path),"driver GS2 path");struct stat named;require(lstat(path,&named)==0&&named.st_dev==signature.signature.st_dev&&named.st_ino==signature.signature.st_ino&&unlink(path)==0,"driver owns test signature cleanup");}
        struct stat st;require(fstat(held,&st)==0,"driver retained signature stat");
        gs2=st.st_dev==signature.signature.st_dev&&st.st_ino==signature.signature.st_ino&&st.st_nlink==0&&st.st_size==signature.signature.st_size;close(held);}
    close(sockets[0]);
    printf("{\"status\":%d,\"receipts\":%u,\"gs1\":%u,\"gs2\":%u,\"wait4_cpu_ns\":%llu,\"rss\":%ld,\"live_cpu_ns\":%llu,\"live_kill\":%u,\"observed_fork_notes\":%llu,\"failed_live_reads_ignored\":%llu,\"terminal_refused\":%s,\"runtime_approved\":false}\n",WIFEXITED(status)?WEXITSTATUS(status):-1,receipts,gs1,gs2,(unsigned long long)ru_cpu(&usage),usage.ru_maxrss,(unsigned long long)live_cpu,live_kill,(unsigned long long)observer.leader_fork_events,(unsigned long long)observer.live_read_failures,ru_cpu(&usage)>250000000?"true":"false");
    return 0;
}
