// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Synthetic source-known API/semantic/terminal controls. No real candidate,
// source authentication, OpenPGP qualification or runtime approval is claimed.
#define main canonical_probe_main
#include "proof-helper-probe.c"
#undef main

static const char payload[]="tree dee19491977e73e485607928be1e1a34625a4ffd\nparent 2ac709c836778e8203ecc9232daa5153a166cc0f\nauthor A U Thor <author@example.com> 1112912653 -0700\ncommitter C O Mitter <committer@example.com> 1112912653 -0700\n\nsecond\n";
static unsigned controls;
static struct private_input outer;
static struct private_basis basis;
static const char input_hash[]="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static struct private_engine_context context={&outer,&basis,input_hash};
static unsigned char capture[8192];
static void reset(void) {
    memset(&private_cost,0,sizeof(private_cost));private_failed=0;private_error="input_invalid";private_detail="fixture unset";
    private_cost.entry=clock_ns(CLOCK_MONOTONIC);private_cost.deadline=private_cost.entry+30000000000ULL;
    private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);engine_cancel_signal=0;
}
static void fixture(int root) {
    memset(&outer,0,sizeof(outer));memset(&basis,0,sizeof(basis));
    strcpy(outer.case_name,"positive");outer.repo.fd=root;outer.prefix.fd=outer.output.fd=-1;
    outer.deadline=clock_ns(CLOCK_MONOTONIC)+30000000000ULL;
    strcpy(basis.fixture.ancestor,"2ac709c836778e8203ecc9232daa5153a166cc0f");
    strcpy(basis.fixture.valid,"cbceced299d68bdfce3b95f182698f280f1da743");
    strcpy(basis.fixture.corrupt,"0cf6ea6f9b0da82362299e1ccad64e4a5d028de3");
    strcpy(basis.fixture.blob,"d00491fd7e5bb6fa28c517a0bb32b8b506539d4d");
    strcpy(basis.fixture.signer,"73D758744BE721698EC54E8713B6F51ECDDE430D");
    basis.fixture.payload.path=(struct private_slice){(const unsigned char *)"\"payload\"",9};
    basis.fixture.payload.len=sizeof(payload)-1;
    strcpy(basis.fixture.payload.sha,"58c4ecb71bef6681cc31d9513ac85bbbd130d4d15438e5c57802a983f6091fb3");
    basis.fixture.valid_signature.len=basis.fixture.corrupt_signature.len=228;
    strcpy(basis.fixture.valid_signature.sha,"d716c2d0807ca1339c774486d55d31768fa9f953df90872f2ceef4cb4568c792");
    strcpy(basis.fixture.corrupt_signature.sha,"cd9777c930d18c7b5b3603ad58a6f63bd64fca702d87bc4e518fca5c1cf0a17b");
    basis.fixture.record=(struct private_slice){(const unsigned char *)"\"second\"",8};
    basis.fixture.origin=(struct private_slice){(const unsigned char *)"\"https://example.invalid/fixture.git\"",37};
    memset(&private_guards,0,sizeof(private_guards));private_guards.outer=&outer;
    private_guards.basis=&basis;private_guards.input_hash=input_hash;private_guards.stage=1;
}
static void output(struct engine_owner *owner,const void *bytes,size_t n) {
    require(n<=sizeof(capture),"source-known output bound");if(n)memcpy(capture,bytes,n);owner->capture=capture;owner->out_len=n;
}
static void semantic_controls(void) {
    struct private_engine_result result;struct engine_owner owner;memset(&owner,0,sizeof(owner));memset(&result,0,sizeof(result));
    owner.context=&context;owner.result=&result;
    char text[256];int n;
    for(unsigned corrupt=0;corrupt<2;corrupt++) {
        strcpy(outer.case_name,corrupt?"corrupt_proof":"positive");
        for(unsigned form=2;form<=3;form++) {
            reset();n=snprintf(text,sizeof(text),"%s\n",basis.fixture.valid);output(&owner,text,(size_t)n);
            require(engine_semantic(&owner,form,0),"both fixed refs retain valid commit in either case");controls++;
            reset();output(&owner,"0000000000000000000000000000000000000000\n",41);
            require(!engine_semantic(&owner,form,0),"wrong exact ref refuses");controls++;
        }
        reset();n=snprintf(text,sizeof(text),"%s\n",corrupt?basis.fixture.corrupt:basis.fixture.valid);output(&owner,text,(size_t)n);
        require(engine_semantic(&owner,8,0),"selected commit is actual fixed last touch");controls++;
    }
    strcpy(outer.case_name,"positive");
    reset();output(&owner,"dee19491977e73e485607928be1e1a34625a4ffd\n",41);
    require(engine_semantic(&owner,10,0),"tree derives from pinned unsigned payload");controls++;
    reset();output(&owner,"0000000000000000000000000000000000000000\n",41);
    require(!engine_semantic(&owner,10,0),"wrong actual tree refuses");controls++;
    reset();output(&owner,"1\n",2);require(engine_semantic(&owner,7,0),"actual record bytes bind Git blob");controls++;
    reset();output(&owner,"2\n",2);require(!engine_semantic(&owner,7,0),"different actual record bytes refuse");controls++;
    reset();output(&owner,NULL,0);require(engine_semantic(&owner,5,0),"actual ancestry status0");
    reset();require(!engine_semantic(&owner,5,1),"nonzero ancestry is not expected cryptographic failure");controls+=2;
    struct private_git_sig_temp sig;memset(&sig,0,sizeof(sig));sig.bound=sig.settled=1;
    owner.git_signature=&sig;owner.verified_descendants=owner.child_created=owner.child_settled=1;
    owner.members[1].exit_status_known=1;owner.members[1].exit_status=0;
    sig.selected=&basis.fixture.valid_signature;
    for(unsigned i=0;i<2;i++) {
        reset();text[0]=i?'U':'G';text[1]=0;memcpy(text+2,basis.fixture.signer,40);text[42]=0;text[43]='\n';output(&owner,text,44);
        require(engine_semantic(&owner,12,0)&&!strcmp(result.observed_proof,"valid"),"G/U plus exact signer accepted");controls++;
    }
    reset();owner.verified_descendants=0;require(!engine_semantic(&owner,12,0),"missing actual verifier refuses valid-looking bytes");controls++;owner.verified_descendants=1;
    strcpy(outer.case_name,"corrupt_proof");sig.selected=&basis.fixture.corrupt_signature;owner.members[1].exit_status=1<<8;
    reset();output(&owner,"B\0\0\n",4);require(engine_semantic(&owner,12,0)&&!strcmp(result.observed_proof,"invalid"),"actual B status is expected crypto rejection");controls++;
    reset();require(!engine_semantic(&owner,12,1),"generic nonzero never masquerades as corrupt proof");controls++;
    reset();output(&owner,"E\0\0\n",4);require(!engine_semantic(&owner,12,0),"missing-key/verifier error is not bad crypto proof");controls++;
    reset();owner.members[1].exit_status_known=0;output(&owner,"B\0\0\n",4);
    require(engine_semantic(&owner,12,0),"actual B and source-bound GS1/GS2 path does not require racy optional BSD status");controls++;
    reset();owner.members[1].exit_status_known=1;owner.members[1].exit_status=0;
    require(!engine_semantic(&owner,12,0),"observed contradictory verifier status refuses");controls++;
    reset();sig.settled=0;output(&owner,"B\0\0\n",4);require(!engine_semantic(&owner,12,0),"missing GS2 refuses bad-signature result");controls++;
    strcpy(outer.case_name,"positive");reset();private_guards_dispose();
}
static void nominal_result(struct private_engine_result *r) {
    memset(r,0,sizeof(*r));r->phase="settled";r->outcome="refused";r->observed_proof="valid";
    r->engine_entered=1;r->semantic_forms=32767;r->commands_completed=15;
    r->attempts=r->created=r->reaped=16;r->members_max=2;r->group_gone=r->retired_coverage=1;
    struct private_engine_metric *all[]={&r->ordinary_max_ns,&r->controller_max_ns,&r->cleanup_max_ns,&r->native_cpu_max_ns,
        &r->native_wall_max_ns,&r->inclusive_max_ns,&r->W_ns,&r->A_ns,&r->C_ns,&r->Q_max_ns,&r->post_cpu_ns,&r->late_kernel_ns,
        &r->debt_charged_ns,&r->debt_paid_ns,&r->helper_group_rss_peak_bytes,&r->probe_rss_peak_bytes,&r->metadata_bytes};
    for(unsigned i=0;i<sizeof(all)/sizeof(all[0]);i++)engine_max(all[i],1000+i);
}
static void result_controls(void) {
    struct private_engine_result r;
    reset();memset(&r,0,sizeof(r));
    require(!private_dispatch(1,0,1,&context,&r)&&!r.engine_entered,"unbound inner context cannot dispatch or create effects");controls++;
    reset();memset(&r,0,sizeof(r));
    // An admitted dispatch reaches the REAL engine, which independently
    // refuses this deliberately incomplete source-known context before birth.
    // A skipped-engine/hardcoded-refusal dispatch mutant loses engine_entered.
    require(!private_dispatch(1,1,1,&context,&r)&&r.engine_entered&&r.created==0
        &&!strcmp(r.phase,"validate")&&!strcmp(r.error_code,"review_unbound"),"admitted dispatch invokes the engine's own bounded prebirth refusal");controls++;
    for(unsigned variant=0;variant<9;variant++) {
        reset();nominal_result(&r);
        if(variant==1)r.engine_entered=0;
        if(variant==2)r.semantic_forms=0; // skipped engine/semantic path
        if(variant==3)r.created=r.reaped=15; // missing implicit verifier
        if(variant==4)r.Q_max_ns.value=1000001;
        if(variant==5)r.retired_coverage=0;
        if(variant==6)r.inclusive_max_ns.known=0;
        if(variant==7)private_cost.debt=9;
        if(variant==8)r.observed_proof="not_reached"; // hardcoded refusal
        int accepted=engine_complete_result(&context,&r);
        require(accepted==(variant==0),"same-invocation complete result conjunction");controls++;
    }
    reset();nominal_result(&r);require(engine_complete_result(&context,&r),"nominal source-known classifier positive");
    char bytes[8192];int n=private_format(&outer,input_hash,&r,bytes,sizeof(bytes));
    require(n>0&&(size_t)n<sizeof(bytes)&&bytes[n-1]=='\n'&&private_canonical((const unsigned char *)bytes,(size_t)n-1),"result canonical sorted JSON+LF");
    require(strstr(bytes,"\"attempts\":16")&&strstr(bytes,"\"created\":16")&&strstr(bytes,"\"reaped\":16")
        &&strstr(bytes,"\"outcome\":\"completed\"")&&strstr(bytes,"\"observed_proof\":\"valid\"")
        &&strstr(bytes,"\"runtime_approved\":false")&&strstr(bytes,"\"error\":null"),"result uses retained observed counters and proof");controls++;
    reset();nominal_result(&r);strcpy(outer.case_name,"corrupt_proof");r.observed_proof="invalid";
    require(engine_complete_result(&context,&r)&&!strcmp(r.outcome,"expected_rejection"),"specific corrupt result classification");controls++;
    strcpy(outer.case_name,"positive");
}
static void process_control(const char *self,int live) {
    reset();require(engine_owner_priority(),"source-known controller priority");
    // A fixture-only fork isolates this terminal-classification witness from
    // dyld/exec latency. It is never a candidate helper-birth qualification.
    // The native fork still obeys the existing2msCPU/25mswall bounds; all
    // original-token/RI/WNOWAIT/roster/window/actual-reap code is canonical.
    uint64_t cpu_begin=clock_ns(CLOCK_THREAD_CPUTIME_ID),wall_begin=clock_ns(CLOCK_MONOTONIC);
    pid_t pid=fork();
    if(pid==0) {
        if(setpgid(0,0)!=0)_exit(71);
        if(raise(SIGSTOP)!=0)_exit(72);
        if(live){volatile uint64_t work=1;for(;;)work=work*33+7;}
        _exit(0);
    }
    if(pid>0)retained_group=pid;
    uint64_t cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID)-cpu_begin,wall=clock_ns(CLOCK_MONOTONIC)-wall_begin;
    require(pid>0&&cpu<=2000000&&wall<=25000000,"source-known fork native2msCPU25mswall");
    struct image first;require(control_observe(pid,&first),"original source-known fork identity");
    struct private_engine_result result;memset(&result,0,sizeof(result));result.created=1;result.phase="git_reads";result.outcome="refused";result.observed_proof="not_reached";
    (void)control_stop_ack(pid,first.birth);
    struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
    owner.input=owner.output=owner.errors=owner.ledger=-1;owner.slots=1;owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};
    strcpy(owner.image,self);owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;
    (void)control_stop_ack(pid,first.birth);
    int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"original stopped attach before user work");
    owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
    owner.grant.operation=owner.ticket.start;owner.grant.sequence=1;owner.grant.slots=1;owner.grant.source_complete=1;owner.grant.cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(engine_actual_ticks(&owner,&owner.grant_tick_before,&cpu,&wall),"original baseline CPU ticks");
    owner.grant.t0=clock_ns(CLOCK_MONOTONIC);require(kill(-pid,SIGCONT)==0,"one original source-known CONT");
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+250000000ULL;int done=0;
    if(live) {
        struct rusage_info_v1 usage;
        do {require(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)==0,"live raw usage");if(usage.ri_user_time)break;}while(clock_ns(CLOCK_MONOTONIC)<until);
        require(usage.ri_user_time>0,"live source-known child actually ran");
    } else do {require(known_exited(pid,&done),"original WNOWAIT");if(done)break;}while(clock_ns(CLOCK_MONOTONIC)<until);
    require(live||done,"positive original terminal before injection");
    owner.stop_requested=1;uint64_t failed_at=clock_ns(CLOCK_MONOTONIC);
    require(engine_stop_result(&owner,-1,EPERM,failed_at)&&owner.stop_pending&&!owner.stop_terminal,"injected consumed failure is pending only");
    int closed=0;require(engine_roster(&owner,1,&closed),"same-controller independent original observation");
    require(closed==!live,"running child cannot acquire terminal classification");
    require(owner.stop_attempts==1&&engine_stop_group(&owner)&&owner.stop_attempts==1,"pending consumed STOP is never repeated");
    if(live) {
        require(!owner.grant.roster_closed&&!owner.grant.end[0],"live endpoint remains open after failed call");
        (void)engine_failure(&owner,"cancelled","source-known runnable negative enters identity-safe cleanup; never a qualifying command");
        require(engine_token_signal(&owner.members[0].token,SIGKILL)==0,"identity-safe cleanup of original live source-known child");
        do {require(known_exited(pid,&done),"cleanup original WNOWAIT");}while(!done&&clock_ns(CLOCK_MONOTONIC)<until);
        require(done&&engine_member_exit(&owner,0),"actual cleanup independent original terminal");
    } else require(owner.stop_terminal&&owner.stop_terminal_ns>=failed_at&&owner.grant.end[0]>=failed_at,"terminal endpoint follows positive proof, never errno time");
    owner.grant.cpu_close=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    uint64_t tick=0;require(engine_actual_delta(&owner,owner.grant_tick_before,&owner.grant.actual_cpu,&tick),"full terminal helper CPU interval");
    engine_window_cost(&owner,clock_ns(CLOCK_MONOTONIC),closed);
    // Terminal classification is only one predicate. Preserve observed costs:
    // a cold source-known executable can exceed Q and must stay unqualified.
    // The same production arithmetic must refuse that entire resource case.
    struct window_ledger checked=owner.grant;enum ledger_result verdict=window_charge(&checked);
    if(!live) {
        fprintf(stderr,"private_terminal_gate verdict=%d operation=%llu sequence=%llu ack=%u identity=%u closed=%d W=%llu A=%llu C=%llu Q=%llu\n",
            verdict,(unsigned long long)owner.grant.operation,(unsigned long long)owner.grant.sequence,
            owner.grant.ack_mask,owner.grant.identity_mask,owner.grant.roster_closed,(unsigned long long)checked.windows,
            (unsigned long long)owner.grant.actual_cpu,(unsigned long long)checked.controller,(unsigned long long)checked.charge);
        require(verdict==(owner.grant.charge<=1000000?LEDGER_OK:LEDGER_LIMIT),"terminal source-known resource verdict is executable, never waived");
    }
    const char *ledger_name=live?"live-ledger.ndjson":"terminal-ledger.ndjson";
    owner.ledger=openat(outer.repo.fd,ledger_name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(owner.ledger>=0,"exclusive owned raw terminal ledger");
    int status=-1;(void)engine_retire(&owner,&status);
    require(owner.direct_reaped&&owner.wait_observed&&result.reaped==1&&result.group_gone
        &&result.retired_coverage,"same-controller actual source-known wait4 and complete owned group settlement");
    require(fsync(owner.ledger)==0&&private_close(owner.ledger),"owned terminal raw ledger persistence and close");
    owner.ledger=-1;
    printf("private_terminal_source_known fixture_fork_only=true classification_only=true qualifies_command=false live=%d injected_signal=true signal_result=%d signal_errno=%d stop_attempts=%u pid=%d pgid=%d birth=%llu token_version=%u failed_call_ns=%llu terminal_ns=%llu t0=%llu end=%llu close=%llu W=%llu A=%llu C=%llu Q=%llu Q_within_1ms=%s wait4_user_ns=%llu wait4_system_ns=%llu late_kernel_ns=%llu late_known=%u post_cpu_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu actual_reaped=true group_gone=true runtime_approved=false\n",
        live,owner.stop_result,owner.stop_errno,owner.stop_attempts,pid,pid,(unsigned long long)owner.ticket.birth,owner.members[0].token.val[7],
        (unsigned long long)failed_at,(unsigned long long)owner.stop_terminal_ns,(unsigned long long)owner.grant.t0,
        (unsigned long long)owner.grant.end[0],(unsigned long long)owner.grant.t_close,(unsigned long long)owner.grant.windows,
        (unsigned long long)owner.grant.actual_cpu,(unsigned long long)owner.grant.controller,(unsigned long long)owner.grant.charge,
        owner.grant.charge<=1000000?"true":"false",(unsigned long long)owner.wait_user,(unsigned long long)owner.wait_system,
        (unsigned long long)result.late_kernel_ns.value,result.late_kernel_ns.known,(unsigned long long)result.post_cpu_ns.value,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    reset();controls++;
}
int main(int argc,char **argv) {
    if(argc==2&&!strcmp(argv[1],"exit-child"))_exit(0);
    if(argc==2&&!strcmp(argv[1],"busy-child")){volatile uint64_t work=1;for(;;)work=work*33+7;}
    require(argc==1,"private source-known controls argc");require(atexit(dispose_owned)==0,"owned cleanup registration");
    char root[]="/private/tmp/dream-private-engine-controls.XXXXXX";require(mkdtemp(root)!=NULL,"owned fixture directory");
    int dir=open(root,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0,"owned fixture root");
    int fd=openat(dir,"payload",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(fd>=0&&write(fd,payload,sizeof(payload)-1)==sizeof(payload)-1&&close(fd)==0,"owned pinned unsigned public payload");
    reset();fixture(dir);semantic_controls();result_controls();
    char self[PROC_PIDPATHINFO_MAXSIZE];require(proc_pidpath(getpid(),self,sizeof(self))>0,"actual source-known image");
    process_control(self,0);process_control(self,1);
    require(close(dir)==0,"owned fixture root close");
    printf("private_engine_controls=pass cases=%u synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);
    return 0;
}
