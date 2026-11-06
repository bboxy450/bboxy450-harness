// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Synthetic source-known API/semantic/terminal controls. No real candidate,
// source authentication, OpenPGP qualification or runtime approval is claimed.
#define DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS 1
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
    basis.fixture.origin=(struct private_slice){(const unsigned char *)"\"https://example.invalid/fixture.git\"",sizeof("\"https://example.invalid/fixture.git\"")-1};
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
static unsigned flow_variant,flow_commands;
static int source_known_current(const struct private_engine_context *ctx) {
    return private_context_same(ctx)&&private_guards.stage==2;
}
static int source_known_command(struct engine_owner *owner,struct engine_prepared *prepared,unsigned form,
    const char *home,const char *scratch,const unsigned char *input,size_t length,int *status) {
    (void)prepared;(void)home;(void)scratch;(void)input;(void)length;
    flow_commands++;owner->command=form;owner->verified_descendants=owner->child_created=owner->child_settled=0;
    unsigned slots=private_forms[form].members;owner->result->attempts+=(uint16_t)slots;
    owner->result->created+=(uint16_t)slots;owner->result->reaped+=(uint16_t)slots;
    if(owner->result->members_max<slots)owner->result->members_max=(uint8_t)slots;
    owner->result->group_gone=owner->result->retired_coverage=1;*status=0;
    struct private_engine_metric *metrics[]={&owner->result->ordinary_max_ns,&owner->result->controller_max_ns,&owner->result->cleanup_max_ns,
        &owner->result->native_cpu_max_ns,&owner->result->native_wall_max_ns,&owner->result->inclusive_max_ns,&owner->result->W_ns,
        &owner->result->A_ns,&owner->result->C_ns,&owner->result->Q_max_ns,&owner->result->post_cpu_ns,&owner->result->late_kernel_ns,
        &owner->result->helper_group_rss_peak_bytes,&owner->result->probe_rss_peak_bytes};
    for(unsigned i=0;i<sizeof(metrics)/sizeof(metrics[0]);i++)engine_max(metrics[i],1000+i);
    char text[512];int n=0;
    switch(form) {
        case 0:output(owner,"false\n",6);break;
        case 1:n=snprintf(text,sizeof(text),"%s\n",owner->context->outer->repo.path);output(owner,text,(size_t)n);break;
        case 2:case 3:n=snprintf(text,sizeof(text),"%s\n",basis.fixture.valid);output(owner,text,(size_t)n);break;
        case 4:output(owner,"commit\n",7);break;
        case 5:case 14:output(owner,NULL,0);break;
        case 6:n=snprintf(text,sizeof(text),"100644 blob %s\tsecond",basis.fixture.blob);output(owner,text,(size_t)n+1);break;
        case 7:output(owner,"1\n",2);break;
        case 8:n=snprintf(text,sizeof(text),"%s\n",engine_selected_commit(owner->context));output(owner,text,(size_t)n);break;
        case 9:n=snprintf(text,sizeof(text),"%s\n",basis.fixture.blob);output(owner,text,(size_t)n);break;
        case 10:output(owner,"dee19491977e73e485607928be1e1a34625a4ffd\n",41);break;
        case 11:output(owner,"https://example.invalid/fixture.git\n",sizeof("https://example.invalid/fixture.git\n")-1);break;
        case 12: {
            int corrupt=!strcmp(outer.case_name,"corrupt_proof");
            owner->verified_descendants=flow_variant==1?0:1;owner->child_created=owner->child_settled=1;
            owner->members[1].exit_status_known=1;owner->members[1].exit_status=(corrupt?1:0)<<8;
            owner->git_signature->bound=owner->git_signature->settled=1;
            if(corrupt)output(owner,"B\0\0\n",4);
            else {text[0]='G';text[1]=0;memcpy(text+2,basis.fixture.signer,40);text[42]=0;text[43]='\n';output(owner,text,44);}
            if(flow_variant==2)*status=1;
            break;
        }
        case 13:n=snprintf(text,sizeof(text),"fpr:::::::::%s:\n",basis.fixture.signer);output(owner,text,(size_t)n);break;
        default:return 0;
    }
    // The surrounding actual owner still measures/pays its real ordinary work.
    // The native/window/counter values above are SCRIPTED test records, never
    // observations of a candidate, source authentication or qualification.
    return private_end_unit(private_cost.open_cpu);
}
static void ref_for(struct private_ref *ref,const char *token,const void *bytes,size_t n) {
    memset(ref,0,sizeof(*ref));ref->path=(struct private_slice){(const unsigned char *)token,strlen(token)};
    ref->len=n;private_hash(bytes,n,ref->sha);
}
static void flow_controls(const char *root,int dir) {
    const struct private_source_known_effects effects={source_known_current,source_known_command};
    for(unsigned variant=0;variant<4;variant++) {
        reset();fixture(dir);basis.source_coverage=127;basis.command_coverage=65535;private_guards.stage=2;
        outer.prefix.fd=dir;strcpy(outer.repo.path,root);strcpy(outer.case_name,variant==3||variant==2?"corrupt_proof":"positive");
        char out[4097];int n=snprintf(out,sizeof(out),"%s/flow-%u",root,variant);
        require(n>0&&(size_t)n<sizeof(out)&&mkdir(out,0700)==0,"owned source-known flow output");
        strcpy(outer.output.path,out);outer.output.fd=open(out,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
        require(outer.output.fd>=0,"owned source-known output custody");
        const char config[]="batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n";
        ref_for(&basis.gpg_conf,"\"gpg.conf\"",config,sizeof(config)-1);ref_for(&basis.common_conf,"\"common.conf\"","",0);
        ref_for(&basis.fixture.keys,"\"keys\"","SOURCE-KNOWN PUBLIC KEY INPUT\n",sizeof("SOURCE-KNOWN PUBLIC KEY INPUT\n")-1);
        basis.fixture.valid_signature.path=(struct private_slice){(const unsigned char *)"\"valid-signature\"",sizeof("\"valid-signature\"")-1};
        basis.fixture.corrupt_signature.path=(struct private_slice){(const unsigned char *)"\"corrupt-signature\"",sizeof("\"corrupt-signature\"")-1};
        memset(&engine_signature_storage,0,sizeof(engine_signature_storage));engine_signature_storage.scratch_fd=engine_signature_storage.signature_fd=-1;
        flow_variant=variant;flow_commands=0;private_source_known_effects=&effects;
        struct private_engine_result r;memset(&r,0,sizeof(r));
        int completed=private_dispatch(1,1,1,&context,&r);
        private_source_known_effects=NULL;
        require(completed==(variant==0||variant==3)&&flow_commands==15,"same functional validate/claim/15-command/semantic/settle flow");
        require(r.engine_entered&&r.created==16&&r.reaped==16&&r.attempts==16,"scripted effects flow through real accounting owner");
        if(completed)require(r.semantic_forms==32767&&r.commands_completed==15&&!strcmp(r.outcome,variant==3?"expected_rejection":"completed"),"same real flow completion and proof outcome");
        else require(r.semantic_forms!=32767&&r.commands_completed==14&&!strcmp(r.observed_proof,"not_reached"),"missing verifier/generic error cannot complete corrupt proof");
        reset();private_guards_dispose();require(close(outer.output.fd)==0,"source-known output close");outer.output.fd=-1;
        controls++;
    }
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
static struct image fixture_stopped(pid_t pid) {
    // The fixture child must have reached its OWN initial stop before the
    // tight production observation. Pacing this setup avoids starving a newly
    // forked utility-priority child in a controller busy loop. No native/1ms
    // production gate or existing100ms readiness bound is widened.
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000ULL;
    for(;;) {
        siginfo_t info;memset(&info,0,sizeof(info));
        require(waitid(P_PID,pid,&info,WSTOPPED|WNOHANG|WNOWAIT)==0,"fixture original nonreaping stop notification");
        if(info.si_pid==pid&&info.si_code==CLD_STOPPED) {
            struct image first;require(control_observe(pid,&first),"fixture original stopped birth");
            (void)control_stop_ack(pid,first.birth);return first;
        }
        if(clock_ns(CLOCK_MONOTONIC)>=until) {
            (void)kill(pid,SIGKILL);require(0,"fixture initial stop readiness exhausted");
        }
        uint64_t used=clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu;
        if(used>=200000&&!private_end_unit(private_cost.open_cpu)) {
            (void)kill(pid,SIGKILL);require(0,"fixture initial stop setup unit refused");
        }
        usleep(100);
    }
}
// Owned kernel pipes exercise the same current controller's 8KiB transfer:
// one4KiB chunk precedes a consumed STOP failure; the remaining chunk cannot
// enter the pipe in pending or terminal state. No new helper/controller exists.
struct pending_stdin_fixture {
    int input_read,output_peer,error_peer;unsigned char payload[8192],*capture;
};
static void pending_stdin_prepare(struct engine_owner *owner,struct pending_stdin_fixture *fixture) {
    int input[2],output[2],errors[2];
    require(owner->input<0&&owner->output<0&&owner->errors<0&&pipe(input)==0&&pipe(output)==0&&pipe(errors)==0,"owned pending stdin fixture pipes");
    fixture->input_read=input[0];fixture->output_peer=output[1];fixture->error_peer=errors[1];
    owner->input=input[1];owner->output=output[0];owner->errors=errors[0];
    int fds[]={input[0],input[1],output[0],output[1],errors[0],errors[1]};
    for(unsigned i=0;i<6;i++)require(fcntl(fds[i],F_SETFL,O_NONBLOCK)==0&&fcntl(fds[i],F_SETFD,FD_CLOEXEC)==0,"owned pending fixture nonblocking descriptors");
    memset(fixture->payload,'p',sizeof(fixture->payload));fixture->capture=malloc(ENGINE_CAPTURE_MAX);
    require(fixture->capture!=NULL,"owned bounded pending fixture capture");
    owner->capture=fixture->capture;owner->stdin_bytes=fixture->payload;owner->stdin_len=sizeof(fixture->payload);
}
static void pending_stdin_first_chunk(struct engine_owner *owner,struct pending_stdin_fixture *fixture) {
    require(!owner->stop_pending&&!owner->stop_terminal&&engine_pipe_io(owner)&&owner->sent==4096,"one actual bounded input chunk before pending transition");
    unsigned char bytes[4096];require(read(fixture->input_read,bytes,sizeof(bytes))==sizeof(bytes)&&!memcmp(bytes,fixture->payload,sizeof(bytes)),"actual pipe contains the first admitted chunk only");
}
static void pending_stdin_refusals(struct engine_owner *owner,struct pending_stdin_fixture *fixture) {
    require(owner->stop_pending&&owner->sent==4096&&owner->sent<owner->stdin_len,"actual consumed failure occurs during the bounded input transfer");
    require(write(fixture->output_peer,"o",1)==1&&write(fixture->error_peer,"e",1)==1,"owned output remains drainable during pending terminal observation");
    require(engine_pipe_io(owner)&&owner->sent==4096&&owner->out_len==1&&owner->err_len==1
        &&owner->capture[0]=='o'&&owner->capture[ENGINE_CAPTURE_MAX-1]=='e',"pending stdin refuses the next chunk while stdout/stderr drain");
    unsigned char byte;errno=0;require(read(fixture->input_read,&byte,1)==-1&&(errno==EAGAIN||errno==EWOULDBLOCK),"pending input pipe received zero new payload bytes");
    // Exercise the terminal snapshot as a separate source-known API control;
    // preserve/restore the actual original pending signal/identity state.
    unsigned pending=owner->stop_pending,terminal=owner->stop_terminal;
    owner->stop_pending=0;owner->stop_terminal=1;
    require(engine_pipe_io(owner)&&owner->sent==4096,"terminal stdin refuses every remaining payload chunk");
    errno=0;require(read(fixture->input_read,&byte,1)==-1&&(errno==EAGAIN||errno==EWOULDBLOCK),"terminal input pipe received zero new payload bytes");
    owner->stop_pending=pending;owner->stop_terminal=terminal;
    require(close(owner->input)==0&&close(owner->output)==0&&close(owner->errors)==0
        &&close(fixture->input_read)==0&&close(fixture->output_peer)==0&&close(fixture->error_peer)==0,"owned pending transport fixture closes exact descriptors");
    owner->input=owner->output=owner->errors=-1;
    printf("private_pending_stdin_source_known pid=%d pre_transition_bytes=4096 total_input_bytes=8192 pending_new_bytes=0 terminal_new_bytes=0 stdout_drained=1 stderr_drained=1 pipe_cpu_subset_ns=%llu no_second_signal=true real_candidate_created=0 runtime_approved=false\n",
        owner->ticket.pid,(unsigned long long)owner->pipe_cpu);
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
    struct image first=fixture_stopped(pid);
    struct private_engine_result result;memset(&result,0,sizeof(result));result.created=1;result.phase="git_reads";result.outcome="refused";result.observed_proof="not_reached";
    (void)control_stop_ack(pid,first.birth);
    struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
    owner.input=owner.output=owner.errors=owner.ledger=-1;owner.slots=1;owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};
    strcpy(owner.image,self);owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;
    (void)control_stop_ack(pid,first.birth);
    int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"original stopped attach before user work");
    owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
    struct pending_stdin_fixture transport;pending_stdin_prepare(&owner,&transport);
    owner.grant.operation=owner.ticket.start;owner.grant.sequence=1;owner.grant.slots=1;owner.grant.source_complete=1;owner.grant.cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(engine_actual_ticks(&owner,&owner.grant_tick_before,&cpu,&wall),"original baseline CPU ticks");
    owner.grant.t0=clock_ns(CLOCK_MONOTONIC);require(kill(-pid,SIGCONT)==0,"one original source-known CONT");
    pending_stdin_first_chunk(&owner,&transport);
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+250000000ULL;int done=0;
    if(live) {
        struct rusage_info_v1 usage;
        do {require(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&usage)==0,"live raw usage");if(usage.ri_user_time)break;}while(clock_ns(CLOCK_MONOTONIC)<until);
        require(usage.ri_user_time>0,"live source-known child actually ran");
    } else do {require(known_exited(pid,&done),"original WNOWAIT");if(done)break;}while(clock_ns(CLOCK_MONOTONIC)<until);
    require(live||done,"positive original terminal before injection");
    owner.stop_requested=1;uint64_t failed_at=clock_ns(CLOCK_MONOTONIC);
    require(engine_stop_result(&owner,-1,EPERM,failed_at)&&owner.stop_pending&&!owner.stop_terminal,"injected consumed failure is pending only");
    pending_stdin_refusals(&owner,&transport);
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
    free(transport.capture);owner.capture=NULL;
    reset();controls++;
}
static void pending_exec_control(const char *self,int dir) {
    reset();outer.deadline=private_cost.deadline;strcpy(outer.case_name,"positive");
    int gate[2],ready[2];require(pipe(gate)==0&&pipe(ready)==0,"owned source-known exec gate and completion");
    pid_t pid=fork();
    if(pid==0) {
        close(gate[1]);close(ready[0]);if(setpgid(0,0)!=0||raise(SIGSTOP)!=0)_exit(73);
        char byte;if(read(gate[0],&byte,1)!=1)_exit(74);close(gate[0]);
        char fd[32];snprintf(fd,sizeof(fd),"%d",ready[1]);execl(self,self,"pending-exec-child",fd,NULL);_exit(75);
    }
    require(pid>0,"source-known exec fork");retained_group=pid;close(gate[0]);close(ready[1]);
    require(fcntl(ready[0],F_SETFL,O_NONBLOCK)==0,"owned source-known nonblocking exec completion");
    struct image first=fixture_stopped(pid);
    struct private_engine_result r;memset(&r,0,sizeof(r));r.created=1;r.phase="git_reads";r.outcome="refused";r.observed_proof="not_reached";
    struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&r;
    owner.input=owner.output=owner.errors=-1;owner.slots=1;owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};
    strcpy(owner.image,self);owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;
    int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"source-known exec original final image admitted");
    owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
    owner.grant.operation=owner.ticket.start;owner.grant.sequence=1;owner.grant.slots=1;owner.grant.source_complete=1;
    owner.grant.cpu_start=private_cost.open_cpu;uint64_t raw,user,system;
    require(engine_actual_ticks(&owner,&raw,&user,&system),"source-known exec original CPU baseline");owner.grant_tick_before=raw;
    owner.grant.t0=clock_ns(CLOCK_MONOTONIC);require(kill(-pid,SIGCONT)==0,"source-known exec single CONT");
    owner.stop_requested=1;require(engine_stop_result(&owner,-1,EPERM,clock_ns(CLOCK_MONOTONIC)),"source-known injected pending failure");
    require(write(gate[1],"x",1)==1&&close(gate[1])==0,"source-known fixture stimulates actual exec without another CONT");
    char completed=0;int exec_ready=0;
    while(clock_ns(CLOCK_MONOTONIC)<outer.deadline) {
        ssize_t n=read(ready[0],&completed,1);if(n==1){exec_ready=completed=='x';break;}
        require(n<0&&(errno==EAGAIN||errno==EINTR),"owned actual exec completion remains pending");usleep(100);
    }
    require(exec_ready&&close(ready[0])==0,"actual exec completion precedes this controlled observation");
    unsigned old_version=owner.members[0].token.val[7];
    while(!owner.pending_identity_lost&&clock_ns(CLOCK_MONOTONIC)<outer.deadline) {
        if(!engine_sample_member(&owner,0,pid,&ack))break;
        require(!ack&&!owner.grant.end[0],"pending original never acknowledged before actual exec negative");
    }
    require(owner.pending_identity_lost&&owner.coverage_lost&&owner.failed&&!owner.stop_terminal
        &&owner.retired_count==2&&owner.retired[1].token.val[7]!=old_version,"observed real same-PID exec stickily invalidates pending continuity");
    owner.ledger=openat(dir,"exec-ledger.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(owner.ledger>=0,"owned actual exec negative ledger");
    int status=-1;(void)engine_retire(&owner,&status);
    require(owner.settlement_quarantined&&!r.group_gone&&!r.retired_coverage,"exec successor never becomes original terminal proof after cleanup");
    unsigned kills=owner.retired[1].kill_attempted;dispose_owned();
    require(owner.retired[1].kill_attempted==kills&&owner.stop_attempts==1,"atexit never repeats pending STOP or consumed kill");
    if(!owner.direct_reaped) {
        int done=0;while(!done&&clock_ns(CLOCK_MONOTONIC)<outer.deadline){require(known_exited(pid,&done),"fixture original WNOWAIT after safe exec cleanup");if(!done)usleep(100);}
        require(done&&waitpid(pid,NULL,0)==pid,"fixture actual exec negative reap within original deadline");
    }
    int group[3];require(proc_listpgrppids(pid,group,sizeof(group))==0,"fixture exec negative actual group gone");retained_group=0;
    printf("private_pending_exec_source_known actual_exec=true pid=%d birth=%llu original_version=%u successor_version=%u sticky_refusal=true original_terminal_proved=false actual_cleanup_reaped=true group_gone=true runtime_approved=false\n",
        pid,(unsigned long long)owner.ticket.birth,old_version,owner.retired[1].token.val[7]);
    require(close(owner.ledger)==0,"owned exec ledger close");engine_current=NULL;dispose_private_engine=NULL;reset();controls++;
}
static void settlement_control(const char *self,int dir,unsigned variant) {
    unsigned unknown_deadline=variant==1,persistence_failure=variant==2;
    reset();outer.output.fd=dir;outer.deadline=private_cost.deadline;strcpy(outer.case_name,"positive");
    pid_t pid=fork();
    if(pid==0){if(setpgid(0,0)!=0||raise(SIGSTOP)!=0)_exit(76);if(unknown_deadline){volatile unsigned work=1;for(;;)work=work*33+7;}_exit(0);}
    require(pid>0,"owned source-known settlement fork");retained_group=pid;
    struct image first=fixture_stopped(pid);
    struct private_engine_result r;memset(&r,0,sizeof(r));r.created=1;r.phase="git_reads";r.outcome="refused";r.observed_proof="not_reached";
    struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&r;
    owner.input=owner.output=owner.errors=-1;owner.slots=unknown_deadline?1:2;owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};
    strcpy(owner.image,self);owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;
    int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"settlement source-known original attach");
    owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
    owner.grant.operation=owner.ticket.start;owner.grant.sequence=1;owner.grant.slots=owner.slots;owner.grant.source_complete=1;
    owner.grant.cpu_start=private_cost.open_cpu;uint64_t raw,u,k;require(engine_actual_ticks(&owner,&raw,&u,&k),"settlement original CPU baseline");owner.grant_tick_before=raw;
    owner.grant.t0=clock_ns(CLOCK_MONOTONIC);require(kill(-pid,SIGCONT)==0,"settlement source-known one original CONT");
    if(!unknown_deadline) {
        int done=0;while(!done&&clock_ns(CLOCK_MONOTONIC)<outer.deadline)require(known_exited(pid,&done),"unknown-slot source-known original WNOWAIT");
        require(done&&engine_member_exit(&owner,0),"unknown-slot source-known actual original terminal");
        // One INJECTED unresolved reserved slot, not a second physical process.
        // An empty census cannot close it or manufacture a token/start binding.
        owner.members[1].state.pid=pid;owner.grant.unknown_mask=2;
        (void)engine_failure(&owner,"coverage_unknown","source-known injected unresolved child/token slot");
    }
    const char *ledger_name=unknown_deadline?"deadline-ledger.ndjson":persistence_failure?"failed-receipt-ledger.ndjson":"unknown-ledger.ndjson";
    owner.ledger=openat(dir,ledger_name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    if(persistence_failure){require(owner.ledger>=0&&close(owner.ledger)==0,"owned persistence negative creation");owner.ledger=openat(dir,ledger_name,O_RDONLY|O_NOFOLLOW|O_CLOEXEC);}
    require(owner.ledger>=0,"owned settlement negative raw ledger");
    source_known_settlement_deadline_unknown=unknown_deadline;
    uint64_t began=clock_ns(CLOCK_MONOTONIC);int status=-1;int completed=engine_retire(&owner,&status);uint64_t ended=clock_ns(CLOCK_MONOTONIC);
    source_known_settlement_deadline_unknown=0;
    require(!completed&&owner.settlement_quarantined&&!r.group_gone&&!r.retired_coverage
        &&!strcmp(r.outcome,"quarantined")&&engine_current==&owner,"unknown original settlement is bounded owned quarantine");
    require(owner.quarantine_durable==!persistence_failure,"actual failed persistence never becomes a durable-evidence claim");
    require(ended>=began&&ended<outer.deadline,"settlement negative returns inside the original thirty-second window");
    unsigned direct_reaped=owner.direct_reaped,group_kills=owner.group_kill_attempted;uint16_t reported_reaps=r.reaped;
    uint64_t exit_begin=clock_ns(CLOCK_MONOTONIC);dispose_owned();uint64_t exit_end=clock_ns(CLOCK_MONOTONIC);
    require(owner.direct_reaped==direct_reaped&&r.reaped==reported_reaps&&owner.group_kill_attempted==group_kills
        &&!r.group_gone&&!r.retired_coverage&&exit_end<outer.deadline,"atexit retains quarantine without blocking, replaying effects or asserting completion");
    if(!owner.direct_reaped) {
        int done=0;while(!done&&clock_ns(CLOCK_MONOTONIC)<outer.deadline){require(known_exited(pid,&done),"fixture original WNOWAIT after consumed kill");if(!done)usleep(100);}
        require(done&&waitpid(pid,NULL,0)==pid,"fixture owner actually reaps its original within unchanged deadline");
    }
    int group[3];require(proc_listpgrppids(pid,group,sizeof(group))==0,"fixture negative actual physical group gone");retained_group=0;
    printf("private_settlement_source_known injected_unknown_deadline=%u injected_unknown_slot=%u pid=%d original_deadline=%llu return_wall_ns=%llu atexit_wall_ns=%llu reported_created=%u reported_reaped=%u reported_group_gone=false reported_retired_coverage=false quarantine_retained_in_process=true full_audit_receipt=%s live_kernel_custody_after_exit=false recovery_handoff_proved=false fixture_cleanup_actual_reaped=true runtime_approved=false\n",
        unknown_deadline,!unknown_deadline,pid,(unsigned long long)outer.deadline,(unsigned long long)(ended-began),(unsigned long long)(exit_end-exit_begin),r.created,r.reaped,owner.quarantine_durable?"true":"false");
    require(close(owner.ledger)==0,"owned quarantine raw ledger close");engine_current=NULL;dispose_private_engine=NULL;reset();controls++;
}
int main(int argc,char **argv) {
    if(argc==2&&!strcmp(argv[1],"exit-child"))_exit(0);
    if(argc==3&&!strcmp(argv[1],"pending-exec-child")){
        int fd=atoi(argv[2]);if(fd<3||write(fd,"x",1)!=1||close(fd)!=0)_exit(77);
        volatile unsigned work=1;for(;;)work=work*33+7;
    }
    if(argc==2&&!strcmp(argv[1],"busy-child")){volatile uint64_t work=1;for(;;)work=work*33+7;}
    require(argc==1,"private source-known controls argc");require(atexit(dispose_owned)==0,"owned cleanup registration");
    char root[]="/private/tmp/dream-private-engine-controls.XXXXXX";require(mkdtemp(root)!=NULL,"owned fixture directory");
    int dir=open(root,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0,"owned fixture root");
    int fd=openat(dir,"payload",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(fd>=0&&write(fd,payload,sizeof(payload)-1)==sizeof(payload)-1&&close(fd)==0,"owned pinned unsigned public payload");
    // Adapted public signature bytes: https://github.com/git/git/blob/v2.54.0/t/t7510-signed-commit.sh
    // Retain project credit/policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
    const char valid_signature[]="-----BEGIN PGP SIGNATURE-----\n\niHQEABECADQWIQRz11h0S+chaY7FTocTtvUezd5DDQUCX/uBDRYcY29tbWl0dGVy\nQGV4YW1wbGUuY29tAAoJEBO29R7N3kMNd+8AoK1I8mhLHviPH+q2I5fIVgPsEtYC\nAKCTqBh+VabJceXcGIZuF0Ry+udbBQ==\n=tQ0N\n-----END PGP SIGNATURE-----\n";
    const char corrupt_signature[]="-----BEGIN PGP SIGNATURE-----\n\niHQEABECADQWIQRz11h0S+chaY7FTocTtvUezd5DDQUCX/uBDRYcY29tbWl0dGVy\nQGV4YW1wbGUuY29tAAoJEBO29R7N3kMNd+8AoK1I8mhLHviPH+q2I5fIVgPsEtYC\nAKCTqBh+VabJceXcGIZuF0Ry+udbBA==\n=M0H2\n-----END PGP SIGNATURE-----\n";
    const char config[]="batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n";
    const char *names[]={"valid-signature","corrupt-signature","gpg.conf","common.conf","keys"};
    const char *values[]={valid_signature,corrupt_signature,config,"","SOURCE-KNOWN PUBLIC KEY INPUT\n"};
    for(unsigned i=0;i<5;i++) {
        int own=openat(dir,names[i],O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);size_t length=strlen(values[i]);
        require(own>=0&&write(own,values[i],length)==(ssize_t)length&&close(own)==0,"owned source-known flow input");
    }
    reset();fixture(dir);semantic_controls();result_controls();flow_controls(root,dir);
    char self[PROC_PIDPATHINFO_MAXSIZE];require(proc_pidpath(getpid(),self,sizeof(self))>0,"actual source-known image");
    process_control(self,0);process_control(self,1);
    pending_exec_control(self,dir);settlement_control(self,dir,0);settlement_control(self,dir,1);settlement_control(self,dir,2);
    require(close(dir)==0,"owned fixture root close");
    printf("private_engine_controls=pass cases=%u synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);
    return 0;
}
