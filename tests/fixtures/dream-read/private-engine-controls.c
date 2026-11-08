// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Synthetic source-known API/semantic/terminal controls. No real candidate,
// source authentication, OpenPGP qualification or runtime approval is claimed.
#define DREAM_PRIVATE_SOURCE_KNOWN_CONTROLS 1
#define main canonical_probe_main
#include "proof-helper-probe.c"
#undef main
#include <poll.h>
#include <pthread.h>

static const char payload[]="tree dee19491977e73e485607928be1e1a34625a4ffd\nparent 2ac709c836778e8203ecc9232daa5153a166cc0f\nauthor A U Thor <author@example.com> 1112912653 -0700\ncommitter C O Mitter <committer@example.com> 1112912653 -0700\n\nsecond\n";
static unsigned controls;
static struct private_input outer;
static struct private_basis basis;
static const char input_hash[]="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static struct private_engine_context context={&outer,&basis,input_hash};
static unsigned char capture[8192];
static struct engine_owner *fixture_live_owner;
static char fixture_release_path[4097];
static void same_wake_prepare(struct engine_owner *,const char *,int);
static void fixture(int);
static void fixture_reset_cost(void) {
    require(!engine_current,"previous original owner must be settled before fixture reset");
    memset(&private_cost,0,sizeof(private_cost));memset(&private_check_batch,0,sizeof(private_check_batch));private_reset_diagnostics();private_failed=0;private_error="input_invalid";private_detail="fixture unset";
    private_cost.entry=clock_ns(CLOCK_MONOTONIC);private_cost.deadline=private_cost.entry+30000000000ULL;
    private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);engine_cancel_signal=engine_owner_loss_signal=0;
    source_known_payment_interrupts=source_known_payment_fail_nth=source_known_schedule_interrupts=source_known_schedule_fail_nth=0;
    source_known_tail_over_cpu=source_known_final_kill_eperm=source_known_terminal_rss_unavailable=0;
    source_known_taskall_pid=0;source_known_taskall_error=source_known_taskall_clock_failure=0;
    source_known_taskall_native_result=source_known_taskall_native_errno=0;source_known_before_live_read=NULL;source_known_live_rechecks=0;source_known_live_read_error=0;source_known_child_watch_bound=NULL;source_known_child_handoff=NULL;
    source_known_event_registration_failure=0;source_known_process_create_case=0;source_known_flow_stage=source_known_flow_failure_stage=0;memset(&source_known_flow_failure_cost,0,sizeof(source_known_flow_failure_cost));
}
static void fixture_cleanup(void) {
    struct engine_owner *o=fixture_live_owner;if(!o||o->ticket.pid<=0)return;
    // Only this fixture's still-unreaped original positive PID is signalled.
    // The consuming wait clears the retained numeric PID immediately.
    if(fixture_release_path[0]) {int fd=open(fixture_release_path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);if(fd>=0)(void)close(fd);}
    int done=0;if(known_exited(o->ticket.pid,&done)&&!done&&o->direct_custody&&!o->group_kill_attempted) {
        o->group_kill_attempted=1;errno=0;int sent=kill(-o->ticket.pgid,SIGKILL);o->group_kill_result=sent?errno:0;
    }
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+2000000000ULL;struct rusage usage;memset(&usage,0,sizeof(usage));int status=0;pid_t got=0;
    while((got=wait4(o->ticket.pid,&status,WNOHANG,&usage))==0&&clock_ns(CLOCK_MONOTONIC)<until) {usleep(1000);(void)private_end_unit(private_cost.open_cpu);}
    int wait_error=got<0?errno:0;
    if(got==o->ticket.pid) {retained_group=0;o->direct_reaped=1;o->direct_custody=0;}
    int ids[3]={0},group_error=0;int count=native_group_count(o->ticket.pgid,ids,sizeof(ids),&group_error);
    fprintf(stderr,"private_total_fixture_failure_cleanup_source_known original_pid=%d own_wait4=%d wait_errno=%d raw_status=%d group_count=%d group_errno=%d retained_numeric_pid=%d actual_usage_ns=%llu runtime_approved=false\n",o->ticket.pid,got==o->ticket.pid,wait_error,status,count,group_error,retained_group,(unsigned long long)(got==o->ticket.pid?ru_cpu(&usage):UINT64_MAX));
    if((got==o->ticket.pid||(got<0&&wait_error==ECHILD&&o->wait_observed))&&native_pid_list_empty(count,group_error)) {engine_current=NULL;dispose_private_engine=NULL;}
}
static void fixture_check(int ok,const char *label) {
    if(ok)return;
    struct engine_owner *o=fixture_live_owner;
    if(o)fprintf(stderr,"private_total_fixture_guard label=%s native_cpu_ns=%llu native_wall_ns=%llu inclusive_ns=%llu ordinary_max_ns=%llu last_unit_ns=%llu debt_ns=%llu paid_ns=%llu original_pid=%d kill_errno=%d retained=%llu runtime_approved=false\n",label,
        (unsigned long long)o->native.cpu,(unsigned long long)o->native.wall,(unsigned long long)o->result->inclusive_max_ns.value,
        (unsigned long long)private_cost.ordinary_max,(unsigned long long)private_cost.last_unit,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,o->ticket.pid,o->group_kill_result,(unsigned long long)private_cost.retained_units);
    fixture_cleanup();fprintf(stderr,"probe refused: %s errno=%d\n",label,errno);exit(81);
}
#define reset fixture_reset_cost
#define require(ok,label) fixture_check((ok),(label))
static void fixture(int root) {
    memset(&outer,0,sizeof(outer));memset(&basis,0,sizeof(basis));
    basis.build.tools[0].ref.path=(struct private_slice){(const unsigned char *)"\"/source-known/git\"",sizeof("\"/source-known/git\"")-1};
    basis.build.tools[1].ref.path=(struct private_slice){(const unsigned char *)"\"/source-known/gpg\"",sizeof("\"/source-known/gpg\"")-1};
    basis.fixture.policy_branch=(struct private_slice){(const unsigned char *)"\"refs/heads/main\"",sizeof("\"refs/heads/main\"")-1};
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
static void argv_expansion_controls(int dir) {
    reset();fixture(dir);strcpy(outer.repo.path,"/source-known/fixture");
    const char *home="/source-known/home",*scratch="/source-known/scratch";
    require(!strcmp(private_forms[10].id,"tree_pin")&&private_forms[10].argc==2
        &&!strcmp(private_forms[10].argv[1],"{commit}^{tree}"),"tree pin registry retains original exact Git syntax");
    for(unsigned corrupt=0;corrupt<2;corrupt++) {
        strcpy(outer.case_name,corrupt?"corrupt_proof":"positive");struct engine_prepared prepared;
        int ready=engine_prepare_argv(&context,10,home,scratch,&prepared);
        char expected[64];int n=snprintf(expected,sizeof(expected),"%s^{tree}",corrupt?basis.fixture.corrupt:basis.fixture.valid);
        require(ready&&n==47&&prepared.argc==19&&prepared.argv[19]==NULL
            &&!strcmp(prepared.argv[17],"rev-parse")&&!strcmp(prepared.argv[18],expected)
            &&prepared.envc==11&&!strcmp(prepared.image,"/source-known/git"),
            "actual registry tree_pin argv preparation preserves selected commit and literal tree selector");
        require(private_end_unit(private_cost.open_cpu),"actual valid or corrupt tree argv preparation cost paid");
    }
    struct engine_prepared forbidden_prepared;
    require(!engine_prepare_argv(&context,15,home,scratch,&forbidden_prepared),"implicit verifier form15 remains forbidden to direct argv preparation");
    strcpy(outer.case_name,"positive");char out[4097];
    require(engine_expand(&context,"^{tree}",home,scratch,out)&&!strcmp(out,"^{tree}"),"only registered terminal tree peel remains literal Git syntax");
    const char *unknown[]={"{tre}","^{tre}","{tree}","^{commit}","^{blob}","^{tag}","^{}","{unknown}","a{tree}b","{}","{open","{outer{commit}}","{{commit}}","}","^{tree}suffix"};
    for(unsigned i=0;i<sizeof(unknown)/sizeof(unknown[0]);i++)
        require(!engine_expand(&context,unknown[i],home,scratch,out),"unregistered or malformed brace syntax refuses instead of copying unknown literals");
    char selected_record[128];int selected_len=snprintf(selected_record,sizeof(selected_record),"%s:second",basis.fixture.valid);
    require(selected_len==47&&engine_expand(&context,"{commit}:{record}",home,scratch,out)&&!strcmp(out,selected_record),"registered commit and record placeholders still compose exact colon syntax");
    const char *forbidden[]={"{git_signature_temp}","{signature}","x{signature}y","prefix{git_signature_temp}suffix","{outer{signature}}"};
    for(unsigned i=0;i<sizeof(forbidden)/sizeof(forbidden[0]);i++)
        require(!engine_expand(&context,forbidden[i],home,scratch,out),"B matcher and retired signature token remain banned even when embedded");
    require(private_end_unit(private_cost.open_cpu),"literal and forbidden brace vectors fully paid");
    char bounded[4098];memset(bounded,'x',4096);bounded[4096]=0;
    require(engine_expand(&context,bounded,home,scratch,out)&&strlen(out)==4096,"literal expansion exact4096 byte bound");
    bounded[4096]='x';bounded[4097]=0;
    require(!engine_expand(&context,bounded,home,scratch,out),"literal expansion refuses4097 bytes without truncation");
    bounded[4096]=0;
    require(engine_expand(&context,"{home}",bounded,scratch,out)&&strlen(out)==4096
        &&!engine_expand(&context,"{home}x",bounded,scratch,out),"named expansion retains exact4096 bound and refuses combined overflow");
    require(private_end_unit(private_cost.open_cpu),"bounded argv overflow controls fully paid");
    printf("private_argv_expansion_source_known actual_registry_form=10 prepared_valid=true prepared_corrupt=true selector_literal=true unknown_braces_refused=true vector_scope=production_prepare_and_expand native_process=false runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"argv source-known evidence fully paid");controls++;reset();
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
static unsigned flow_variant,flow_commands,flow_prepares,flow_prepared_forms;
static int source_known_current(const struct private_engine_context *ctx) {
    return private_context_same(ctx)&&private_guards.stage==2;
}
static int source_known_command(struct engine_owner *owner,struct engine_prepared *prepared,unsigned form,
    const char *home,const char *scratch,const unsigned char *input,size_t length,int *status) {
    ENGINE_FLOW_STAGE(1000+form);
    (void)input;(void)length;
    if(!engine_prepare_argv(owner->context,form,home,scratch,prepared))
        return engine_failure(owner,"input_invalid","actual fixed command argv preparation failed in source-known flow");
    flow_prepares++;flow_prepared_forms|=1u<<form;
    flow_commands++;owner->command=form;owner->verified_descendants=owner->child_created=owner->child_settled=0;
    unsigned slots=private_forms[form].members;owner->result->attempts+=(uint16_t)slots;
    owner->result->created+=(uint16_t)slots;owner->result->reaped+=(uint16_t)slots;
    if(owner->result->members_max<slots)owner->result->members_max=(uint8_t)slots;
    owner->result->group_gone=owner->result->retired_coverage=1;*status=0;
    // Scripted metadata for this low-level flow seam; no actual wait4 or
    // candidate terminal CPU observation is claimed by these counters.
    owner->result->terminal_cpu_checked++;owner->result->terminal_cpu_passed++;owner->result->helper_job_cpu_ns+=1000;
    struct private_engine_metric *metrics[]={&owner->result->ordinary_max_ns,&owner->result->controller_max_ns,&owner->result->cleanup_max_ns,
        &owner->result->native_cpu_max_ns,&owner->result->native_wall_max_ns,&owner->result->inclusive_max_ns,
        &owner->result->A_ns,&owner->result->C_ns,&owner->result->post_cpu_ns,
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
    // The native/total-CPU values above are SCRIPTED test records, never
    // observations of a candidate, source authentication or qualification.
    return private_end_unit(private_cost.open_cpu);
}
static void ref_for(struct private_ref *ref,const char *token,const void *bytes,size_t n) {
    memset(ref,0,sizeof(*ref));ref->path=(struct private_slice){(const unsigned char *)token,strlen(token)};
    ref->len=n;private_hash(bytes,n,ref->sha);
}
static void flow_failure_diagnostic(unsigned variant,int completed,int dispatch_error,const struct private_engine_result *r) {
    // Snapshot before formatter/payment can add any later error. These real
    // controller costs are separate from the explicitly scripted r metrics.
    const struct private_metrics failed=source_known_flow_failure_cost,now=private_cost;
    unsigned stage=source_known_flow_stage,first_stage=source_known_flow_failure_stage;
    const char *code=private_error,*detail=private_detail;uint64_t wall=UINT64_MAX,cpu=UINT64_MAX;
    int wall_known=known_clock_ns(CLOCK_MONOTONIC,&wall),cpu_known=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu);
    char line[4096];int n=snprintf(line,sizeof(line),"private_flow_failure_source_known variant=%u completed=%d scripted_commands=%u source_stage=%u first_failure_stage=%u private_failed=%d private_error_count=%llu error=<%.48s> detail=<%.320s> r_error=<%.48s> r_detail=<%.320s> phase=<%.32s> outcome=<%.32s> proof=<%.32s> engine_entered=%u semantic_forms=%u commands_completed=%u attempts=%u created=%u reaped=%u group_gone=%u retired_coverage=%u terminal_checked=%u terminal_passed=%u scripted_job_cpu_ns=%llu scripted_native_cpu=%u:%u:%llu scripted_native_wall=%u:%u:%llu scripted_inclusive=%u:%u:%llu signature_initialized=%u signature_ready=%u signature_bound=%u signature_quarantined=%u current_owner=%d guard_stage=%u dispatch_errno_unclassified=%d wall_known=%d wall_ns=%llu cpu_known=%d cpu_ns=%llu entry_ns=%llu deadline_ns=%llu open_cpu_ns=%llu last_unit_ns=%llu ordinary_max_ns=%llu debt_ns=%llu paid_ns=%llu rest_until_ns=%llu retained_raw_ns=%llu retained_cost_ns=%llu retained_units=%llu retained_adjustment_ns=%llu retained_overflow=%u rest_poll_cpu_ns=%llu first_fail_last_unit_ns=%llu first_fail_ordinary_max_ns=%llu first_fail_open_ns=%llu first_fail_debt_ns=%llu first_fail_paid_ns=%llu first_fail_retained_raw_ns=%llu actual_helper_births=0 native_metrics_scripted=true runtime_approved=false\n",
        variant,completed,flow_commands,stage,first_stage,private_failed,(unsigned long long)private_error_count,code?code:"null",detail?detail:"null",r->error_code?r->error_code:"null",r->error_detail?r->error_detail:"null",r->phase?r->phase:"null",r->outcome?r->outcome:"null",r->observed_proof?r->observed_proof:"null",r->engine_entered,r->semantic_forms,r->commands_completed,(unsigned)r->attempts,(unsigned)r->created,(unsigned)r->reaped,r->group_gone,r->retired_coverage,r->terminal_cpu_checked,r->terminal_cpu_passed,(unsigned long long)r->helper_job_cpu_ns,
        r->native_cpu_max_ns.observed,r->native_cpu_max_ns.known,(unsigned long long)r->native_cpu_max_ns.value,r->native_wall_max_ns.observed,r->native_wall_max_ns.known,(unsigned long long)r->native_wall_max_ns.value,r->inclusive_max_ns.observed,r->inclusive_max_ns.known,(unsigned long long)r->inclusive_max_ns.value,
        engine_signature_storage.initialized,engine_signature_storage.ready,engine_signature_storage.bound,engine_signature_storage.quarantined,engine_current!=NULL,private_guards.stage,dispatch_error,wall_known,(unsigned long long)wall,cpu_known,(unsigned long long)cpu,
        (unsigned long long)now.entry,(unsigned long long)now.deadline,(unsigned long long)now.open_cpu,(unsigned long long)now.last_unit,(unsigned long long)now.ordinary_max,(unsigned long long)now.debt,(unsigned long long)now.debt_paid,(unsigned long long)now.rest_until,(unsigned long long)now.retained_raw_ns,(unsigned long long)now.retained_cost_ns,(unsigned long long)now.retained_units,(unsigned long long)now.retained_adjustment_cost_ns,now.retained_overflow,(unsigned long long)now.rest_poll_cpu,
        (unsigned long long)failed.last_unit,(unsigned long long)failed.ordinary_max,(unsigned long long)failed.open_cpu,(unsigned long long)failed.debt,(unsigned long long)failed.debt_paid,(unsigned long long)failed.retained_raw_ns);
    if(n>0&&(size_t)n<sizeof(line))(void)write(STDERR_FILENO,line,(size_t)n);
    else{const char overflow[]="private_flow_failure_source_known diagnostic_overflow=true runtime_approved=false\n";(void)write(STDERR_FILENO,overflow,sizeof(overflow)-1);}
    // Keep the original failed assertion/verdict, but retain/pay all actual
    // diagnostic formatting/write/bookkeeping CPU before reaching it.
    (void)private_end_unit(private_cost.open_cpu);
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
        flow_variant=variant;flow_commands=flow_prepares=flow_prepared_forms=0;private_source_known_effects=&effects;
        struct private_engine_result r;memset(&r,0,sizeof(r));
        int completed=private_dispatch(1,1,1,&context,&r);int dispatch_error=errno;
        private_source_known_effects=NULL;
        uint64_t flow_end=clock_ns(CLOCK_MONOTONIC);
        if(completed!=(variant==0||variant==3)||flow_commands!=15||flow_prepares!=15||flow_prepared_forms!=32767u)flow_failure_diagnostic(variant,completed,dispatch_error,&r);
        require(completed==(variant==0||variant==3)&&flow_commands==15&&flow_prepares==15&&flow_prepared_forms==32767u,"same functional validate/claim/15-command/semantic/settle flow");
        printf("private_flow_cost_source_known variant=%u actual_argv_prepares=%u prepared_forms=%u entry_ns=%llu end_ns=%llu elapsed_ns=%llu original_deadline_ns=%llu ordinary_max_ns=%llu ordinary_cost_ns=%llu ordinary_raw_cpu_ns=%llu ordinary_floor_surcharge_ns=%llu booked_ordinary_units=%llu global_debt_ns=%llu global_paid_ns=%llu helper_cost_total_ns=%llu path_units=%llu retained_fixture_guards=%u actual_helper_births=0 native_wait4_metrics=scripted entry_scope=command_engine_only outer_inner_auth_platform_validation=not_exercised runtime_approved=false\n",
            variant,flow_prepares,flow_prepared_forms,(unsigned long long)private_cost.entry,(unsigned long long)flow_end,
            (unsigned long long)engine_interval(private_cost.entry,flow_end),(unsigned long long)private_cost.deadline,(unsigned long long)private_cost.ordinary_max,
            (unsigned long long)private_cost.booked_ordinary_cost_ns,(unsigned long long)private_cost.ordinary_raw_total_ns,(unsigned long long)private_cost.ordinary_floor_total_ns,
            (unsigned long long)private_cost.booked_units,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,
            (unsigned long long)private_cost.booked_helper_cost_ns,(unsigned long long)private_cost.path_walk_units,private_guards.count);
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"full15 engine source-known raw cost evidence paid under original deadline");
        require(flow_end!=UINT64_MAX&&flow_end>=private_cost.entry&&flow_end<private_cost.deadline&&private_cost.debt==private_cost.debt_paid,
            "full15 actual argv engine loop preserves original30s and paid ordinary floors");
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
    r->terminal_cpu_checked=r->terminal_cpu_passed=15;
    r->attempts=r->created=r->reaped=16;r->members_max=2;r->group_gone=r->retired_coverage=1;
    struct private_engine_metric *all[]={&r->ordinary_max_ns,&r->controller_max_ns,&r->cleanup_max_ns,&r->native_cpu_max_ns,
        &r->native_wall_max_ns,&r->inclusive_max_ns,&r->A_ns,&r->C_ns,&r->post_cpu_ns,
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
        if(variant==4)r.helper_job_cpu_ns=ENGINE_JOB_CPU_NS+1;
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
    reset();strcpy(outer.case_name,"positive");nominal_result(&r);r.terminal_cpu_passed=14;
    require(!engine_complete_result(&context,&r),"every completed actual command needs its own authoritative terminal CPU verdict");controls++;
    strcpy(outer.case_name,"positive");
}
static void final_conjunction_controls(int dir) {
    reset();struct engine_owner owner;struct private_engine_result result;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
    owner.ledger=openat(dir,"final-conjunction-controls.ndjson",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(owner.ledger>=0&&private_end_unit(private_cost.open_cpu),"owned final conjunction diagnostic FD prepared and paid");
    for(unsigned variant=0;variant<3;variant++) {
        reset();nominal_result(&result);strcpy(outer.case_name,"positive");
        if(variant==1){result.members_max=1;result.group_gone=0;}
        if(variant==2){result.inclusive_max_ns.known=0;result.helper_job_cpu_ns=ENGINE_JOB_CPU_NS+1;result.helper_group_rss_peak_bytes.known=0;}
        struct engine_final_conjunction c;engine_final_conjunction_capture(&context,&result,&c);
        off_t start=lseek(owner.ledger,0,SEEK_END);
        require(start>=0&&engine_final_conjunction_record(&owner,&c),"all final conjunction fields serialized on the original owned FD with full cost payment");
        char bytes[4096];ssize_t count=pread(owner.ledger,bytes,sizeof(bytes)-1,start);require(count>0,"read only the just-written owned diagnostic record");bytes[count]=0;
        int every=1;
        for(unsigned i=0;i<ENGINE_FINAL_TERMS;i++) {
            char needle[128];int n=snprintf(needle,sizeof(needle),"\"%s\":%s",engine_final_term_names[i],c.term[i]?"true":"false");
            if(n<=0||(size_t)n>=sizeof(needle)||!strstr(bytes,needle))every=0;
        }
        const char *first=variant==1?"members_max_complete":variant==2?"inclusive_within_3ms":"none";
        char needle[128];int n=snprintf(needle,sizeof(needle),"\"first_failing_term\":\"%s\"",first);
        every=every&&n>0&&(size_t)n<sizeof(needle)&&strstr(bytes,needle)&&!strcmp(c.first_failing_term,first)
            &&c.coverage_passed==(variant!=1)&&c.resources_passed==(variant!=2);
        require(private_end_unit(private_cost.open_cpu),"all diagnostic fields and later terms inspected with full cost payment");
        require(every,"final conjunction ledger contains every boolean term and first failing name");
    }
    int closed=0;require(private_close_observed(owner.ledger,&closed)&&closed,"original diagnostic FD closed once with full paid cost");owner.ledger=-1;
    printf("private_final_conjunction_API_source_known rows=3 terms=28 actual_owned_FD=true no_path_reopen=true all_cost_paid=true cutoff=before_this_diagnostic_format_write_fsync_close native_process=false runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"conjunction API completion evidence paid");controls++;
}

// Source-known fixture setup uses the SAME inclusive epoch and additive debt
// as the frozen probe. Stage clocks label work; they never replace open_cpu.
struct diagnostic_setup_step {
    const char *name;uint64_t start,end,epoch_before,epoch_after,observed_cpu_ns,raw_cpu_ns;
    uint64_t debt_before,debt_after,paid_after;int clocks_known,action_ok,ok;
};
static void diagnostic_setup_begin(struct diagnostic_setup_step *step,const char *name) {
    memset(step,0,sizeof(*step));step->name=name;step->epoch_before=private_cost.open_cpu;
    step->debt_before=private_cost.debt;step->clocks_known=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&step->start);
}
static int diagnostic_setup_end(struct diagnostic_setup_step *step) {
    uint64_t end=0;int known=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&end);
    step->end=end;step->clocks_known=step->clocks_known&&known&&end>=step->start;
    step->observed_cpu_ns=step->clocks_known?end-step->start:UINT64_MAX;
    int ok=private_end_unit(step->start);
    step->raw_cpu_ns=private_cost.last_unit;step->epoch_after=private_cost.open_cpu;
    step->debt_after=private_cost.debt;step->paid_after=private_cost.debt_paid;
    step->clocks_known=step->clocks_known&&step->raw_cpu_ns!=UINT64_MAX;
    step->ok=ok&&step->clocks_known;return step->ok;
}
static void diagnostic_setup_trace(unsigned variant,const struct diagnostic_setup_step *steps,unsigned count) {
    for(unsigned i=0;i<count;i++) {
        const struct diagnostic_setup_step *step=&steps[i];
        printf("private_diagnostic_setup_stage_source_known variant=%u stage=%s clocks_known=%d action_ok=%d guard_return=%d cpu_start_ns=%llu cpu_end_ns=%llu original_epoch_ns=%llu closed_epoch_ns=%llu observed_cpu_ns=%llu raw_cpu_ns=%llu debt_before_ns=%llu debt_after_ns=%llu paid_after_ns=%llu real_candidate_created=0 runtime_approved=false\n",
            variant,step->name,step->clocks_known,step->action_ok,step->ok,(unsigned long long)step->start,(unsigned long long)step->end,
            (unsigned long long)step->epoch_before,(unsigned long long)step->epoch_after,(unsigned long long)step->observed_cpu_ns,
            (unsigned long long)step->raw_cpu_ns,(unsigned long long)step->debt_before,(unsigned long long)step->debt_after,(unsigned long long)step->paid_after);
    }
}
static void diagnostic_setup_budget_control(void) {
    reset();struct diagnostic_setup_step step;diagnostic_setup_begin(&step,"actual-over-2ms-cpu");
    require(step.clocks_known&&step.start<=UINT64_MAX-2000000,"diagnostic setup negative has known finite beginning clock");
    uint64_t until=step.start+2000000,now=step.start;
    while(now<until)require(known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&now)&&now>=step.start,"diagnostic setup negative has known ordered CPU clocks");
    step.action_ok=1;int ok=diagnostic_setup_end(&step);
    int retained=!ok&&step.clocks_known&&step.observed_cpu_ns>=2000000&&step.raw_cpu_ns!=UINT64_MAX
        &&step.raw_cpu_ns>=step.observed_cpu_ns&&step.raw_cpu_ns<=UINT64_MAX/9
        &&step.debt_after>=step.debt_before&&step.debt_after-step.debt_before==9*step.raw_cpu_ns&&step.paid_after==step.debt_after;
    diagnostic_setup_trace(0,&step,1);
    printf("private_diagnostic_setup_budget_source_known guard_return=%d clocks_known=%d observed_cpu_ns=%llu raw_cpu_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu full_cost_retained=%s control_only=true real_candidate_created=0 runtime_approved=false whole_command_lifetime_proof=false\n",
        ok,step.clocks_known,(unsigned long long)step.observed_cpu_ns,(unsigned long long)step.raw_cpu_ns,
        (unsigned long long)step.debt_after,(unsigned long long)step.paid_after,retained?"true":"false");
    require(fflush(stdout)==0,"diagnostic setup negative raw evidence flush");
    require(retained,"diagnostic setup overrun retains full paid cost before refusal");
}
static void terminal_diagnostic_controls(int root) {
    for(unsigned variant=0;variant<4;variant++) {
        reset();char name[48];snprintf(name,sizeof(name),"diagnostic-%u",variant);
        struct diagnostic_setup_step steps[20];unsigned step_count=0;
        const char *setup_failure=NULL;int dir=-1,planted=-1,out=-1,err=-1,saved_out=-1,saved_err=-1;
        struct private_engine_result result;
#define DIAGNOSTIC_SETUP(name,action,failure) do { \
    require(step_count<20,"fixed diagnostic setup stage bound"); \
    struct diagnostic_setup_step *step=&steps[step_count++];diagnostic_setup_begin(step,(name)); \
    step->action_ok=(action);int stage_ok=diagnostic_setup_end(step); \
    if(!step->action_ok||!stage_ok){setup_failure=(failure);goto diagnostic_setup_failed;} \
} while(0)
        DIAGNOSTIC_SETUP("directory-create",mkdirat(root,name,0700)==0,"owned diagnostic output directory");
        DIAGNOSTIC_SETUP("directory-open",(dir=openat(root,name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC))>=0,"owned diagnostic output custody");
        outer.output.fd=dir;strcpy(outer.case_name,"positive");
        if(variant==0||variant==3) {
            DIAGNOSTIC_SETUP("collision-open",(planted=openat(dir,"result.json",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600))>=0,"actual exclusive publication collision open");
            DIAGNOSTIC_SETUP("collision-write",write(planted,"owned collision\n",16)==16,"actual exclusive publication collision write");
            DIAGNOSTIC_SETUP("collision-close",close(planted)==0,"actual exclusive publication collision close");planted=-1;
            DIAGNOSTIC_SETUP("causal-errors",(private_fail("cancelled","source-known causal control failure"),private_fail("coverage_unknown","source-known second independent control error"),1),"owned diagnostic causal error setup");
        }
        DIAGNOSTIC_SETUP("stdout-open",(out=openat(dir,"stdout",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600))>=0,"owned diagnostic stdout capture");
        DIAGNOSTIC_SETUP("stderr-open",(err=openat(dir,"stderr",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600))>=0,"owned diagnostic stderr capture");
        DIAGNOSTIC_SETUP("stdout-flush",fflush(stdout)==0,"owned diagnostic stdout flush");
        DIAGNOSTIC_SETUP("stderr-flush",fflush(stderr)==0,"owned diagnostic stderr flush");
        DIAGNOSTIC_SETUP("stdout-save",(saved_out=dup(STDOUT_FILENO))>=0,"owned diagnostic stdout descriptor pin");
        DIAGNOSTIC_SETUP("stderr-save",(saved_err=dup(STDERR_FILENO))>=0,"owned diagnostic stderr descriptor pin");
        if(variant==1) {
            DIAGNOSTIC_SETUP("stdout-readonly-close",close(out)==0,"owned readonly stdout setup");out=-1;
            DIAGNOSTIC_SETUP("stdout-readonly-open",(out=openat(dir,"stdout",O_RDONLY|O_NOFOLLOW|O_CLOEXEC))>=0,"actual readonly stdout negative");
        }
        DIAGNOSTIC_SETUP("stdout-redirect",dup2(out,STDOUT_FILENO)==STDOUT_FILENO,"owned diagnostic stdout redirection");
        DIAGNOSTIC_SETUP("stderr-redirect",dup2(err,STDERR_FILENO)==STDERR_FILENO,"owned diagnostic stderr redirection");
        DIAGNOSTIC_SETUP("nominal-result",(nominal_result(&result),private_terminal.control_known=1,private_terminal.control_ok=variant!=0&&variant!=3,1),"owned diagnostic nominal result setup");
#undef DIAGNOSTIC_SETUP
        int emitted=private_emit(&outer,input_hash,&result);
        int actual_closed=0;(void)private_close_observed(dir,&actual_closed);outer.output.fd=-1;
        source_known_tail_over_cpu=variant>=2;
        private_finish_diagnostics("settled",result.created,result.reaped,actual_closed);
        source_known_tail_over_cpu=0;
        require(dup2(saved_out,STDOUT_FILENO)==STDOUT_FILENO&&dup2(saved_err,STDERR_FILENO)==STDERR_FILENO
            &&close(saved_out)==0&&close(saved_err)==0,"restore exact diagnostic standard descriptors");
        saved_out=saved_err=-1;
        diagnostic_setup_trace(variant,steps,step_count);
        printf("private_terminal_diagnostic_conjuncts_source_known variant=%u emitted=%d control_ok=%u cost_ok=%u cost_unknown=%u persist_ok=%u persist_unknown=%u stdout_complete=%u cancelled_count=%llu coverage_count=%llu settlement_count=%llu resource_count=%llu first_error_cancelled=%d ordinary_max_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu real_candidate_created=0 runtime_approved=false\n",
            variant,emitted,private_terminal.control_ok,private_terminal.cost_ok,private_terminal.cost_unknown,private_terminal.persist_ok,
            private_terminal.persist_unknown,private_terminal.stdout_complete,(unsigned long long)private_errors[0].count,
            (unsigned long long)private_errors[2].count,(unsigned long long)private_errors[7].count,(unsigned long long)private_errors[5].count,
            !strcmp(private_error,"cancelled"),(unsigned long long)private_cost.ordinary_max,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
        require(fflush(stdout)==0,"bounded diagnostic setup and conjunct evidence flush");
        char log[32769];ssize_t n=pread(err,log,sizeof(log)-1,0);require(n>0&&(size_t)n<sizeof(log)-1,"bounded actual terminal diagnostic bytes");log[n]=0;
        const char *tail=strstr(log,"private_candidate_exit_tail "),*final=strstr(log,"private_candidate_terminal ");
        require(private_terminal.tail_complete&&tail&&final&&tail<final&&!strstr(final+1,"private_candidate_terminal "),
            "final raw verdict follows paid complete exit tail exactly once");
        require(private_terminal.persist_ok==(variant!=0&&variant!=3),"actual failed persistence remains independent of causal control");
        require(private_terminal.stdout_complete==(variant!=1),"actual stdout failure does not erase successful file persistence");
        if(variant==0) {
            require(!emitted&&!private_terminal.control_ok&&private_terminal.cost_ok
                &&private_errors[0].count&&private_errors[2].count&&private_errors[7].count&&!strcmp(private_error,"cancelled"),
                "bounded diagnostics retain later errors after first causal control failure");
            require(strstr(final,"control_verdict=failed persistence_verdict=not_persisted terminal_cost_verdict=passed")!=NULL,
                "causal and persistence failure axes remain complete while terminal cost passes");
        } else if(variant==1) {
            require(!emitted&&private_terminal.control_ok&&private_terminal.cost_ok&&private_errors[7].count
                &&strstr(final,"control_verdict=passed persistence_verdict=persisted terminal_cost_verdict=passed"),
                "stdout failure is explicit without falsifying control persistence or cost");
        } else if(variant==2) {
            require(emitted&&private_terminal.control_ok&&!private_terminal.cost_ok&&private_errors[5].count
                &&private_cost.ordinary_max>1000000&&private_cost.debt_paid==private_cost.debt
                &&strstr(final,"control_verdict=passed persistence_verdict=persisted terminal_cost_verdict=failed"),
                "actual over-CPU exit tail independently refuses after successful control and persistence");
        } else {
            require(!emitted&&!private_terminal.control_ok&&!private_terminal.persist_ok&&!private_terminal.cost_ok
                &&private_errors[0].count&&private_errors[2].count&&private_errors[7].count&&private_errors[5].count
                &&strstr(final,"control_verdict=failed persistence_verdict=not_persisted terminal_cost_verdict=failed"),
                "one refusal preserves all three failure axes and every later error class");
        }
        require(close(out)==0&&close(err)==0,"owned diagnostic captures close");
        printf("private_terminal_diagnostic_source_known variant=%u control=%s persistence=%s stdout_complete=%u terminal_cost=%s all_error_count=%llu tail_before_final=true real_candidate_created=0 runtime_approved=false\n",
            variant,private_terminal.control_ok?"passed":"failed",private_terminal.persist_ok?"passed":"failed",private_terminal.stdout_complete,
            private_terminal.cost_ok?"passed":"failed",(unsigned long long)private_error_count);controls++;
        continue;
diagnostic_setup_failed:
        if(saved_out>=0){require(dup2(saved_out,STDOUT_FILENO)==STDOUT_FILENO&&close(saved_out)==0,"restore diagnostic stdout after setup refusal");saved_out=-1;}
        if(saved_err>=0){require(dup2(saved_err,STDERR_FILENO)==STDERR_FILENO&&close(saved_err)==0,"restore diagnostic stderr after setup refusal");saved_err=-1;}
        diagnostic_setup_trace(variant,steps,step_count);
        printf("private_terminal_diagnostic_setup_refused_source_known variant=%u completed_stages=%u ordinary_max_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu real_candidate_created=0 runtime_approved=false\n",
            variant,step_count,(unsigned long long)private_cost.ordinary_max,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
        require(fflush(stdout)==0,"diagnostic setup refusal raw evidence flush");
        require(0,setup_failure?setup_failure:"diagnostic setup stage refused");
    }
    reset();
}

// Actual file/FD and stdout APIs; no helper or real candidate is created.
static void terminal_deadline_controls(int root) {
    reset();int original=-1,replacement=-1;struct stat admitted;char expected[65];
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    original=openat(root,"admitted-descriptor",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(original>=0&&write(original,"original\n",9)==9&&fstat(original,&admitted)==0,"owned immutable descriptor admission");
    private_hash("original\n",9,expected);require(private_end_unit(at),"descriptor admission cost paid");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(renameat(root,"admitted-descriptor",root,"retained-descriptor")==0,"actual original locator rename");
    replacement=openat(root,"admitted-descriptor",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(replacement>=0&&write(replacement,"different",9)==9,"actual different inode at original locator");
    require(private_end_unit(at),"actual locator replacement cost paid");
    int current=private_admitted_descriptor_current(original,&admitted,expected);
    require(current,"admitted original FD remains current when only its pathname locator is replaced");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);require(pwrite(original,"changed!\n",9,0)==9,"actual same-inode descriptor mutation");
    require(private_end_unit(at),"actual same-inode mutation cost paid");
    current=private_admitted_descriptor_current(original,&admitted,expected);
    require(!current&&private_errors[9].count,"admitted FD fresh hash refuses actual in-place mutation despite identical length");
    require(private_close(original)&&private_close(replacement),"exact descriptor fixture FD disposal paid");
    controls++;

    reset();at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(mkdirat(root,"bounded-walk",0700)==0,"owned bounded path root");
    int nested=openat(root,"bounded-walk",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
    require(nested>=0&&mkdirat(nested,"a",0700)==0,"owned bounded path middle");
    int middle=openat(nested,"a",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
    require(middle>=0&&mkdirat(middle,"b",0700)==0,"owned bounded path leaf");
    require(close(nested)==0&&close(middle)==0&&private_end_unit(at),"bounded walk fixture preparation paid");
    int walked=private_open_walk(root,"bounded-walk/a/b",0,O_RDONLY|O_DIRECTORY);
    require(walked>=0&&private_open_last.components==3&&private_open_last.units<private_open_last.components
        &&private_cost.debt==private_cost.debt_paid,"one bounded path walk pays actual work without a floor per tiny component");
    require(private_close(walked),"original walked FD disposal paid");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);int held=openat(root,"retained-descriptor",O_RDONLY|O_NOFOLLOW|O_CLOEXEC);struct stat stamp;
    require(held>=0&&fstat(held,&stamp)==0&&private_end_unit(at),"owned currentness guard setup paid");
    uint64_t units_before=private_cost.closed_units;
    require(private_named_current(held,root,"retained-descriptor",0,&stamp)
        &&private_cost.closed_units-units_before==1&&private_cost.debt==private_cost.debt_paid,
        "same bounded currentness open stat and close share one fully paid unit");
    require(private_close(held),"original currentness fixture held descriptor disposed");
    private_cost.deadline=clock_ns(CLOCK_MONOTONIC);
    int admitted_more=private_end_unit(private_cost.open_cpu);
    require(!admitted_more&&private_cost.deadline_expired&&private_errors[3].count
        &&private_cost.debt==private_cost.debt_paid&&!private_cost.reporting_only,"expired original work deadline forbids next work after full cost payment");
    int missing=private_open_walk(root,"bounded-walk/a/b",0,O_RDONLY|O_DIRECTORY);
    int classified=private_open_failure("source_changed","source-known path classification");
    require(missing<0&&!classified&&private_open_last.budget_refused&&!private_open_last.native_failed&&!private_errors[9].count,
        "paid deadline refusal is not fabricated pathname-change evidence");
    controls++;

    reset();at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    require(mkdirat(root,"deadline-result",0700)==0,"owned deadline refusal result directory");
    int dir=openat(root,"deadline-result",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
    require(dir>=0&&private_end_unit(at),"deadline result directory custody paid");
    at=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    int out=openat(dir,"stdout",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(out>=0&&fflush(stdout)==0,"owned deadline stdout capture");int saved=dup(STDOUT_FILENO);
    require(saved>=0&&dup2(out,STDOUT_FILENO)==STDOUT_FILENO&&private_end_unit(at),"deadline stdout redirection paid");
    int old_output=outer.output.fd;outer.output.fd=dir;
    struct private_engine_result result;memset(&result,0,sizeof(result));result.phase="validate";result.outcome="refused";
    result.observed_proof="not_reached";result.group_gone=result.retired_coverage=1;
    private_cost.deadline=clock_ns(CLOCK_MONOTONIC);
    require(!private_end_unit(private_cost.open_cpu),"actual original deadline refuses ordinary work before result publication");
    int emitted=private_emit(&outer,input_hash,&result);
    int actual_closed=0;(void)private_close_observed(dir,&actual_closed);outer.output.fd=old_output;
    private_finish_diagnostics("settled",0,0,actual_closed);
    require(dup2(saved,STDOUT_FILENO)==STDOUT_FILENO&&close(saved)==0,"restore original stdout after deadline refusal");
    require(!emitted&&private_failed&&!private_terminal.control_ok&&private_errors[3].count
        &&private_cost.reporting_only&&private_terminal.stdout_attempted&&private_terminal.stdout_complete
        &&private_terminal.persist_ok&&private_terminal.format_paid&&private_terminal.persistence_paid&&private_terminal.write_paid
        &&private_terminal.cost_ok&&!private_terminal.cost_unknown&&!private_errors[7].count
        &&private_cost.debt==private_cost.debt_paid,"expired deadline still attempts one bounded fully paid refusal result and persistence");
    char bytes[8192];ssize_t n=pread(out,bytes,sizeof(bytes)-1,0);require(n>0&&n<(ssize_t)sizeof(bytes)-1,"actual deadline refusal stdout is bounded");bytes[n]=0;
    require(strstr(bytes,"\"code\":\"deadline\"")&&strstr(bytes,"\"runtime_approved\":false"),"actual deadline result preserves refusal and runtime false");
    require(!private_dispatch(1,1,1,&context,&result),"terminal reporting cannot admit a new engine command");
    require(private_close(out),"deadline stdout capture disposal paid");controls++;
    printf("private_terminal_deadline_source_known cases=3 actual_files=true actual_stdout=true helpers=0 deadline_preserved=true runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"deadline control bounded evidence paid");reset();
}

static void hold_observer_checkpoint(void) {
    if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)
        require(private_end_unit(private_cost.open_cpu),"source-known holder observer ordinary unit and rest");
}
struct hold_witness_info {pid_t helper;uint64_t birth;int fd;uint64_t dev,ino,deadline;};
struct hold_fixture_cleanup_state {
    pid_t controller,helper;int controller_reaped,controller_status,controller_status_known,release_fd,identity_known;
    unsigned release_path,release_closed;ssize_t release_write_result;int release_write_errno,release_close_result,release_close_errno;
    uint64_t birth,start,deadline;audit_token_t token;
};
static struct hold_fixture_cleanup_state hold_fixture_cleanup={.release_fd=-1};
static int hold_fixture_release(int send_byte) {
    struct hold_fixture_cleanup_state *s=&hold_fixture_cleanup;
    if(s->release_fd<0)return 0;
    int fd=s->release_fd;s->release_path=send_byte?1:2;s->release_write_result=-2;s->release_write_errno=0;
    if(send_byte){errno=0;s->release_write_result=write(fd,"x",1);s->release_write_errno=s->release_write_result<0?errno:0;}
    // Consuming this original descriptor is attempted once. A failed close
    // remains unknown; never retry a numeric FD after its possible release.
    s->release_fd=-1;errno=0;s->release_close_result=close(fd);s->release_close_errno=s->release_close_result<0?errno:0;
    s->release_closed=s->release_close_result==0;
    return s->release_closed&&(!send_byte||s->release_write_result==1);
}
static pid_t hold_fixture_wait(int *status) {
    struct hold_fixture_cleanup_state *s=&hold_fixture_cleanup;
    if(s->controller_reaped){*status=s->controller_status;return s->controller;}
    pid_t joined=waitpid(s->controller,status,WNOHANG);
    if(joined==s->controller) {
        // A consuming wait ends the original PID ownership immediately,
        // including on an assertion's failure path. Never signal it again.
        if(retained_child==joined)retained_child=0;
        s->controller_reaped=s->controller_status_known=1;s->controller_status=*status;
    } else if(joined<0&&errno==ECHILD) {
        if(retained_child==s->controller)retained_child=0;
    }
    return joined;
}
static void hold_fixture_require_alive(int *status,const char *stage) {
    pid_t joined=hold_fixture_wait(status);int error=joined<0?errno:0;
    if(joined==0)return;
    const struct hold_fixture_cleanup_state *s=&hold_fixture_cleanup;
    fprintf(stderr,"private_hold_controller_liveness_source_known stage=%s controller=%d wait_result=%d wait_errno=%d controller_reaped=%d controller_status_known=%d controller_status_raw=%d retained_controller_pid=%d release_gate_held=%d runtime_approved=false\n",
        stage,s->controller,joined,error,s->controller_reaped,s->controller_status_known,s->controller_status,retained_child,s->release_fd>=0);
    require(0,"same controller remains alive until external fixture release");
}
static void hold_fixture_failure_cleanup(void) {
    struct hold_fixture_cleanup_state *s=&hold_fixture_cleanup;
    dispose_private_engine=NULL;
    if(s->release_fd>=0) {
        // EOF releases only this fixture's existing one-byte gate. The
        // original controller can reap normally; an orphaned negative helper
        // also exits. No PID/group signal or invented wait4 follows.
        (void)hold_fixture_release(0);
    }
    int terminal=0,gone=0,status=0,expired=0,group_error=0;
    struct engine_incarnation original={.pid=s->helper,.birth=s->birth,.start=s->start,.token=s->token,.current=1};
    for(;;) {
        if(!s->controller_reaped) {
            pid_t joined=hold_fixture_wait(&status);
            if(joined<0&&errno!=EINTR)break;
        }
        if(s->identity_known)terminal=engine_physical_exit(&original);
        int pids[3]={0},count=-1;if(s->helper>0)count=native_group_count(s->helper,pids,sizeof(pids),&group_error);gone=s->helper>0&&native_pid_list_empty(count,group_error);
        if(s->controller_reaped&&terminal&&gone)break;
        if(s->controller_reaped&&!s->identity_known)break; // Unknown identity cannot improve by waiting.
        uint64_t now;if(!known_clock_ns(CLOCK_MONOTONIC,&now)||now>=s->deadline){expired=1;break;}
        uint64_t cpu;if(known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu)&&private_cost.open_cpu!=UINT64_MAX
            &&cpu>=private_cost.open_cpu&&cpu-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
        usleep(1000);
    }
    (void)private_end_unit(private_cost.open_cpu);
    fprintf(stderr,"private_hold_fixture_cleanup_source_known controller=%d controller_reaped=%d controller_status_known=%d controller_status_raw=%d retained_controller_pid=%d helper=%d helper_birth=%llu helper_start=%llu helper_identity_known=%d original_helper_terminal_observed=%d original_group_gone=%d original_group_errno=%d release_gate_closed=%u release_path=%s release_write_result=%lld release_write_errno=%d release_close_result=%d release_close_errno=%d signals_sent=0 helper_reaped_by_observer=false deadline_exhausted=%d debt_charged_ns=%llu debt_paid_ns=%llu runtime_approved=false\n",
        s->controller,s->controller_reaped,s->controller_status_known,s->controller_status,retained_child,s->helper,
        (unsigned long long)s->birth,(unsigned long long)s->start,s->identity_known,terminal,gone,group_error,s->release_closed,
        s->release_path==1?"normal_byte_close":s->release_path==2?"failure_eof_close":"unattempted",(long long)s->release_write_result,s->release_write_errno,s->release_close_result,s->release_close_errno,expired,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    // A missing identity/status stays explicitly unknown. In particular do
    // not fall through to generic numeric cleanup after a reaped controller.
}
static void quarantine_hold_process_control(const char *self,const char *root,int dir) {
    reset();
    require(!engine_current&&!dispose_private_engine,"fixture observer starts without another private owner");
    int wire[2],release[2],status_pipe[2];require(pipe(wire)==0&&pipe(release)==0&&pipe(status_pipe)==0,"owned live-holder fixture channels");
    // The child redirects stdout to the result pipe and later exits normally;
    // flush only this fixture's inherited stdio buffers before that fork.
    require(fflush(stdout)==0&&fflush(stderr)==0,"source-known observer stdio boundary before controller fork");
    pid_t controller=fork();require(controller>=0,"owned source-known controller fork");
    if(controller==0) {
        close(wire[0]);close(release[1]);close(status_pipe[0]);
        require(dup2(wire[1],STDOUT_FILENO)==STDOUT_FILENO&&close(wire[1])==0,"owned refusal wire");
        reset();private_cost.original_parent=getppid();engine_owner_loss_signal=engine_cancel_signal=0;
        struct sigaction action;memset(&action,0,sizeof(action));sigemptyset(&action.sa_mask);
        action.sa_handler=SIG_IGN;require(sigaction(SIGPIPE,&action,NULL)==0,"source-known refusal pipe disposition");
        action.sa_handler=engine_owner_loss_handler;require(sigaction(SIGHUP,&action,NULL)==0,"source-known actual owner-loss signal disposition");
        action.sa_handler=engine_cancel_handler;require(sigaction(SIGTERM,&action,NULL)==0,"source-known held cancellation disposition");
        // This original200ms work deadline is fixed BEFORE any helper effect,
        // inside the unchanged30s maximum; it is never reset or extended.
        outer.deadline=private_cost.deadline=private_cost.entry+200000000ULL;
        outer.output.fd=dir;strcpy(outer.output.path,root);outer.prefix.fd=outer.repo.fd=dir;
        strcpy(outer.run,"00000000000000000000000001");strcpy(outer.nonce,"00000000000000000000000002");outer.incarnation=1;
        memset(outer.source,'a',64);outer.source[64]=0;memset(outer.exe,'b',64);outer.exe[64]=0;strcpy(outer.case_name,"positive");
        struct private_file *pins[]={&outer.source_lock,&outer.build,&outer.review,&outer.closure,&outer.config,&outer.commands,&outer.git,&outer.gpg};
        for(unsigned i=0;i<8;i++){memset(pins[i]->sha,'a'+(i%6),64);pins[i]->sha[64]=0;}
        struct private_engine_result r;memset(&r,0,sizeof(r));r.phase="git_reads";r.outcome="refused";r.observed_proof="not_reached";
        require(engine_original_claim(&context,&r),"same-controller original claim before its effect");
        int sentinel=openat(dir,"hold-fd-sentinel",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(sentinel>=0,"source-known retained original FD");struct stat identity;require(fstat(sentinel,&identity)==0,"source-known sentinel identity");
        struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&r;
        owner.controller_parent=getppid();owner.input=owner.output=owner.errors=-1;
        owner.ledger=openat(dir,"hold-ledger.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(owner.ledger>=0,"source-known holder ledger");
        owner.slots=1;owner.source_admitted=1;owner.ticket.sequence=1;r.attempts=1;
        struct engine_prepared p;memset(&p,0,sizeof(p));for(unsigned i=0;i<2;i++)p.input[i]=p.output[i]=p.errors[i]=-1;
        strcpy(p.image,self);strcpy(p.descendant,self);strcpy(p.cwd,root);strcpy(owner.image,self);strcpy(owner.descendant,self);
        require(engine_put(&p,self,&p.argv[p.argc++])&&engine_put(&p,"hold-gated-child",&p.argv[p.argc++]),"bounded held helper argv");p.argv[p.argc]=NULL;
        require(engine_put(&p,"LANG=C",&p.env[p.envc++]),"bounded held helper environment");p.env[p.envc]=NULL;
        require(engine_prepare_effect(&owner,&p,0)&&posix_spawn_file_actions_adddup2(&p.actions,release[0],0)==0&&private_end_unit(private_cost.open_cpu),"held helper exact background preparation");
        engine_current=&owner;dispose_private_engine=engine_emergency_dispose;
        require(engine_birth(&owner,&p),"actual held helper unchanged native gates and observed priority4");engine_dispose_prepared(&p);
        pid_t helper=owner.ticket.pid;struct image first=owner.members[0].state;close(release[0]);owner.input=sentinel;
        while(clock_ns(CLOCK_MONOTONIC)<outer.deadline) {
            if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
            usleep(1000);
        }
        source_known_final_kill_eperm=1;require(engine_kill_group(&owner),"held actual helper retains injected EPERM final kill");source_known_final_kill_eperm=0;
        require(!engine_quarantine(&owner,"source-known live helper after original deadline and EPERM final kill")&&owner.settlement_quarantined&&!owner.direct_reaped,"actual unresolved final kill enters owned quarantine without retry");
        struct hold_witness_info info={helper,first.birth,sentinel,(uint64_t)identity.st_dev,(uint64_t)identity.st_ino,outer.deadline};
        require(write(status_pipe[1],&info,sizeof(info))==sizeof(info)&&close(status_pipe[1])==0,"source-known holder observation identity");
        (void)private_emit(&outer,input_hash,&r);
        require(owner.refusal_attempted&&owner.refusal_complete,"same controller emits the existing refusal before holding");
        source_known_schedule_interrupts=1;require(!engine_checkpoint(&owner)&&private_has_retained_cost(),"actual interrupted hold unit retains its full closed cost");
        unsigned observed_before=owner.hold_rounds;source_known_schedule_interrupts=1;
        require(!engine_hold_observe(&owner)&&owner.hold_rounds==observed_before&&private_has_retained_cost(),"interrupted prior payment forbids every new hold observation");
        engine_quarantine_hold(&owner);
        require(owner.quarantine_settled&&owner.direct_reaped&&r.reaped==1&&r.group_gone&&r.retired_coverage
            &&owner.owner_lost&&owner.owner_lost_recorded&&!strcmp(r.outcome,"refused")&&owner.group_kill_attempted==1&&owner.group_kill_result==EPERM&&owner.group_kill_raced_exit,
            "same held controller settles itself after release and remains refused/owner-lost");
        require(fcntl(sentinel,F_GETFD)>=0,"original FD remains held through actual terminal proof");
        engine_close_transferred(&owner);require(private_close(owner.ledger),"settled holder ledger close");owner.ledger=-1;
        int marker=openat(dir,"hold-control-success",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(marker>=0&&write(marker,"1",1)==1&&close(marker)==0&&private_end_unit(private_cost.open_cpu),"held controller reaches every fixture assertion before its terminal receipt");
        int output_closed=0;(void)private_close_observed(dir,&output_closed);outer.output.fd=-1;
        private_finish_diagnostics("settled",r.created,r.reaped,output_closed);exit(81);
    }
    retained_child=controller;
    memset(&hold_fixture_cleanup,0,sizeof(hold_fixture_cleanup));hold_fixture_cleanup.controller=controller;
    hold_fixture_cleanup.release_write_result=hold_fixture_cleanup.release_close_result=-2;
    hold_fixture_cleanup.release_fd=release[1];hold_fixture_cleanup.deadline=private_cost.deadline;
    dispose_private_engine=hold_fixture_failure_cleanup;
    require(private_end_unit(private_cost.open_cpu),"source-known observer fork preparation closes its ordinary epoch");
    close(wire[1]);close(release[0]);close(status_pipe[1]);
    require(fcntl(wire[0],F_SETFL,O_NONBLOCK)==0&&fcntl(status_pipe[0],F_SETFL,O_NONBLOCK)==0,"owned holder observer channels nonblocking");
    uint64_t observer_deadline=clock_ns(CLOCK_MONOTONIC)+30000000000ULL;
    hold_fixture_cleanup.deadline=observer_deadline;
    char refusal[8193];size_t used=0;int got_line=0;
    while(clock_ns(CLOCK_MONOTONIC)<observer_deadline&&!got_line) {
        ssize_t n=read(wire[0],refusal+used,sizeof(refusal)-1-used);
        if(n>0){used+=(size_t)n;got_line=memchr(refusal,'\n',used)!=NULL;}
        else require(n<0&&(errno==EAGAIN||errno==EWOULDBLOCK||errno==EINTR),"held controller cannot exit/EOF before prompt refusal");
        require(used<sizeof(refusal)-1,"bounded existing refusal wire");hold_observer_checkpoint();if(!got_line)usleep(1000);
    }
    refusal[used]=0;uint64_t received=clock_ns(CLOCK_MONOTONIC);
    require(private_end_unit(private_cost.open_cpu),"owned refusal observation closes before bounded canonical decoding");
    require(got_line&&refusal[used-1]=='\n'&&private_canonical((unsigned char *)refusal,used-1)
        &&strstr(refusal,"\"phase\":\"quarantined\"")&&strstr(refusal,"\"runtime_approved\":false")
        &&strstr(refusal,"\"created\":1")&&strstr(refusal,"\"reaped\":0")&&strstr(refusal,"\"group_gone\":false"),
        "prompt existing-wire refusal precedes release/settlement");
    struct hold_witness_info info;size_t have=0;
    while(have<sizeof(info)&&clock_ns(CLOCK_MONOTONIC)<observer_deadline) {
        ssize_t n=read(status_pipe[0],(char *)&info+have,sizeof(info)-have);if(n>0)have+=(size_t)n;
        else require(n<0&&(errno==EAGAIN||errno==EINTR),"holder identity observation pending");hold_observer_checkpoint();if(have<sizeof(info))usleep(1000);
    }
    require(have==sizeof(info)&&received>=info.deadline,"refusal follows this unchanged original work deadline");
    hold_fixture_cleanup.helper=info.helper;hold_fixture_cleanup.birth=info.birth;
    struct rusage_info_v1 original_usage;
    hold_fixture_cleanup.identity_known=known_control_token(info.helper,info.birth,&hold_fixture_cleanup.token)
        &&proc_pid_rusage(info.helper,RUSAGE_INFO_V1,(rusage_info_t *)&original_usage)==0&&original_usage.ri_proc_start_abstime!=0;
    if(hold_fixture_cleanup.identity_known)hold_fixture_cleanup.start=original_usage.ri_proc_start_abstime;
    int status=0;hold_fixture_require_alive(&status,"post_refusal");
    struct proc_bsdinfo helper;require(proc_pidinfo(info.helper,PROC_PIDTBSDINFO,0,&helper,sizeof(helper))==sizeof(helper)
        &&helper.pbi_ppid==(uint32_t)controller&&helper.pbi_pgid==(uint32_t)info.helper&&helper.pbi_status!=SZOMB,
        "independent observer sees original live helper still owned by this controller");
    struct vnode_fdinfowithpath held_fd;require(proc_pidfdinfo(controller,info.fd,PROC_PIDFDVNODEPATHINFO,&held_fd,sizeof(held_fd))==sizeof(held_fd)
        &&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_dev==info.dev&&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_ino==info.ino,
        "independent kernel FD snapshot proves original descriptor retained after refusal");
    require(kill(controller,SIGHUP)==0,"actual source-known session-loss stimulus targets only owned controller");
    int ledger=openat(dir,"hold-ledger.ndjson",O_RDONLY|O_NOFOLLOW|O_CLOEXEC);require(ledger>=0,"read exact owned holder ledger");
    char log[8193];int lost=0;
    while(!lost&&clock_ns(CLOCK_MONOTONIC)<observer_deadline) {
        ssize_t n=pread(ledger,log,sizeof(log)-1,0);require(n>=0,"held owner-loss ledger observation");log[n]=0;lost=strstr(log,"\"phase\":\"owner_lost\"")!=NULL;
        hold_observer_checkpoint();
        if(!lost){hold_fixture_require_alive(&status,"owner_loss_pending");usleep(1000);}
    }
    require(lost,"same held controller records owner loss before the unchanged observation deadline");
    hold_fixture_require_alive(&status,"owner_loss_recorded");
    // This second independent snapshot is causally AFTER the holder entered
    // its observation loop, catching FD release after the prompt wire write.
    require(proc_pidfdinfo(controller,info.fd,PROC_PIDFDVNODEPATHINFO,&held_fd,sizeof(held_fd))==sizeof(held_fd)
        &&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_dev==info.dev&&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_ino==info.ino,
        "independent kernel FD snapshot proves original descriptor retained after refusal");
    hold_fixture_require_alive(&status,"before_external_release");
    require(hold_fixture_release(1),"external fixture release follows custody observations; controller sends no new stdin");
    pid_t joined=0;
    while(!joined&&clock_ns(CLOCK_MONOTONIC)<observer_deadline){joined=hold_fixture_wait(&status);require(joined>=0,"owned holder join observation");hold_observer_checkpoint();if(!joined)usleep(1000);}
    require(joined==controller&&WIFEXITED(status)&&WEXITSTATUS(status)==81,"same controller exits only after its own settled refusal");retained_child=0;
    ssize_t n=read(wire[0],refusal,sizeof(refusal));require(n==0,"holder emits exactly one existing result, never a second success");
    n=pread(ledger,log,sizeof(log)-1,0);require(n>=0,"settled refusal ledger read");log[n]=0;
    require(strstr(log,"\"phase\":\"settled_refusal_owner_lost\"")!=NULL,"same controller records settled refusal without upgrading the result");
    int group[3],group_error=0;int group_count=native_group_count(info.helper,group,sizeof(group),&group_error);
    require(native_pid_list_empty(group_count,group_error),"independent actual helper group gone after own-controller reap");
    int marker=openat(dir,"hold-control-success",O_RDONLY|O_NOFOLLOW|O_CLOEXEC);char success=0;
    require(marker>=0&&read(marker,&success,1)==1&&success=='1'&&close(marker)==0,"same controller completed every custody and paid-observation assertion");
    dispose_private_engine=NULL;
    require(close(ledger)==0&&close(wire[0])==0&&close(status_pipe[0])==0,"owned live-holder observer close");
    printf("private_quarantine_hold_source_known controller=%d helper=%d original_deadline=%llu refusal_received=%llu prompt_before_release=true controller_alive_after_refusal=true original_fd_retained=true child_parent_pin_retained=true owner_lost_recorded=true released_then_controller_own_reap=true group_gone=true exit=81 result_count=1 runtime_approved=false\n",
        controller,info.helper,(unsigned long long)info.deadline,(unsigned long long)received);controls++;
}

static void charge_diagnostic_control(int dir) {
    reset();outer.deadline=private_cost.deadline;
    struct private_engine_result result;struct engine_owner owner;memset(&result,0,sizeof(result));memset(&owner,0,sizeof(owner));
    owner.context=&context;owner.result=&result;owner.ledger=openat(dir,"charge-diagnostic.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(owner.ledger>=0&&private_end_unit(private_cost.open_cpu),"ordinary charge diagnostic bounded preparation");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);while(clock_ns(CLOCK_THREAD_CPUTIME_ID)-at<1100000){}
    require(!engine_charge(&owner,0,0,1)&&owner.failed_units==1&&owner.failed_unit_first.actual>=1100000
        &&owner.failed_unit_first.inclusive==owner.failed_unit_first.actual&&private_cost.debt==private_cost.debt_paid,
        "actual ordinary CPU tail is retained charged paid and diagnosed before refusal");
    uint64_t first=owner.failed_unit_first.actual;
    require(!engine_charge(&owner,0,1500001,1)&&owner.failed_units==2&&owner.failed_unit_last.actual<1000000
        &&owner.failed_unit_last.supplied_cost==1500001&&owner.failed_unit_first.actual==first
        &&private_cost.debt==private_cost.debt_paid,
        "cost-only ordinary refusal remains distinct from actual controller CPU overrun");
    require(engine_record(&owner,"charge_diagnostic")&&fsync(owner.ledger)==0&&close(owner.ledger)==0,"ordinary first-last failed charge evidence persisted");
    require(private_end_unit(private_cost.open_cpu),"ordinary charge diagnostic raw formatter persistence and disposal are paid");
    printf("private_charge_diagnostic_source_known failures=2 first_actual_ns=%llu last_actual_ns=%llu last_supplied_cost_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu helpers=0 runtime_approved=false\n",
        (unsigned long long)first,(unsigned long long)owner.failed_unit_last.actual,(unsigned long long)owner.failed_unit_last.supplied_cost,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"charge diagnostic stdout formatting and actual flush are paid before reset");
    controls++;reset();
}
static uint64_t burn_cpu_work(uint64_t amount) {
    uint64_t start=clock_ns(CLOCK_PROCESS_CPUTIME_ID);volatile uint64_t value=1;
    while(clock_ns(CLOCK_PROCESS_CPUTIME_ID)-start<amount)for(unsigned i=0;i<256;i++)value=value*33+i;
    return value;
}
static void burn_cpu(uint64_t amount) {uint64_t value=burn_cpu_work(amount);_exit(value==0?1:0);}
static void pause_bypass_child(const char *start,const char *done,const char *release) {
    while(access(start,F_OK)!=0)usleep(1000);
    uint64_t value=burn_cpu_work(8000000);int fd=open(done,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    if(fd<0||close(fd)!=0)_exit(77);
    while(access(release,F_OK)!=0)usleep(1000);_exit(value==0?1:0);
}
static void fixture_touch(const char *path) {
    int fd=open(path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(fd>=0&&close(fd)==0,"exact owned external-pause fixture marker");
}
static void rollup_child(const char *self,const char *report) {
    pid_t pid=fork();if(pid<0)_exit(77);
    if(pid==0) {raise(SIGSTOP);burn_cpu(260000000);}
    FILE *out=fopen(report,"wx");if(!out)_exit(77);
    if(fprintf(out,"%d\n",pid)<0||fflush(out))_exit(77);raise(SIGSTOP);
    // The same helper parent retains its own terminal child until this
    // fixture's controller has observed exact positive exited RI counters.
    // Only this helper parent calls child wait4; no adoption is introduced.
    siginfo_t terminal;memset(&terminal,0,sizeof(terminal));
    if(waitid(P_PID,pid,&terminal,WEXITED|WNOWAIT)!=0||terminal.si_pid!=pid)_exit(77);
    char ready[4097],release[4097];snprintf(ready,sizeof(ready),"%s-terminal",report);snprintf(release,sizeof(release),"%s-release",report);
    int marker=open(ready,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);if(marker<0||close(marker)!=0)_exit(77);
    while(access(release,F_OK)!=0)usleep(1000);
    int status=0;struct rusage r;pid_t got=wait4(pid,&status,0,&r);if(got!=pid)_exit(77);
    if(fprintf(out,"%llu\n",(unsigned long long)ru_cpu(&r))<0||fclose(out))_exit(77);
    (void)self;_exit(0);
}
static int fixture_total_argument(unsigned mode,const char *root,char argument[4097],char release[4097]) {
    int n;release[0]=0;
    if(mode==9||mode==10)n=snprintf(argument,4097,"%s/pause-%u-start",root,mode);
    else if(mode==2||mode==6)n=snprintf(argument,4097,"%s/%s-%u",root,mode==2?"rollup":"release",mode);
    else if(mode>=16&&mode<=22)n=snprintf(argument,4097,"%s/live-race-%u",root,mode);
    else if(mode>=23&&mode<=25)n=snprintf(argument,4097,"%s/event-release-%u",root,mode);
    else if(mode==26)n=snprintf(argument,4097,"%s/knote-rollup-%u",root,mode);
    else if(mode==27)n=snprintf(argument,4097,"%s",root);
    else n=snprintf(argument,4097,"%llu",(unsigned long long)(mode==1?260000000:mode==3?500000000:mode==5?500000000:3000000));
    if(n<=0||n>=4097)return 0;
    if(mode==2)n=snprintf(release,4097,"%s-release",argument);
    else if(mode==6||(mode>=16&&mode<=25))n=snprintf(release,4097,"%s",argument);
    else return 1;
    return n>0&&n<4097;
}
static int total_spawn(struct engine_owner *o,struct private_engine_result *r,const char *self,int dir,unsigned mode,const char *root) {
    memset(o,0,sizeof(*o));memset(r,0,sizeof(*r));reset();fixture(dir);
    source_known_process_create_case=mode>=13&&mode<=15?mode-12:0;
    source_known_event_registration_failure=mode==25;
    o->context=&context;o->result=r;o->slots=(mode==2||mode==23||mode==24||mode==26||mode==27)?2:1;o->source_admitted=1;o->controller_parent=getppid();
    o->input=o->output=o->errors=-1;o->ledger=-1;o->capture=malloc(ENGINE_CAPTURE_MAX);require(o->capture!=NULL,"bounded total-budget fixture capture");
    fixture_live_owner=o;fixture_release_path[0]=0;
    char ledger[48];snprintf(ledger,sizeof(ledger),"total-%u.ndjson",mode);o->ledger=openat(dir,ledger,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(o->ledger>=0,"owned total-budget fixture ledger");
    struct engine_prepared p;memset(&p,0,sizeof(p));for(unsigned i=0;i<2;i++)p.input[i]=p.output[i]=p.errors[i]=-1;
    strcpy(p.image,self);strcpy(p.descendant,self);strcpy(p.cwd,root);strcpy(o->image,self);strcpy(o->descendant,self);
    const char *kind=mode==27?"same-wake-leader":mode==26?"knote-rollup-child":(mode==23||mode==24)?"event-gated-leader":mode==2?"rollup-child":(mode==6||(mode>=16&&mode<=22)||mode==25)?"held-child":mode==7?"exit-child":mode==8?"rss-child":mode==12?"long-wall-cpu-child":(mode==9||mode==10)?"pause-bypass-child":"burn-child";
    char argument[4097];require(fixture_total_argument(mode,root,argument,fixture_release_path),"bounded fixture argument and exact original release path");
    require(engine_put(&p,self,&p.argv[p.argc++])&&engine_put(&p,kind,&p.argv[p.argc++])&&engine_put(&p,argument,&p.argv[p.argc++]),"bounded source-known helper argv");
    if(mode==23||mode==24)require(engine_put(&p,mode==23?"fork":"none",&p.argv[p.argc++]),"bounded source-known event leader choice");
    if(mode==9||mode==10){char path[4097];snprintf(path,sizeof(path),"%s/pause-%u-done",root,mode);require(engine_put(&p,path,&p.argv[p.argc++]),"bounded actual pause done marker");snprintf(path,sizeof(path),"%s/pause-%u-release",root,mode);require(engine_put(&p,path,&p.argv[p.argc++]),"bounded actual pause release marker");strcpy(fixture_release_path,path);}
    p.argv[p.argc]=NULL;
    if(mode==27){same_wake_prepare(o,root,dir);strcpy(p.descendant,o->descendant);}
    require(engine_put(&p,"PATH=/usr/bin:/bin",&p.env[p.envc++])&&engine_put(&p,"LANG=C",&p.env[p.envc++]),"bounded source-known helper environment");p.env[p.envc]=NULL;
    require(engine_prepare_effect(o,&p,0)&&private_end_unit(private_cost.open_cpu),"total-budget original preparation unit paid");
    o->ticket.sequence=1;r->attempts=(uint16_t)o->slots;engine_current=o;dispose_private_engine=engine_emergency_dispose;
    int born=engine_birth(o,&p);engine_dispose_prepared(&p);return born;
}
static void await_original(struct engine_owner *o) {
    for(;;) {
        int done=0;require(known_exited(o->ticket.pid,&done),"fixture observes original own-WNOWAIT");if(done)break;
        require(clock_ns(CLOCK_MONOTONIC)<o->context->outer->deadline,"original fixture deadline unchanged");
        require(engine_checkpoint(o),"bounded paid terminal observation unit");usleep(1000);
    }
}
static void member_peak_census_control(const char *self,const char *root,int parent) {
    require(mkdirat(parent,"member-peak-census",0700)==0,"fresh census-only member peak fixture");
    int dir=openat(parent,"member-peak-census",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0,"owned census-only root");
    char owned[4097];int n=snprintf(owned,sizeof(owned),"%s/member-peak-census",root);require(n>0&&(size_t)n<sizeof(owned)&&private_end_unit(private_cost.open_cpu),"bounded census-only root setup fully paid before original birth");
    struct engine_owner o;struct private_engine_result r;int status=-1;
    require(total_spawn(&o,&r,self,dir,6,owned),"census-only original creation obeys unchanged gates");
    require(engine_sample(&o)&&o.group_list_known&&o.group_list_count==1&&r.members_max==1,
        "actual census-only live original keeps unchanged independent member maximum update");
    fixture_touch(fixture_release_path);await_original(&o);(void)engine_retire(&o,&status);
    int settled=o.direct_reaped&&o.wait_observed&&r.group_gone&&r.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&status==0&&!o.failed&&!private_failed&&o.helper_cost_retained&&o.helper_cost_paid
        &&private_cost.debt==private_cost.debt_paid,"actual census-only original fully reaped and billed before control completes");
    printf("private_members_peak_census_source_known actual_group_count=1 members_max=%u actual_own_wait4=1 group_gone=1 all_cost_paid=true native_process=true runtime_approved=false\n",r.members_max);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"census-only member peak evidence paid");
    engine_close_transferred(&o);require(close(o.ledger)==0,"census-only owned ledger closed");free(o.capture);
    require(close(dir)==0&&private_end_unit(private_cost.open_cpu),"census-only exact original fixture disposal paid");controls++;
}

// Fixed first/last/worst records distinguish time in the real sample/payment
// path from the fixture's WNOWAIT and sleep. This is observation, never a
// promise that the OS schedules a controller within its hard50ms RI gate.
enum {LIVE_ENTRY_WALL,LIVE_ENTRY_CPU,LIVE_PRIOR_RI,LIVE_QUERY_END_WALL,LIVE_QUERY_END_CPU,
    LIVE_SAMPLE_END_WALL,LIVE_SAMPLE_END_CPU,LIVE_CURRENT_RI,LIVE_DEBT_BEFORE,LIVE_DEBT_AFTER,
    LIVE_PAID_AFTER,LIVE_SLEEP_END_WALL,LIVE_SLEEP_END_CPU,LIVE_GAP,LIVE_OUTCOME,
    LIVE_WAIT_CALLS_BEFORE,LIVE_WAIT_CALLS_AFTER,LIVE_WAIT_NS_BEFORE,LIVE_WAIT_NS_AFTER,LIVE_WAIT_MAX_WALL,LIVE_FIELDS};
struct fixture_live_row {uint64_t values[LIVE_FIELDS];};
struct fixture_live_pacing {uint64_t count,worst_wall;unsigned worst_has_prior;struct fixture_live_row first,last,worst_prior,worst;};
static void fixture_live_retain(struct fixture_live_pacing *p,const struct fixture_live_row *row) {
    uint64_t span=engine_interval(row->values[LIVE_ENTRY_WALL],row->values[LIVE_SLEEP_END_WALL]);
    if(!p->count)p->first=*row;
    if(!p->count||span>p->worst_wall){p->worst_wall=span;p->worst=*row;p->worst_has_prior=p->count!=0;if(p->count)p->worst_prior=p->last;}
    p->last=*row;p->count++;
}
static int fixture_live_record(struct engine_owner *o,const struct fixture_live_pacing *p) {
    char bytes[4096];int n=snprintf(bytes,sizeof(bytes),"{\"kind\":\"fixture_live_pacing\",\"iterations\":%llu,\"worst_wall_ns\":%llu,\"worst_has_prior\":%u,\"redundant_after_sample_charge_removed\":true,\"extra_fixture_sleep\":false,\"rest_request_max_ns\":1000000,\"columns\":[\"entry_wall\",\"entry_cpu\",\"prior_ri_start\",\"query_end_wall\",\"query_end_cpu\",\"sample_end_wall\",\"sample_end_cpu\",\"current_ri_start\",\"debt_before\",\"debt_after\",\"paid_after\",\"loop_end_wall\",\"loop_end_cpu\",\"max_gap\",\"outcome_0sample_1terminal_2refusal\",\"wait_calls_before\",\"wait_calls_after\",\"wait_requested_before\",\"wait_requested_after\",\"wait_max_wall\"],\"rows\":{",(unsigned long long)p->count,(unsigned long long)p->worst_wall,p->worst_has_prior);
    if(n<=0||(size_t)n>=sizeof(bytes))return 0;size_t used=(size_t)n;
    const char *names[]={"first","last","worst_prior","worst"};
    const struct fixture_live_row *rows[]={&p->first,&p->last,&p->worst_prior,&p->worst};
    for(unsigned i=0;i<4;i++) {
        n=snprintf(bytes+used,sizeof(bytes)-used,"%s\"%s\":[",i?",":"",names[i]);if(n<=0||(size_t)n>=sizeof(bytes)-used)return 0;used+=(size_t)n;
        for(unsigned j=0;j<LIVE_FIELDS;j++){n=snprintf(bytes+used,sizeof(bytes)-used,"%s%llu",j?",":"",(unsigned long long)rows[i]->values[j]);if(n<=0||(size_t)n>=sizeof(bytes)-used)return 0;used+=(size_t)n;}
        if(used+1>=sizeof(bytes))return 0;bytes[used++]=']';
    }
    n=snprintf(bytes+used,sizeof(bytes)-used,"},\"runtime_approved\":false}\n");if(n<=0||(size_t)n>=sizeof(bytes)-used)return 0;used+=(size_t)n;
    return engine_write_record(o,bytes,(int)used)&&engine_checkpoint(o);
}
// Fixed saved diagnostic only: no additional native helper observation/effect.
// The capture precedes its own bounded formatter/write/full paid ending guard.
static void fixture_pause_pre_stimulus_diagnostic(const struct engine_owner *owner,unsigned mode,int sample_return,int guard_result,int sample_errno,
        uint64_t entry_wall,uint64_t entry_cpu,uint64_t return_wall,uint64_t return_cpu) {
    const struct engine_owner saved=*owner;const struct private_metrics cost=private_cost;
    int first_private_failed=private_failed;
    char owner_code[49],owner_detail[161],first_private_code[49],first_private_detail[161];
    int a=snprintf(owner_code,sizeof(owner_code),"%s",saved.result&&saved.result->error_code?saved.result->error_code:"none");
    int b=snprintf(owner_detail,sizeof(owner_detail),"%s",saved.result&&saved.result->error_detail?saved.result->error_detail:"none");
    int c=snprintf(first_private_code,sizeof(first_private_code),"%s",first_private_failed?private_error:"none");
    int d=snprintf(first_private_detail,sizeof(first_private_detail),"%s",first_private_failed?private_detail:"none");
    int truncated=a<0||b<0||c<0||d<0||(size_t)a>=sizeof(owner_code)||(size_t)b>=sizeof(owner_detail)
        ||(size_t)c>=sizeof(first_private_code)||(size_t)d>=sizeof(first_private_detail);
    char q_owner_code[300],q_owner_detail[1000],q_private_code[300],q_private_detail[1000];
    int quoted=private_json_quoted(owner_code,q_owner_code,sizeof(q_owner_code))
        &&private_json_quoted(owner_detail,q_owner_detail,sizeof(q_owner_detail))
        &&private_json_quoted(first_private_code,q_private_code,sizeof(q_private_code))
        &&private_json_quoted(first_private_detail,q_private_detail,sizeof(q_private_detail));
    struct {const char *name;uint64_t value;} fields[]={
        {"mode",mode},{"original_pid",(uint64_t)saved.ticket.pid},{"owner_failed",saved.failed},{"private_failed",(uint64_t)first_private_failed},
        {"pause_ack_mask",saved.pause_ack_mask},{"counter_known_mask",saved.pause_counter_known_mask},{"baseline_known",saved.pause_baseline_known},
        {"paused",saved.paused},{"STOP_attempted",saved.pause_stop_attempted},{"resume_attempted",saved.pause_resume_attempted},{"resume_admitted",saved.pause_resume_admitted},
        {"USER_violation",saved.pause_user_violation},{"SYSTEM_violation",saved.pause_system_violation},{"status",saved.members[0].state.status},{"runnable",saved.members[0].state.runnable},
        {"current_USER_raw",saved.members[0].user},{"current_SYSTEM_raw",saved.members[0].system},{"ACK_USER_raw",saved.pause_user[0]},{"ACK_SYSTEM_raw",saved.pause_system[0]},
        {"observed_USER_raw",saved.pause_observed_user[0]},{"observed_SYSTEM_raw",saved.pause_observed_system[0]},
        {"pause_started_ns",saved.pause_started_ns},{"ACK_ns",saved.pause_ack_ns},{"member_ACK_ns",saved.pause_member_ack_ns[0]},
        {"stopped_RI_ACK_ns",saved.members[0].stopped_ri_ack_ns},{"ACK_RI_begin_ns",saved.pause_ack_ri_begin_ns[0]},{"ACK_RI_return_ns",saved.pause_ack_ri_end_ns[0]},
        {"validated_RI_return_ns",saved.pause_validated_ri_end_ns[0]},{"resume_RI_begin_ns",saved.pause_resume_ri_begin_ns[0]},
        {"resume_RI_return_ns",saved.pause_resume_ri_end_ns[0]},{"resume_signal_ns",saved.pause_resume_ns},
        {"RI_previous_begin_ns",saved.ri_previous_ns[0]},{"RI_begin_ns",saved.ri_begin_ns[0]},{"RI_return_ns",saved.ri_end_ns[0]},
        {"member_RI_start_ns",saved.members[0].ri_started_ns},{"RI_previous_mask",saved.ri_previous_mask},{"RI_begin_mask",saved.ri_begin_mask},
        {"RI_return_mask",saved.ri_end_mask},{"RI_sticky_gap_mask",saved.ri_gap_failed_mask},
        {"raw_gap_ns",saved.sample_gap_ns},{"raw_gap_max_ns",saved.max_sample_gap_ns},{"running_gap_ns",saved.running_gap_ns},
        {"running_gap_max_ns",saved.running_gap_max_ns},{"stopped_interval_ns",saved.stopped_interval_ns},
        {"first_failed_check_ns",saved.gap_failed_observed_ns},{"first_failed_previous_ns",saved.gap_failed_previous_ns[0]},
        {"first_failed_raw_gap_ns",saved.gap_failed_raw_ns[0]},{"first_failed_running_gap_ns",saved.gap_failed_running_ns[0]},
        {"sample_entry_wall_ns",entry_wall},{"sample_return_wall_ns",return_wall},{"sample_entry_CPU_ns",entry_cpu},{"sample_return_CPU_ns",return_cpu},
        {"unit_open_CPU_ns",cost.open_cpu},{"debt_ns",cost.debt},{"paid_ns",cost.debt_paid},{"rest_until_ns",cost.rest_until},
        {"deadline_ns",cost.deadline},{"ordinary_max_ns",cost.ordinary_max},{"retained_units",cost.retained_units},{"retained_cost_ns",cost.retained_cost_ns},
        {"native_CPU_ns",saved.native.cpu},{"native_wall_ns",saved.native.wall},{"native_inclusive_ns",saved.native_gate_inclusive_ns},
        {"sample_clocks_known",entry_wall!=UINT64_MAX&&entry_cpu!=UINT64_MAX&&return_wall!=UINT64_MAX&&return_cpu!=UINT64_MAX&&return_wall>=entry_wall&&return_cpu>=entry_cpu}
    };
    _Static_assert(sizeof(fields)/sizeof(fields[0])<=64,"bounded pre-stimulus diagnostic scalar count");
    char bytes[8192];int n=quoted?snprintf(bytes,sizeof(bytes),"private_pause_pre_stimulus_snapshot_source_known {\"kind\":\"fixture_pause_pre_stimulus\",\"member\":0,\"sample_return\":%d,\"guard_result\":%d,\"sample_saved_errno\":%d,\"owner_first_error\":%s,\"owner_first_detail\":%s,\"private_first_error\":%s,\"private_first_detail\":%s,\"first_text_truncated\":%s",sample_return,guard_result,sample_errno,q_owner_code,q_owner_detail,q_private_code,q_private_detail,truncated?"true":"false"):-1;
    int formatted=n>0&&(size_t)n<sizeof(bytes);size_t used=formatted?(size_t)n:0;
    for(unsigned i=0;formatted&&i<sizeof(fields)/sizeof(fields[0]);i++) {
        n=snprintf(bytes+used,sizeof(bytes)-used,",\"%s\":%llu",fields[i].name,(unsigned long long)fields[i].value);
        if(n<=0||(size_t)n>=sizeof(bytes)-used)formatted=0;else used+=(size_t)n;
    }
    if(formatted) {
        n=snprintf(bytes+used,sizeof(bytes)-used,",\"capture_cutoff\":\"before_own_format_write_ending_guard\",\"native_process\":true,\"installed_tool_qualification\":false,\"runtime_approved\":false}\n");
        if(n<=0||(size_t)n>=sizeof(bytes)-used)formatted=0;else used+=(size_t)n;
    }
    int written=formatted&&fwrite(bytes,1,used,stderr)==used&&fflush(stderr)==0;
    int paid=private_end_unit(cost.open_cpu);
    require(written&&paid,"bounded saved pre-stimulus diagnostic formatter write and full ending cost paid");
}
// Capture the first post-burn refusal BEFORE release/retirement can replace
// diagnostics. The direct elapsed bound uses fixture operation clocks, not
// a stopped-time discount after the immutable USER seal has been violated.
static void fixture_pause_post_stimulus_diagnostic(const struct engine_owner *owner,int sample_return,int sample_errno,
        uint64_t cont_begin,uint64_t restop_ack,uint64_t read_entry,uint64_t read_return,int counter_first) {
    const struct engine_owner saved=*owner;const struct private_metrics cost=private_cost;
    char code[49],detail[161],owner_code[49],owner_detail[161];
    int a=snprintf(code,sizeof(code),"%s",private_failed?private_error:"none");
    int b=snprintf(detail,sizeof(detail),"%s",private_failed?private_detail:"none");
    int c=snprintf(owner_code,sizeof(owner_code),"%s",saved.result&&saved.result->error_code?saved.result->error_code:"none");
    int d=snprintf(owner_detail,sizeof(owner_detail),"%s",saved.result&&saved.result->error_detail?saved.result->error_detail:"none");
    int truncated=a<0||b<0||c<0||d<0||(size_t)a>=sizeof(code)||(size_t)b>=sizeof(detail)
        ||(size_t)c>=sizeof(owner_code)||(size_t)d>=sizeof(owner_detail);
    char q_code[300],q_detail[1000],q_owner_code[300],q_owner_detail[1000];
    int quoted=private_json_quoted(code,q_code,sizeof(q_code))&&private_json_quoted(detail,q_detail,sizeof(q_detail))
        &&private_json_quoted(owner_code,q_owner_code,sizeof(q_owner_code))&&private_json_quoted(owner_detail,q_owner_detail,sizeof(q_owner_detail));
    struct {const char *name;uint64_t value;} fields[]={
        {"original_pid",(uint64_t)saved.ticket.pid},{"owner_failed",saved.failed},{"private_failed",(uint64_t)private_failed},
        {"external_CONT_begin_ns",cont_begin},{"positive_external_reSTOP_ACK_ns",restop_ack},{"RI_call_entry_ns",read_entry},{"RI_call_return_ns",read_return},
        {"direct_CONT_to_RI_return_ns",engine_interval(cont_begin,read_return)},{"counter_failure_first",(uint64_t)counter_first},
        {"ACK_USER_raw",saved.pause_user[0]},{"observed_USER_raw",saved.pause_observed_user[0]},
        {"ACK_SYSTEM_raw",saved.pause_system[0]},{"observed_SYSTEM_raw",saved.pause_observed_system[0]},
        {"USER_violation",saved.pause_user_violation},{"SYSTEM_violation",saved.pause_system_violation},
        {"live_checked",saved.pause_live_checked},{"counter_known_mask",saved.pause_counter_known_mask},
        {"status",saved.members[0].state.status},{"runnable",saved.members[0].state.runnable},
        {"raw_gap_ns",saved.sample_gap_ns},{"raw_gap_max_ns",saved.max_sample_gap_ns},
        {"running_gap_ns",saved.running_gap_ns},{"running_gap_max_ns",saved.running_gap_max_ns},
        {"RI_previous_begin_ns",saved.ri_previous_ns[0]},{"RI_begin_ns",saved.ri_begin_ns[0]},{"RI_return_ns",saved.ri_end_ns[0]},
        {"RI_sticky_gap_mask",saved.ri_gap_failed_mask},{"first_failed_check_ns",saved.gap_failed_observed_ns},
        {"first_failed_previous_ns",saved.gap_failed_previous_ns[0]},{"first_failed_raw_gap_ns",saved.gap_failed_raw_ns[0]},
        {"first_failed_running_gap_ns",saved.gap_failed_running_ns[0]},
        {"unit_open_CPU_ns",cost.open_cpu},{"debt_ns",cost.debt},{"paid_ns",cost.debt_paid},{"rest_until_ns",cost.rest_until},{"deadline_ns",cost.deadline}
    };
    _Static_assert(sizeof(fields)/sizeof(fields[0])<=64,"bounded post-stimulus diagnostic scalar count");
    char bytes[8192];int n=quoted?snprintf(bytes,sizeof(bytes),"private_pause_post_stimulus_snapshot_source_known {\"kind\":\"fixture_pause_post_stimulus\",\"member\":0,\"sample_return\":%d,\"sample_saved_errno\":%d,\"private_first_error\":%s,\"private_first_detail\":%s,\"owner_first_error\":%s,\"owner_first_detail\":%s,\"first_text_truncated\":%s",sample_return,sample_errno,q_code,q_detail,q_owner_code,q_owner_detail,truncated?"true":"false"):-1;
    int formatted=n>0&&(size_t)n<sizeof(bytes);size_t used=formatted?(size_t)n:0;
    for(unsigned i=0;formatted&&i<sizeof(fields)/sizeof(fields[0]);i++) {
        n=snprintf(bytes+used,sizeof(bytes)-used,",\"%s\":%llu",fields[i].name,(unsigned long long)fields[i].value);
        if(n<=0||(size_t)n>=sizeof(bytes)-used)formatted=0;else used+=(size_t)n;
    }
    if(formatted) {
        n=snprintf(bytes+used,sizeof(bytes)-used,",\"capture_cutoff\":\"before_own_format_write_ending_guard\",\"native_process\":true,\"installed_tool_qualification\":false,\"runtime_approved\":false}\n");
        if(n<=0||(size_t)n>=sizeof(bytes)-used)formatted=0;else used+=(size_t)n;
    }
    int written=formatted&&fwrite(bytes,1,used,stderr)==used&&fflush(stderr)==0;
    int paid=private_end_unit(cost.open_cpu);
    require(written&&paid,"bounded saved post-stimulus diagnostic formatter write and full ending cost paid");
}
static void total_control(const char *self,const char *root,int dir,unsigned mode) {
    require(mode<12,"eight total-budget controls plus RSS and retained paused-user boundaries");
    struct engine_owner owner;struct private_engine_result result;int status=-1;
    int born=total_spawn(&owner,&result,self,dir,mode,root);
    if(!born) {
        (void)engine_retire(&owner,&status);
        fprintf(stderr,"total_birth_refusal_source_known priority=%d qos_mask=%u detail=%s native_cpu_ns=%llu native_wall_ns=%llu inclusive_ns=%llu actual_reaped=%u group_gone=%u\n",owner.members[0].state.priority,owner.qos_checked_mask,private_detail,(unsigned long long)owner.native.cpu,(unsigned long long)owner.native.wall,(unsigned long long)result.inclusive_max_ns.value,owner.direct_reaped,result.group_gone);
        require(0,"background atomic spawn and unchanged native/application gates before first pipe");
    }
    int observed=1,physical_pending=0,foreground_axis=1,paused_read_return=1;uint64_t child_cpu=0;pid_t child_pid=0;
    uint64_t external_cont_begin=0,external_restop_ack=0,post_RI_entry=0,post_RI_return=0;
    int post_counter_first=0,post_gap_clean=0;
    struct fixture_live_pacing pacing;memset(&pacing,0,sizeof(pacing));
    if(mode==11)source_known_terminal_rss_unavailable=1;
    if(mode==9||mode==10) {
        owner.foreground=1;require(engine_pause(&owner)&&owner.paused&&owner.pause_baseline_known,"below-cap external-CONT fixture obtains immutable positive ACK before stimulus");
        char start[4097],done[4097],release[4097];snprintf(start,sizeof(start),"%s/pause-%u-start",root,mode);snprintf(done,sizeof(done),"%s/pause-%u-done",root,mode);snprintf(release,sizeof(release),"%s/pause-%u-release",root,mode);
        fixture_touch(start);
        uint64_t sample_entry_wall=clock_ns(CLOCK_MONOTONIC),sample_entry_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);
        int stimulus_sample=engine_sample_member(&owner,0,owner.ticket.pid),stimulus_errno=errno;
        uint64_t sample_return_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID),sample_return_wall=clock_ns(CLOCK_MONOTONIC);
        int stimulus_guard=stimulus_sample&&owner.members[0].state.status==SSTOP&&owner.members[0].state.runnable==0;
        fixture_pause_pre_stimulus_diagnostic(&owner,mode,stimulus_sample,stimulus_guard,stimulus_errno,
            sample_entry_wall,sample_entry_cpu,sample_return_wall,sample_return_cpu);
        require(stimulus_guard,
            "fresh actual stopped RI before external pause stimulus preserves immutable ACK");
        require(engine_checkpoint(&owner),"external pause fixture stimulus preparation is paid");
        external_cont_begin=clock_ns(CLOCK_MONOTONIC);
        require(kill(-owner.ticket.pgid,SIGCONT)==0,"fixture external CONT deliberately bypasses product resume");
        require(!owner.pause_resume_admitted&&!owner.pause_resume_attempted,"external CONT never fabricates admitted product resume");
        while(access(done,F_OK)!=0){require(clock_ns(CLOCK_MONOTONIC)<outer.deadline,"original external-pause fixture deadline");require(engine_checkpoint(&owner),"bounded external-pause burn observation paid");usleep(1000);}
        if(mode==10) {
            // The adversarial fixture restores kernel STOP after its burn, so
            // a status-only guard cannot substitute for immutable RI equality.
            require(kill(-owner.ticket.pgid,SIGSTOP)==0,"fixture external reSTOP preserves original PID pin");
            uint64_t until=clock_ns(CLOCK_MONOTONIC)+ENGINE_PAUSE_NS;
            for(;;){struct image state;require(control_observe(owner.ticket.pid,&state)&&state.birth==owner.ticket.birth,"original external reSTOP identity");if(state.status==SSTOP&&state.runnable==0){external_restop_ack=clock_ns(CLOCK_MONOTONIC);break;}require(clock_ns(CLOCK_MONOTONIC)<until,"external reSTOP finite ACK");require(engine_checkpoint(&owner),"external reSTOP observation paid");usleep(1000);}
            post_RI_entry=clock_ns(CLOCK_MONOTONIC);
            paused_read_return=engine_sample_member(&owner,0,owner.ticket.pid);
            int post_sample_errno=errno;post_RI_return=clock_ns(CLOCK_MONOTONIC);
            post_counter_first=private_failed&&!strcmp(private_error,"resource_limit")
                &&(!strcmp(private_detail,"user CPU grew after immutable positive pause ACK")
                    ||!strcmp(private_detail,"system CPU changed across observed stopped interval"));
            post_gap_clean=!owner.ri_gap_failed_mask&&!owner.gap_failed_observed_ns;
            fixture_pause_post_stimulus_diagnostic(&owner,paused_read_return,post_sample_errno,
                external_cont_begin,external_restop_ack,post_RI_entry,post_RI_return,post_counter_first);
        }
        fixture_touch(release);int fixture_cont_ok=1;
        if(mode==10) {
            int terminal=0;require(known_exited(owner.ticket.pid,&terminal),"external-pause release retains original own-WNOWAIT observation");
            unsigned product_kill=owner.group_kill_attempted;int product_errno=owner.group_kill_result;
            int attempted=0,cont_result=-2,cont_errno=0;
            // A valid product refusal may already have consumed final kill.
            // No fixture CONT follows that kill or a positive original exit.
            if(!terminal&&!product_kill){attempted=1;errno=0;cont_result=kill(-owner.ticket.pgid,SIGCONT);cont_errno=cont_result?errno:0;fixture_cont_ok=cont_result==0;}
            fprintf(stderr,"private_pause_fixture_cont_source_known original_pid=%d terminal_WNOWAIT=%d product_kill_attempted=%u product_kill_errno=%d max_RI_gap_ns=%llu paused_read_return=%d live_user_checked=%u ACK_user_raw=%llu observed_user_raw=%llu sticky_user_violation=%u CONT_attempted=%d CONT_return=%d CONT_errno=%d runtime_approved=false\n",owner.ticket.pid,terminal,product_kill,product_errno,(unsigned long long)owner.max_sample_gap_ns,paused_read_return,owner.pause_live_checked,(unsigned long long)owner.pause_user[0],(unsigned long long)owner.pause_observed_user[0],owner.pause_user_violation,attempted,cont_result,cont_errno);
        }
        int release_paid=engine_checkpoint(&owner);
        if(mode==10)require(fixture_cont_ok,"fixture resumes only original externally reSTOPped helper for its exit");
        require(release_paid,"external-pause fixture release observation paid");await_original(&owner);
    } else if(mode==2) {
        char report[4097];snprintf(report,sizeof(report),"%s/rollup-2",root);
        for(;;) {
            FILE *f=fopen(report,"r");if(f){(void)fscanf(f,"%d",&child_pid);fclose(f);}
            struct image leader;if(child_pid&&control_observe(owner.ticket.pid,&leader)&&leader.status==SSTOP)break;
            require(engine_checkpoint(&owner),"bounded rollup fixture discovery unit");usleep(1000);
        }
        struct image child;require(observe(child_pid,&child)&&child.ppid==owner.ticket.pid&&child.pgid==owner.ticket.pid&&child.status==SSTOP&&child.priority==4,"independent actual child inherits observed background priority4");
        owner.members[1].state=child;owner.members[1].phase=ENGINE_EXPECTED;owner.child_image_admitted=1;
        require(known_control_token(child_pid,child.birth,&owner.members[1].token),"exact original fixture descendant audit identity");owner.members[1].token_known=1;
        require(engine_owned_token(&owner,child_pid,child.birth,child.start_absolute,&owner.members[1].token),"retain actual fixture descendant before continuation");
        struct rusage_info_v1 initial_child;require(proc_pid_rusage(child_pid,RUSAGE_INFO_V1,(rusage_info_t *)&initial_child)==0&&engine_usage(&owner,1,&initial_child),"actual admitted child raw starting counters retained");
        owner.child_created=1;result.created++;result.members_max=2;
        printf("private_total_rollup_identity_source_known leader=%d child=%d birth=%llu start=%llu token_version=%u priority=%d controller_reaps_child=false runtime_approved=false\n",owner.ticket.pid,child_pid,(unsigned long long)child.birth,(unsigned long long)child.start_absolute,owner.members[1].token.val[7],child.priority);
        require(engine_checkpoint(&owner),"independent child priority observation paid");
        require(kill(-owner.ticket.pgid,SIGCONT)==0,"one fixture attachment continuation for original leader and child");
        char terminal[4097],release[4097];snprintf(terminal,sizeof(terminal),"%s-terminal",report);snprintf(release,sizeof(release),"%s-release",report);
        while(access(terminal,F_OK)!=0){require(clock_ns(CLOCK_MONOTONIC)<outer.deadline,"original child terminal-capture deadline");require(engine_checkpoint(&owner),"bounded child terminal-gate observation paid");usleep(1000);}
        require(engine_member_terminal(&owner,1)&&owner.members[1].terminal_observed_ns&&owner.members[1].exit_abstime,"positive original exited-child RI precedes helper-owned child wait4");
        fixture_touch(release);require(engine_checkpoint(&owner),"actual terminal child capture and fixture release paid");
        await_original(&owner);
        FILE *f=fopen(report,"r");pid_t reported=0;require(f&&fscanf(f,"%d\n%llu",&reported,(unsigned long long *)&child_cpu)==2&&reported==child_pid&&fclose(f)==0,"actual helper-owned child wait4 report retained");
        require(engine_checkpoint(&owner),"bounded child wait4 report observation paid");
    } else if(mode==3) {
        while(!owner.failed) {
            struct fixture_live_row row;for(unsigned i=0;i<LIVE_FIELDS;i++)row.values[i]=UINT64_MAX;
            row.values[LIVE_ENTRY_WALL]=clock_ns(CLOCK_MONOTONIC);row.values[LIVE_ENTRY_CPU]=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            row.values[LIVE_PRIOR_RI]=owner.members[0].ri_started_ns;row.values[LIVE_DEBT_BEFORE]=private_cost.debt;
            row.values[LIVE_WAIT_CALLS_BEFORE]=private_cost.rest_wait_calls;row.values[LIVE_WAIT_NS_BEFORE]=private_cost.rest_wait_requested_ns;
            int done=0;require(known_exited(owner.ticket.pid,&done),"live-kill fixture owns original WNOWAIT");
            row.values[LIVE_QUERY_END_WALL]=clock_ns(CLOCK_MONOTONIC);row.values[LIVE_QUERY_END_CPU]=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            if(!done)(void)engine_sample(&owner);
            row.values[LIVE_SAMPLE_END_WALL]=clock_ns(CLOCK_MONOTONIC);row.values[LIVE_SAMPLE_END_CPU]=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            row.values[LIVE_CURRENT_RI]=owner.members[0].ri_started_ns;row.values[LIVE_DEBT_AFTER]=private_cost.debt;row.values[LIVE_PAID_AFTER]=private_cost.debt_paid;
            // engine_sample already closes/pays the full observation unit.
            // Do not manufacture a second almost-empty1ms/9ms unit here.
            // This bookkeeping, control CPU and next WNOWAIT stay in
            // the existing open epoch and enter the next sample or retire.
            // The sample already paid at least9ms actual rest. An additional
            // fixture-only sleep is unnecessary; no busy polling replaces it.
            row.values[LIVE_SLEEP_END_WALL]=clock_ns(CLOCK_MONOTONIC);row.values[LIVE_SLEEP_END_CPU]=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            row.values[LIVE_WAIT_CALLS_AFTER]=private_cost.rest_wait_calls;row.values[LIVE_WAIT_NS_AFTER]=private_cost.rest_wait_requested_ns;row.values[LIVE_WAIT_MAX_WALL]=private_cost.rest_wait_max_wall_ns;
            row.values[LIVE_GAP]=owner.max_sample_gap_ns;row.values[LIVE_OUTCOME]=done?1:owner.failed?2:0;fixture_live_retain(&pacing,&row);
            if(done)break;
        }
        observed=owner.live_over_cap&&owner.live_kill_attempted&&owner.group_kill_attempted;
    } else if(mode==5) {
        uint64_t before_user=owner.members[0].user;owner.foreground=1;int paused=engine_pause(&owner);uint64_t user=owner.members[0].user;
        foreground_axis=paused||(!strcmp(private_detail,"foreground positive stopped ACK exceeded100ms")&&owner.max_sample_gap_ns<=ENGINE_SAMPLE_NS&&owner.members[0].state.status!=SSTOP&&owner.members[0].user>before_user);
        if(paused) {
            // One real positively ACKed hold, deliberately longer than the
            // running50ms sampling limit. No helper CONT occurs in this span.
            uint64_t until=clock_ns(CLOCK_MONOTONIC)+80000000;
            while(clock_ns(CLOCK_MONOTONIC)<until){require(engine_checkpoint(&owner),"foreground held observation cost paid");usleep(1000);}
            require(engine_sample_member(&owner,0,owner.ticket.pid),"paused original identity and user snapshot");
            observed=owner.paused&&owner.pause_ack_ns>=owner.pause_started_ns&&owner.pause_ack_ns-owner.pause_started_ns<=ENGINE_PAUSE_NS&&owner.members[0].user==user;
            const struct private_source_known_effects effects={source_known_current,NULL};private_guards.stage=2;private_source_known_effects=&effects;
            owner.foreground=0;observed=observed&&engine_resume(&owner)&&owner.pause_resume_attempted==1;private_source_known_effects=NULL;
        } else observed=0;
    } else if(mode==6) {
        source_known_final_kill_eperm=1;require(engine_kill_group(&owner),"known source seam retains final EPERM without claiming settlement");source_known_final_kill_eperm=0;
        (void)engine_quarantine(&owner,"source-known live original while final kill reports EPERM");
        int done=0;require(known_exited(owner.ticket.pid,&done),"held live original WNOWAIT independently observed");
        physical_pending=!done&&owner.group_kill_pending&&owner.group_kill_result==EPERM&&!owner.direct_reaped&&engine_current==&owner&&!engine_settle_proven(&owner)&&fcntl(owner.output,F_GETFD)>=0;
        // Fixture-only external release stimulus. This does not call a product
        // payload/CONT/signal path or close any of the owner's retained FDs.
        char release[4097];snprintf(release,sizeof(release),"%s/release-6",root);int fd=open(release,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(fd>=0&&close(fd)==0,"owned fixture release stimulus");
        if(physical_pending) {while(!engine_hold_observe(&owner)){require(clock_ns(CLOCK_MONOTONIC)<outer.deadline,"fixture original quarantine settlement deadline");usleep(1000);}}
        else fixture_cleanup();
        observed=physical_pending&&owner.quarantine_settled&&owner.direct_reaped&&result.group_gone&&owner.group_kill_raced_exit;
    } else await_original(&owner);
    if(mode==7)require(engine_kill_group(&owner),"natural-exit final group kill retains raw result");
    if(mode!=6)(void)engine_retire(&owner,&status);
    int settled=owner.direct_reaped&&result.group_gone&&result.retired_coverage&&!engine_current;
    uint64_t authoritative=engine_wait4_cpu(&owner.retained_wait);source_known_terminal_rss_unavailable=0;
    if(mode==3)require(fixture_live_record(&owner,&pacing),"bounded fixture pacing diagnostics are fully charged");
    // Distinct prerequisite assertions prevent an unrelated native/clock/unit
    // refusal from satisfying a mutant's intended property assertion.
    require(owner.native.cpu_known&&owner.native.wall_known&&owner.native.cpu<=2000000&&owner.native.wall<=250000000
        &&result.inclusive_max_ns.known&&result.inclusive_max_ns.value<=3000000&&private_cost.ordinary_max<=1000000,
        "total causal fixture requires unchanged native inclusive and own-thread unit gates");
    if(mode!=6)require(settled&&owner.wait_observed,"total causal fixture requires actual original ownwait4 and group proof before property verdict");
    if(mode==1)require(authoritative>ENGINE_COMMAND_CPU_NS,"terminal cap causal fixture independently crossed actual CPU boundary");
    if(mode==2)require(child_cpu>ENGINE_COMMAND_CPU_NS&&authoritative>=child_cpu,"rollup causal fixture independently retained child and leader wait4 CPU");
    if(mode==3)require(owner.live_over_cap&&owner.max_sample_gap_ns<=ENGINE_SAMPLE_NS,"live kill causal fixture reached CPU boundary without a sampling-gap refusal");
    if(mode==5)require(foreground_axis,"foreground causal fixture reached its intended stopped-ACK or missing-STOP boundary");
    if(mode==8)require(owner.wait_rss>ENGINE_HELPER_RSS,"terminal RSS causal fixture independently crossed actual RSS boundary");
    if(mode==9||mode==10)require(owner.pause_baseline_known&&owner.wait_user>owner.pause_user_baseline_ns+1000&&authoritative<=ENGINE_COMMAND_CPU_NS&&!owner.pause_resume_admitted,"external-CONT causal fixture retained actual below-cap user growth beyond ACK resolution");
    if(mode==10)require(owner.pause_live_checked&&owner.pause_observed_user[0]>owner.pause_user[0]
        &&external_cont_begin&&external_cont_begin!=UINT64_MAX&&external_restop_ack>=external_cont_begin
        &&post_RI_entry>=external_restop_ack&&post_RI_return>=post_RI_entry
        &&engine_interval(external_cont_begin,post_RI_return)<=ENGINE_SAMPLE_NS
        &&post_counter_first&&post_gap_clean&&owner.running_gap_max_ns<=ENGINE_SAMPLE_NS
        &&!owner.ri_gap_failed_mask&&!owner.gap_failed_observed_ns,
        "live paused-user causal fixture reached actual RI growth without timing refusal");
    if(mode==11)require(owner.wait_rss_test_withheld&&owner.wait_rss_native_raw>=0&&authoritative!=UINT64_MAX&&authoritative<=ENGINE_COMMAND_CPU_NS,"unknown RSS causal fixture preserves actual known CPU and native RSS separately");
    printf("private_total_property_axis_source_known mode=%u own_wait4=%u group_gone=%u authoritative_cpu_ns=%llu terminal_cpu_ns=%llu terminal_pass=%u max_ri_gap_ns=%llu own_unit_max_ns=%llu native_cpu_ns=%llu native_wall_ns=%llu runtime_approved=false\n",mode,owner.wait_observed,result.group_gone,(unsigned long long)authoritative,(unsigned long long)owner.terminal_cpu_ns,owner.terminal_budget_passed,(unsigned long long)owner.max_sample_gap_ns,(unsigned long long)private_cost.ordinary_max,(unsigned long long)owner.native.cpu,(unsigned long long)owner.native.wall);
    if(mode==9)require(settled&&owner.pause_terminal_checked&&owner.pause_user_violation&&!owner.pause_user_pass&&!owner.terminal_budget_passed&&private_errors[5].count,"terminal actual below-cap user growth after ACK refuses without any product resume");
    if(mode==10)require(settled&&!paused_read_return&&owner.pause_user_violation&&!owner.terminal_budget_passed&&private_errors[5].count,"live actual below-cap user growth after ACK remains sticky through ownwait4");
    if(mode==11)require(settled&&!owner.wait_rss_known&&!owner.terminal_budget_passed&&private_errors[2].count&&!private_errors[5].count&&owner.helper_cost_retained&&owner.helper_cost_paid,"unknown terminal RSS stays unavailable while full known wait4 CPU debt is paid");
    if(mode==0)require(settled&&owner.qos_checked_mask==1&&owner.members[0].state.priority==4,"background atomic attribute and observed priority4 gate before pipe");
    if(mode==1)require(settled&&authoritative>ENGINE_COMMAND_CPU_NS&&!owner.terminal_budget_passed&&!strcmp(private_error,"resource_limit"),"actual terminal wait4 refuses CPU burned beyond last live sample");
    if(mode==2)require(settled&&child_cpu>ENGINE_COMMAND_CPU_NS&&authoritative>=child_cpu&&owner.terminal_cpu_ns>=child_cpu&&!owner.terminal_budget_passed,"authoritative leader wait4 retains actual reaped-child CPU rollup");
    if(mode==3)require(settled&&observed&&authoritative>ENGINE_COMMAND_CPU_NS,"live cap crossing sends original group kill before terminal verdict");
    if(mode==4)require(settled&&owner.helper_cost_retained&&owner.helper_cost_paid&&private_cost.debt_paid>=owner.helper_debt_target&&owner.helper_debt_target>=owner.helper_debt_before+9*(authoritative<1000000?1000000:authoritative),"actual terminal helper debt precedes next expensive work");
    if(mode==5)require(settled&&observed,"foreground one STOP positive ACK no user growth and admitted single resume");
    if(mode==6)require(settled&&observed,"EPERM with live original retains same-owner quarantine until actual ownreap and groupgone");
    if(mode==8)require(settled&&owner.wait_rss>ENGINE_HELPER_RSS&&!owner.terminal_budget_passed,"actual terminal wait4 RSS above16MiB refuses after ownreap and full CPU payment");
    if(mode==7)require(settled&&!owner.group_kill_pending&&owner.group_kill_raced_exit&&private_errors[7].count==0,"natural-exit pending final kill resolves errno after ownreap groupgone without stale unknown");
    require(owner.terminal_cpu_ns==authoritative&&owner.helper_cost_retained&&owner.helper_cost_paid&&private_cost.debt==private_cost.debt_paid,"raw terminal CPU never clamps and full rounded helper cost is paid");
    printf("private_total_budget_control_source_known mode=%u original_pid=%d actual_own_wait4=%u group_gone=%u terminal_cpu_ns=%llu child_cpu_ns=%llu job_cpu_ns=%llu command_cap_ns=%llu helper_cost_retained=%u helper_cost_paid=%u kill_errno=%d raced_exit=%u pause_ack_ns=%llu debt_ns=%llu paid_ns=%llu runtime_approved=false\n",mode,owner.ticket.pid,owner.direct_reaped,result.group_gone,(unsigned long long)owner.terminal_cpu_ns,(unsigned long long)child_cpu,(unsigned long long)result.helper_job_cpu_ns,(unsigned long long)ENGINE_COMMAND_CPU_NS,owner.helper_cost_retained,owner.helper_cost_paid,owner.group_kill_result,owner.group_kill_raced_exit,(unsigned long long)owner.pause_ack_ns,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    engine_close_transferred(&owner);require(close(owner.ledger)==0,"owned total fixture ledger close");free(owner.capture);fixture_live_owner=NULL;
    if(mode==5) {
        require(owner.pause_resume_admitted&&owner.pause_ack_mask==1&&owner.pause_counter_known_mask==1
            &&owner.pause_member_ack_ns[0]==owner.pause_ack_ns&&owner.pause_ack_ri_begin_ns[0]>=owner.pause_ack_ns
            &&owner.pause_ack_ri_end_ns[0]>=owner.pause_ack_ri_begin_ns[0]
            &&owner.pause_resume_ri_begin_ns[0]>=owner.pause_ack_ri_end_ns[0]
            &&owner.pause_resume_ri_end_ns[0]>=owner.pause_resume_ri_begin_ns[0]
            &&owner.pause_resume_ns>=owner.pause_resume_ri_end_ns[0]
            &&owner.pause_user[0]==owner.pause_observed_user[0]&&owner.pause_system[0]==owner.pause_observed_system[0]
            &&!owner.pause_user_violation&&!owner.pause_system_violation
            &&owner.stopped_interval_ns==owner.pause_resume_ns-owner.pause_ack_ns
            &&owner.stopped_interval_ns>ENGINE_SAMPLE_NS&&owner.max_sample_gap_ns>ENGINE_SAMPLE_NS
            &&owner.running_gap_max_ns<=ENGINE_SAMPLE_NS&&private_cost.debt==private_cost.debt_paid,
            "actual ACKed stable USER and SYSTEM stop beyond50ms preserves bounded running sides and full settlement");
        printf("private_stopped_gap_process_source_known original_pid=%d ack_ns=%llu ack_RI_begin_ns=%llu ack_RI_return_ns=%llu resume_RI_begin_ns=%llu resume_RI_return_ns=%llu resume_signal_ns=%llu ACK_USER_raw=%llu end_USER_raw=%llu ACK_SYSTEM_raw=%llu end_SYSTEM_raw=%llu stopped_interval_ns=%llu raw_gap_max_ns=%llu running_gap_max_ns=%llu helper_CPU_ns=%llu helper_debt_delta_ns=%llu debt_ns=%llu paid_ns=%llu actual_own_wait4=1 group_gone=1 original_FD_closed=true native_process=true installed_tool_qualification=false runtime_approved=false\n",
            owner.ticket.pid,(unsigned long long)owner.pause_ack_ns,(unsigned long long)owner.pause_ack_ri_begin_ns[0],(unsigned long long)owner.pause_ack_ri_end_ns[0],
            (unsigned long long)owner.pause_resume_ri_begin_ns[0],(unsigned long long)owner.pause_resume_ri_end_ns[0],(unsigned long long)owner.pause_resume_ns,
            (unsigned long long)owner.pause_user[0],(unsigned long long)owner.pause_observed_user[0],(unsigned long long)owner.pause_system[0],(unsigned long long)owner.pause_observed_system[0],
            (unsigned long long)owner.stopped_interval_ns,(unsigned long long)owner.max_sample_gap_ns,(unsigned long long)owner.running_gap_max_ns,
            (unsigned long long)owner.terminal_cpu_ns,(unsigned long long)owner.helper_debt_delta_ns,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"stopped interval actual process receipt fully paid after original FD disposal");
    }
    controls++;
}

static void helper_cpu_debt_control(const char *self,const char *root,int dir) {
    struct engine_owner owner;struct private_engine_result result;int status=-1;
    int born=total_spawn(&owner,&result,self,dir,12,root);
    if(!born){(void)engine_retire(&owner,&status);require(0,"long-wall CPU witness requires unchanged actual native admission");}
    for(;;) {
        int done=0;require(known_exited(owner.ticket.pid,&done),"long-wall original WNOWAIT remains owned");if(done)break;
        require(engine_sample(&owner),"long-wall live helper keeps actual priority CPU and50ms RI guards");
    }
    (void)engine_retire(&owner,&status);
    uint64_t cpu=engine_wait4_cpu(&owner.retained_wait),wall=engine_interval(owner.activation_ns,owner.wait_read_ns);
    int settled=owner.wait_observed&&owner.direct_reaped&&result.group_gone&&result.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&owner.native.cpu_known&&owner.native.wall_known&&owner.native.cpu<=2000000&&owner.native.wall<=250000000
        &&result.inclusive_max_ns.known&&result.inclusive_max_ns.value<=3000000&&private_cost.ordinary_max<=1000000
        &&owner.max_sample_gap_ns<=ENGINE_SAMPLE_NS&&owner.terminal_budget_passed,
        "long-wall CPU witness retains every original native ordinary live terminal and actual-reap gate");
    require(wall!=UINT64_MAX&&cpu!=UINT64_MAX&&wall>=100000000&&wall>2*cpu,
        "long-wall CPU witness actually separates wall from authoritative original wait4 CPU");
    printf("private_helper_cpu_debt_source_known original_pid=%d actual_own_wait4=%u group_gone=%u wall_ns=%llu CPU_ns=%llu helper_cost_ns=%llu helper_debt_before_ns=%llu helper_debt_delta_ns=%llu helper_debt_target_ns=%llu global_paid_after_helper_ns=%llu global_debt_ns=%llu global_paid_ns=%llu scope=actual_owned_fixture_not_real_candidate runtime_approved=false\n",
        owner.ticket.pid,owner.direct_reaped,result.group_gone,(unsigned long long)wall,(unsigned long long)cpu,(unsigned long long)owner.helper_cost_ns,
        (unsigned long long)owner.helper_debt_before,(unsigned long long)owner.helper_debt_delta_ns,(unsigned long long)owner.helper_debt_target,
        (unsigned long long)owner.helper_global_paid_after,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"long-wall CPU witness raw formatter and write fully paid");
    require(owner.helper_debt_before>0&&owner.helper_cost_ns==engine_helper_cpu_cost(cpu)
        &&owner.helper_debt_delta_ns==9*engine_helper_cpu_cost(cpu)
        &&owner.helper_debt_target-owner.helper_debt_before==owner.helper_debt_delta_ns
        &&owner.helper_global_paid_after>=owner.helper_debt_target&&private_cost.debt==private_cost.debt_paid&&private_debt_components_match(),
        "long wall helper adds only nine times wait4 CPU above its existing global debt baseline");
    engine_close_transferred(&owner);require(close(owner.ledger)==0,"long-wall original raw ledger close");free(owner.capture);
    require(private_end_unit(private_cost.open_cpu),"long-wall fixture original disposal cost paid");controls++;reset();
}

static unsigned live_race_mode,live_race_hook_calls,live_race_gate_released;
static uint64_t live_race_terminal_start,live_race_terminal_exit;
static void live_race_before_read(struct engine_owner *o,unsigned i,pid_t pid,unsigned site) {
    if(site!=(live_race_mode==6?2u:1u))return;
    source_known_before_live_read=NULL;live_race_hook_calls++;
    require(i==0&&pid==o->ticket.pid&&o->direct_custody&&!o->direct_reaped,"race fixture targets only its original unreaped direct member");
    source_known_taskall_pid=pid;
    if(live_race_mode==4){source_known_taskall_clock_failure=1;return;}
    if(live_race_mode==5){source_known_taskall_error=ESRCH;return;}
    int fd=open(fixture_release_path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(fd>=0&&close(fd)==0,"race fixture releases only the original held child's gate after precheck");live_race_gate_released=1;
    await_original(o);
    struct rusage_info_v1 u;memset(&u,0,sizeof(u));
    require(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&u)==0
        &&u.ri_proc_start_abstime==o->ticket.start&&u.ri_proc_exit_abstime,
        "race fixture independently observes exact original terminal RI before injected TASKALLINFO error");
    live_race_terminal_start=u.ri_proc_start_abstime;live_race_terminal_exit=u.ri_proc_exit_abstime;
    if(live_race_mode==1)o->members[0].state.birth++;
    if(live_race_mode==2)o->members[0].token.val[7]^=1;
    require(engine_checkpoint(o),"race fixture exit and supplied identity mutation CPU fully paid");
    source_known_taskall_error=live_race_mode==3?EIO:ESRCH;
}
static int live_sample_race_control(const char *self,const char *root,int dir,unsigned mode) {
    require(mode<7,"seven first and closing live-sample race cases");
    struct engine_owner owner;struct private_engine_result result;int status=-1;
    int born=total_spawn(&owner,&result,self,dir,16+mode,root);
    if(!born){(void)engine_retire(&owner,&status);require(0,"live race fixture requires actual unchanged native and background admission");}
    require(engine_sample_owned_member(&owner,0,owner.ticket.pid),"race fixture starts with exact current retained PID birth start and full token");
    struct engine_member original=owner.members[0];
    owner.members[0].token.val[0]^=1;
    require(!engine_sample_owned_member(&owner,0,owner.ticket.pid),"race recovery full-token premise refuses changes outside PID version");owner.members[0]=original;
    owner.direct_reaped=1;require(!engine_sample_owned_member(&owner,0,owner.ticket.pid),"race recovery never rechecks a reaped original PID");owner.direct_reaped=0;
    require(engine_checkpoint(&owner),"race fixture original identity premise checks fully paid");
    live_race_mode=mode;live_race_hook_calls=live_race_gate_released=0;live_race_terminal_start=live_race_terminal_exit=0;
    source_known_before_live_read=live_race_before_read;
    int sampled=engine_sample_member(&owner,0,owner.ticket.pid);
    int read_error=source_known_live_read_error,native_result=source_known_taskall_native_result,native_error=source_known_taskall_native_errno;
    unsigned rechecks=source_known_live_rechecks;
    source_known_before_live_read=NULL;source_known_taskall_pid=0;source_known_taskall_error=source_known_taskall_clock_failure=0;
    // Only supplied fixture identities are restored; original custody pins and
    // kernel identities never changed. Settlement uses the original owner.
    if(mode==1||mode==2)owner.members[0]=original;
    if(!live_race_gate_released) {
        int fd=open(fixture_release_path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(fd>=0&&close(fd)==0,"unreleased race fixture opens only its original held gate before cleanup");live_race_gate_released=1;
        await_original(&owner);
    }
    (void)engine_retire(&owner,&status);
    int settled=owner.direct_reaped&&owner.wait_observed&&result.group_gone&&result.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&owner.native.cpu_known&&owner.native.wall_known&&owner.native.cpu<=2000000&&owner.native.wall<=250000000
        &&result.inclusive_max_ns.known&&result.inclusive_max_ns.value<=3000000&&private_cost.ordinary_max<=1000000
        &&owner.helper_cost_retained&&owner.helper_cost_paid&&private_cost.debt==private_cost.debt_paid,
        "live race fixture preserves actual native unit CPU debt ownwait4 and group settlement prerequisites");
    printf("private_live_sample_race_axis_source_known mode=%u original_pid=%d hook_calls=%u sampled=%d applied_read_error=%d terminal_rechecks=%u native_TASKALLINFO_result=%d native_TASKALLINFO_errno=%d terminal_start=%llu terminal_exit=%llu actual_own_wait4=%u group_gone=%u actual_CPU_ns=%llu debt_ns=%llu paid_ns=%llu read_error_or_clock_fault_injected=true real_candidate_created=0 runtime_approved=false\n",
        mode,owner.ticket.pid,live_race_hook_calls,sampled,read_error,rechecks,native_result,native_error,
        (unsigned long long)live_race_terminal_start,(unsigned long long)live_race_terminal_exit,owner.direct_reaped,result.group_gone,
        (unsigned long long)owner.terminal_cpu_ns,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"live race raw outcome and actual native distinction fully paid");
    require(live_race_hook_calls==1,"live race fixture injects strictly after the first terminal precheck");
    if(!mode)require(sampled&&read_error==ESRCH&&rechecks==1&&live_race_terminal_start==owner.ticket.start
        &&live_race_terminal_exit&&!owner.failed&&!private_failed,
        "ESRCH after injected original exit rechecks qualified terminal proof and settles successfully");
    else if(mode==6)require(sampled&&read_error==ESRCH&&rechecks==1&&live_race_terminal_start==owner.ticket.start
        &&live_race_terminal_exit&&!owner.failed&&!private_failed,
        "closing TASKALLINFO ESRCH reuses the same qualified original terminal rescue");
    else if(mode==1||mode==2)require(!sampled&&read_error==ESRCH&&!rechecks&&private_errors[2].count,
        "ESRCH with supplied foreign birth or full token mismatch remains coverage unknown");
    else if(mode==3)require(!sampled&&read_error==EIO&&!rechecks&&private_errors[2].count,
        "nonESRCH read failure never uses otherwise available terminal proof");
    else if(mode==4)require(!sampled&&read_error==EIO&&!rechecks&&private_errors[2].count&&native_result==(int)sizeof(struct proc_taskallinfo),
        "successful TASKALLINFO then clock failure cannot reuse stale ESRCH for recovery");
    else require(!sampled&&read_error==ESRCH&&rechecks==1&&!live_race_terminal_exit&&private_errors[2].count,
        "ESRCH while exact original remains live never substitutes errno for terminal proof");
    engine_close_transferred(&owner);require(close(owner.ledger)==0,"race fixture original ledger close");free(owner.capture);
    require(private_end_unit(private_cost.open_cpu),"race fixture exact original disposal fully paid");controls++;
    printf("private_live_sample_race_completed_source_known mode=%u actual_own_wait4=1 group_gone=1 expected_exit=%u terminal_fields_changed=false runtime_approved=false\n",mode,(mode&&mode!=6)?81:0);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"race completion evidence fully paid");
    return mode&&mode!=6?81:0;
}

// Deterministic production-API trace: child lifetime100us lies strictly
// between polling instants. These supplied times are NOT OS timing claims.
// The separate owned process cases below use actual kqueue notifications.
static void verifier_event_api(void) {
    reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    o.ticket.pid=getpid();o.slots=2;
    const uint64_t previous_poll=1000000,fork_at=1100000,exit_at=1200000,next_poll=10000000;
    require(previous_poll<fork_at&&fork_at<exit_at&&exit_at<next_poll&&exit_at-fork_at<1000000,"scripted fast verifier is absent at both finite poll instants");
    struct kevent event;EV_SET(&event,(uintptr_t)o.ticket.pid,EVFILT_PROC,0,NOTE_FORK|NOTE_EXEC|NOTE_EXIT,999999,NULL);
    require(engine_event_note(&o,&event,10000000),"coalesced original leader proc notification is accepted without child PID inference");
    unsigned via=o.fork_census_pending?2:1;o.fork_census_pending=0;
    int census=engine_census(&o,via,0);
    require(!census&&o.leader_fork_events==1&&o.leader_exec_events==1&&o.fork_seen_child_missed
        &&!o.child_created&&!o.members[1].state.pid&&!o.verified_descendants&&!r.created&&!r.reaped
        &&!strcmp(private_detail,"verifier forked and exited before census"),
        "fast scripted between-poll fork notification produces distinct honest missed-verifier refusal");
    require(private_end_unit(private_cost.open_cpu),"scripted notification and actual empty census CPU fully paid");
    reset();memset(&o,0,sizeof(o));o.context=&context;o.result=&r;
    require(!strcmp(engine_missing_verifier_detail(&o),"signature leader created no verifier"),"no observed fork retains distinct created-no-verifier refusal");
    // FIRST cause is retained; a later fork event cannot relabel an already
    // admitted polling discovery. The notification alone creates no member.
    o.ticket.pid=getpid();o.members[1].state.pid=123;o.child_discovered_via=1;
    require(engine_event_note(&o,&event,10000000)&&o.child_discovered_via==1&&!o.child_created,"later fork notification never relabels first polling discovery or invents a birth");
    require(!engine_event_note(&o,&(struct kevent){.ident=999999,.filter=EVFILT_PROC,.fflags=NOTE_EXIT},10000000),"foreign process notification is not retained identity or settlement evidence");
    require(private_end_unit(private_cost.open_cpu),"event API independent predicates fully paid");
    reset();memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    int queue=kqueue();require(queue>=0&&fcntl(queue,F_SETFD,FD_CLOEXEC)==0,"event lifetime API owns one actual close-on-exec queue");
    o.event_fd=queue;o.event_fd_live=1;o.ticket.pid=getpid();
    require(!engine_events_close(&o)&&o.event_fd_live&&!o.event_close_attempted&&fcntl(queue,F_GETFD)>=0,"owned event queue cannot close before original physical settlement");
    require(private_end_unit(private_cost.open_cpu),"event close refusal actual cost paid before supplied settlement");
    o.direct_reaped=1;r.group_gone=1;
    require(engine_events_close(&o)&&!o.event_fd_live&&o.event_close_attempted&&!o.event_close_errno,"event queue closes once only after supplied ownreap and groupgone API premises");
    require(private_end_unit(private_cost.open_cpu),"actual event queue close is charged and paid");
    printf("private_verifier_event_api_source_known scripted_child_lifetime_ns=100000 notifications_coalesce=true native_process=false actual_queue_lifetime=true runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"event API marker fully paid");controls++;reset();
}
static void verifier_event_leader(const char *gate,int forks) {
    while(access(gate,F_OK)!=0)usleep(1000);
    if(forks) {
        pid_t child=fork();if(child<0)_exit(77);if(!child)_exit(0);
        // Keep this original child unreaped until its actual kernel birth/exit
        // interval is retained. No supplied API time is an OS lifetime claim.
        siginfo_t terminal;memset(&terminal,0,sizeof(terminal));
        int waited;do{waited=waitid(P_PID,child,&terminal,WEXITED|WNOWAIT);}while(waited<0&&errno==EINTR);
        int own_terminal=waited==0&&terminal.si_pid==child&&terminal.si_code==CLD_EXITED;
        struct rusage_info_v1 usage;memset(&usage,0,sizeof(usage));errno=0;
        int ri_result=proc_pid_rusage(child,RUSAGE_INFO_V1,(rusage_info_t *)&usage),ri_error=errno;
        mach_timebase_info_data_t tb={0,0};int timebase=mach_timebase_info(&tb)==KERN_SUCCESS&&tb.numer&&tb.denom;
        uint64_t lifetime=UINT64_MAX;
        int known=own_terminal&&ri_result==0&&usage.ri_proc_start_abstime&&usage.ri_proc_exit_abstime>usage.ri_proc_start_abstime
            &&timebase&&engine_cpu_ticks(usage.ri_proc_exit_abstime-usage.ri_proc_start_abstime,&lifetime);
        char line[512];int n=snprintf(line,sizeof(line),"private_verifier_child_lifetime_source_known child_pid=%d own_WNOWAIT=%d ri_result=%d ri_errno=%d start_abstime=%llu exit_abstime=%llu lifetime_ns=%llu timebase_numer=%u timebase_denom=%u\n",
            child,own_terminal,ri_result,ri_error,(unsigned long long)usage.ri_proc_start_abstime,(unsigned long long)usage.ri_proc_exit_abstime,(unsigned long long)lifetime,tb.numer,tb.denom);
        int written=n>0&&(size_t)n<sizeof(line)&&write(STDOUT_FILENO,line,(size_t)n)==n;
        int status=0;pid_t got;do{got=waitpid(child,&status,0);}while(got<0&&errno==EINTR);
        if(!known||!written||got!=child||!WIFEXITED(status)||WEXITSTATUS(status))_exit(77);
    }
    _exit(0);
}
static int verifier_event_control(const char *self,const char *root,int dir,unsigned mode) {
    require(mode<3,"three owned verifier notification controls");
    struct engine_owner o;struct private_engine_result r;int status=-1;
    int born=total_spawn(&o,&r,self,dir,23+mode,root);source_known_event_registration_failure=0;
    struct private_git_sig_temp missing_sig;memset(&missing_sig,0,sizeof(missing_sig));missing_sig.scratch_fd=missing_sig.signature_fd=missing_sig.transient_fd=-1;
    if(mode!=2)o.command=12;
    if(mode==1)o.git_signature=&missing_sig; // No real GS1 descriptor: GS2 must still refuse independently.
    unsigned registered=o.event_registered_mask,held_before=o.event_fd_live;int queue=o.event_fd;
    int missed_fork_observed=0;uint64_t poll_before_abstime=0,poll_after_abstime=0;
    if(mode!=2) {
        int flags=held_before?fcntl(queue,F_GETFD):-1;
        require(born&&registered==1&&held_before&&flags>=0&&(flags&FD_CLOEXEC),"actual leader registration and owned close-on-exec queue precede first controlled continuation");
        // The first real empty census finishes while the original leader is
        // still gated. The next census begins only after its positive WNOWAIT.
        require(engine_census(&o,1,1)&&native_pid_list_empty(o.child_list_count[0],o.child_list_errno[0]),"gated event fixture proves original pre-fork native census empty");
        poll_before_abstime=mach_absolute_time();
        fixture_touch(fixture_release_path);await_original(&o);
        poll_after_abstime=mach_absolute_time();
        int observed=engine_events_service(&o,0);
        if(!mode)missed_fork_observed=!observed&&o.leader_fork_events&&o.fork_seen_child_missed&&!o.child_created
            &&!strcmp(private_detail,"verifier forked and exited before census");
        else {
            require(observed&&!o.leader_fork_events&&!o.fork_seen_child_missed&&!o.child_created,"actual never-fork leader has no invented verifier notification");
        }
    }
    (void)engine_retire(&o,&status);
    int settled=o.direct_reaped&&o.wait_observed&&r.group_gone&&r.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&o.helper_cost_retained&&o.helper_cost_paid&&private_cost.debt==private_cost.debt_paid
        &&!o.event_fd_live&&o.event_close_attempted&&!o.event_close_errno,
        "event refusal retains original owner through actual ownwait4 groupgone full CPU debt and paid queue close");
    printf("private_verifier_event_axis_source_known mode=%u original_pid=%d actual_own_wait4=%u group_gone=%u leader_fork_events=%llu leader_exec_events=%llu fork_seen_child_missed=%u registered_mask=%u registration_errno=%d queue_held_before=%u close_attempted=%u close_errno=%d helper_CPU_ns=%llu helper_debt_delta_ns=%llu debt_ns=%llu paid_ns=%llu runtime_approved=false\n",
        mode,o.ticket.pid,o.direct_reaped,r.group_gone,(unsigned long long)o.leader_fork_events,(unsigned long long)o.leader_exec_events,o.fork_seen_child_missed,registered,o.event_register_errno[0],held_before,o.event_close_attempted,o.event_close_errno,(unsigned long long)o.terminal_cpu_ns,(unsigned long long)o.helper_debt_delta_ns,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"event refusal raw evidence fully paid");
    if(!mode) {
        require(engine_capture_drain(&o)&&o.out_len<1024,"actual child lifetime receipt is bounded and drained after original settlement");
        o.capture[o.out_len]=0;int child_pid=0,own_terminal=0,ri_result=-1,ri_error=0,consumed=0;
        unsigned numer=0,denom=0;unsigned long long start=0,exit_at=0,lifetime=UINT64_MAX;
        int fields=sscanf((char *)o.capture,"private_verifier_child_lifetime_source_known child_pid=%d own_WNOWAIT=%d ri_result=%d ri_errno=%d start_abstime=%llu exit_abstime=%llu lifetime_ns=%llu timebase_numer=%u timebase_denom=%u\n%n",
            &child_pid,&own_terminal,&ri_result,&ri_error,&start,&exit_at,&lifetime,&numer,&denom,&consumed);
        uint64_t checked=UINT64_MAX;int known=fields==9&&consumed>0&&(size_t)consumed==o.out_len
            &&child_pid>0&&own_terminal==1&&ri_result==0&&numer&&denom&&start&&exit_at>start
            &&engine_cpu_ticks(exit_at-start,&checked)&&checked==lifetime
            &&(((__uint128_t)(exit_at-start)*numer+denom-1)/denom)==lifetime;
        int between=known&&poll_before_abstime&&poll_before_abstime<start&&exit_at<poll_after_abstime;
        int normal=WIFEXITED(o.retained_status)&&WEXITSTATUS(o.retained_status)==0;
        require(write(STDOUT_FILENO,o.capture,o.out_len)==(ssize_t)o.out_len,"actual child kernel lifetime raw receipt retained by controller");
        printf("private_verifier_event_lifetime_axis_source_known original_pid=%d child_pid=%d known=%u normal_leader_exit=%u poll_before_abstime=%llu child_start_abstime=%llu child_exit_abstime=%llu poll_after_abstime=%llu lifetime_ns=%llu strictly_between_polls=%u actual_own_wait4=1 group_gone=1 runtime_approved=false\n",
            o.ticket.pid,child_pid,(unsigned)known,(unsigned)normal,(unsigned long long)poll_before_abstime,start,exit_at,(unsigned long long)poll_after_abstime,lifetime,(unsigned)between);
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"actual child kernel lifetime parsing and raw evidence fully paid before verdict");
        require(known&&normal&&between,"actual measured verifier kernel lifetime is known and strictly between native census polls");
        require(missed_fork_observed,"actual queued leader fork survives child reap and terminal precheck before census");
    }
    if(mode==2)require(!born&&!registered&&held_before&&o.event_register_errno[0]==EIO&&!o.activation_ns&&private_errors[2].count,
        "failed leader event registration refuses coverage before any controlled continuation and still settles original effect");
    else if(mode==1)require(o.failed&&private_errors[2].count&&private_errors[7].count
        &&!strcmp(r.error_detail,"signature leader created no verifier")&&missing_sig.quarantined,
        "never-fork actual leader keeps created-no-verifier detail before independent missing-GS1 GS2 refusal");
    else require(o.failed&&private_errors[2].count&&!o.verified_descendants,"missed real fork never claims verifier retention GS1 or whole-lifetime coverage");
    engine_close_transferred(&o);require(close(o.ledger)==0,"event refusal original ledger close");free(o.capture);
    require(private_end_unit(private_cost.open_cpu),"event refusal original disposal fully paid");controls++;
    printf("private_verifier_event_completed_source_known mode=%u actual_own_wait4=1 group_gone=1 expected_exit=81 runtime_approved=false\n",mode);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"event refusal completion fully paid");return 81;
}

static int child_exit_witness_api(unsigned mode) {
    require(mode<9,"nine supplied child exit witness API cases");reset();
    struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    o.ticket.pid=o.ticket.pgid=100;o.ticket.birth=10;o.ticket.start=20;o.direct_custody=1;o.slots=2;o.source_admitted=1;
    o.event_fd_live=1;o.event_registered_mask=3;o.child_created=1;o.child_image_admitted=1;
    struct engine_member *m=&o.members[1];m->state.pid=200;m->state.ppid=m->state.pgid=100;m->state.birth=11;m->state.start_absolute=22;m->token_known=1;m->phase=ENGINE_EXPECTED;
    m->token.val[5]=200;m->token.val[7]=1;m->user=4000;m->system=3000;
    o.retired_count=1;o.retired[0]=(struct engine_incarnation){.pid=200,.birth=11,.start=22,.token=m->token,.current=1};
    o.event_watch[1]=(struct engine_event_watch){.pid=200,.birth=11,.start=22,.token=m->token,.bound=1};
    // Same process, approved exec changes full audit-token version. The old
    // watch row remains retained; exactly one CURRENT row matches the new token.
    o.retired[0].current=0;m->token.val[7]=2;o.retired[1]=o.retired[0];o.retired[1].current=1;o.retired[1].token=m->token;o.retired_count=2;
    if(mode==2)o.event_registered_mask&=~2u;
    if(mode==3)o.event_watch[1].birth++;
    if(mode==7)o.child_cost_unproven=1;
    struct kevent event;EV_SET(&event,200,EVFILT_PROC,0,mode==1?NOTE_EXEC:NOTE_EXIT,12345,mode==4?NULL:&o.event_watch[1]);
    int note=engine_event_note(&o,&event,1000),first_receipt_retained=1;
    if(note&&mode!=1)first_receipt_retained=engine_event_note(&o,&event,2000)&&m->terminal_observed_ns==1000;
    int terminal=engine_child_note_terminal(&o,mode==5?EIO:ESRCH,mode!=8);
    unsigned mask=27u|(engine_child_terminal_bit(m,terminal,mode!=8)?4u:0u);
    uint64_t leader_user=mode==6?3999:5000,leader_system=4000;
    int qualified=!o.child_cost_unproven&&engine_child_rollup_covers(mask,leader_user,leader_system,m->user,m->system);
    if(!qualified)(void)engine_child_unproven(&o,"retired child lacks qualified nonduplicated same-leader rollup");
    require(private_end_unit(private_cost.open_cpu),"child exit witness API all actual controller CPU fully charged and paid");
    printf("private_child_exit_witness_API_source_known mode=%u note_accepted=%d RI_error_supplied=%d token_gone_supplied=%u received=%u witness=%u exit_abstime=%llu terminal_observed_ns=%llu mask=%u child_cost_unproven=%u qualified=%u native_process=false values_supplied=true actual_child_wait4=false runtime_approved=false\n",
        mode,note,mode==5?EIO:ESRCH,(unsigned)(mode!=8),m->exit_note_received,m->exit_note_witness,(unsigned long long)m->exit_abstime,(unsigned long long)m->terminal_observed_ns,mask,o.child_cost_unproven,(unsigned)qualified);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"child exit witness API raw evidence fully paid before verdict");
    if(!mode)require(qualified&&note&&first_receipt_retained&&m->exit_note_witness&&!m->exit_abstime&&m->terminal_observed_ns==1000&&mask==31&&!o.child_cost_unproven,
        "retained process-bound NOTE_EXIT qualifies RI_ESRCH with unchanged rollup coverage across approved exec");
    else require(!qualified&&o.child_cost_unproven&&!strcmp(private_errors[2].last,"retired child lacks qualified nonduplicated same-leader rollup"),
        "unbound or incomplete child exit witness retains rollup refusal and sticky child-cost state");
    controls++;printf("private_child_exit_witness_API_completed_source_known mode=%u expected_exit=%u native_process=false runtime_approved=false\n",mode,mode?81:0);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"child exit witness API completion fully paid");return mode?81:0;
}
static int leader_rollup_refresh_api(unsigned mode) {
    require(mode<5,"five supplied fresh terminal leader RI API cases");reset();
    struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    o.ticket.pid=o.ticket.pgid=100;o.ticket.birth=10;o.ticket.start=20;o.direct_custody=1;
    struct engine_member *m=&o.members[0];m->state.pid=100;m->state.birth=10;m->state.start_absolute=20;m->token_known=1;
    m->token.val[5]=100;m->token.val[7]=1;m->phase=ENGINE_EXITED;m->exit_abstime=333;m->terminal_observed_ns=1000;
    m->user=100;m->system=200;m->child_user=300;m->child_system=400;o.child_rollup_read_ns=1000;o.members[1].terminal_observed_ns=2000;
    o.retired_count=1;o.retired[0]=(struct engine_incarnation){.pid=100,.birth=10,.start=20,.token=m->token,.current=1};
    struct rusage_info_v1 u;memset(&u,0,sizeof(u));u.ri_proc_start_abstime=20;u.ri_proc_exit_abstime=333;
    u.ri_user_time=100;u.ri_system_time=201;u.ri_child_user_time=300;u.ri_child_system_time=401;
    uint64_t begin=2100,end=2200;int result=mode==1?-1:0;
    if(mode==2)u.ri_proc_start_abstime=21;
    if(mode==3){begin=1500;end=1600;}
    if(mode==4)o.direct_reaped=1;
    int cached_current=engine_leader_rollup_after_terminal(&o);
    int valid=engine_fresh_leader_ri_valid(&o,mode==1?NULL:&u,result,begin,end);
    require(private_end_unit(private_cost.open_cpu),"fresh leader RI API supplied identity and clock guards fully paid");
    printf("private_leader_rollup_refresh_API_source_known mode=%u cached_leader_terminal=true cached_RI_ns=%llu child_receipt_ns=%llu supplied_RI_begin_ns=%llu supplied_RI_return_ns=%llu cached_current=%u fresh_RI_valid=%u native_process=false values_supplied=true runtime_approved=false\n",
        mode,(unsigned long long)o.child_rollup_read_ns,(unsigned long long)o.members[1].terminal_observed_ns,(unsigned long long)begin,(unsigned long long)end,(unsigned)cached_current,(unsigned)valid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"fresh leader RI API raw evidence paid before verdict");
    require(!cached_current,"cached terminal leader RI predating child receipt cannot satisfy fresh rollup observation");
    if(!mode)require(valid,"fresh original terminal leader RI after child receipt satisfies unchanged raw identity and counter guards");
    else require(!valid,"missing foreign stale or unowned fresh leader RI remains unproven");
    controls++;printf("private_leader_rollup_refresh_API_completed_source_known mode=%u expected_exit=%u native_process=false runtime_approved=false\n",mode,mode?81:0);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"fresh leader RI API completion fully paid");return mode?81:0;
}
static void knote_rollup_child(const char *report) {
    pid_t child=fork();if(child<0)_exit(77);if(!child){raise(SIGSTOP);burn_cpu(3000000);}
    FILE *out=fopen(report,"wx");
    if(!out||fprintf(out,"%d\n",child)<0||fflush(out)) {
        (void)kill(child,SIGKILL);while(waitpid(child,NULL,0)<0&&errno==EINTR){};_exit(77);
    }
    raise(SIGSTOP);int status=0;struct rusage usage;pid_t got;
    do{got=wait4(child,&status,0,&usage);}while(got<0&&errno==EINTR);
    if(got!=child||!WIFEXITED(status)||WEXITSTATUS(status))_exit(77);
    if(fprintf(out,"%llu\n",(unsigned long long)ru_cpu(&usage))<0||fclose(out))_exit(77);_exit(0);
}
static void child_exit_witness_control(const char *self,const char *root,int dir) {
    struct engine_owner o;struct private_engine_result r;int status=-1;pid_t child_pid=0;
    require(total_spawn(&o,&r,self,dir,26,root),"original child exit witness leader passes unchanged native and QoS gates");
    char report[4097];snprintf(report,sizeof(report),"%s/knote-rollup-26",root);
    for(;;) {
        FILE *f=fopen(report,"r");if(f){(void)fscanf(f,"%d",&child_pid);fclose(f);}
        struct image leader;if(child_pid&&control_observe(o.ticket.pid,&leader)&&leader.status==SSTOP)break;
        require(clock_ns(CLOCK_MONOTONIC)<outer.deadline&&engine_checkpoint(&o),"bounded original child exit witness discovery");usleep(1000);
    }
    struct image child;require(observe(child_pid,&child)&&child.ppid==o.ticket.pid&&child.pgid==o.ticket.pgid&&child.status==SSTOP&&child.priority==4,
        "actual stopped child belongs to original leader before process-bound watch");
    o.members[1].state=child;o.members[1].phase=ENGINE_EXPECTED;o.child_image_admitted=1;
    require(known_control_token(child_pid,child.birth,&o.members[1].token),"actual child full token precedes its immutable knote binding");o.members[1].token_known=1;
    require(engine_owned_token(&o,child_pid,child.birth,child.start_absolute,&o.members[1].token),"retain original stopped child identity and sampled counters");
    struct rusage_info_v1 initial;require(proc_pid_rusage(child_pid,RUSAGE_INFO_V1,(rusage_info_t *)&initial)==0&&engine_usage(&o,1,&initial),"actual child RI start and last live counters precede its watch");
    o.child_created=1;r.created++;r.members_max=2;o.child_discovered_via=1;
    require(engine_events_register(&o,1,child_pid)&&o.event_watch[1].bound&&(o.event_registered_mask&2),"actual child NOTE_EXIT watch is registered under retained current full identity");
    require(engine_checkpoint(&o),"actual child identity and watch setup fully paid before fixture continuation");
    require(kill(-o.ticket.pgid,SIGCONT)==0,"one fixture continuation releases only original stopped leader and child");
    await_original(&o);
    struct rusage_info_v1 unavailable;errno=0;int child_ri=proc_pid_rusage(child_pid,RUSAGE_INFO_V1,(rusage_info_t *)&unavailable);int child_error=child_ri?errno:0;
    // Cache a GENUINE terminal leader RI before draining the child queue.
    // This regression must exercise qualification's late-receipt refresh,
    // rather than the ordinary terminal path that already reads leader RI.
    require(engine_member_terminal(&o,0)&&o.members[0].phase==ENGINE_EXITED&&o.members[0].exit_abstime&&o.child_rollup_read_ns,
        "actual terminal leader RI is cached before late child NOTE_EXIT receipt");
    uint64_t cached_leader_read=o.child_rollup_read_ns,first_leader_terminal=o.members[0].terminal_observed_ns;
    require(engine_checkpoint(&o),"genuine cached leader RI observation fully paid before child event drain");
    require(engine_events_poll(&o,0)&&o.members[1].exit_note_received&&o.members[1].terminal_observed_ns>cached_leader_read,
        "actual child NOTE_EXIT is first received after cached leader terminal RI");
    int child_terminal=engine_member_terminal(&o,1),children[3]={0};int count=engine_list_children(&o,0,o.ticket.pid,children,sizeof(children));
    int qualified=engine_qualify_child_rollup(&o,child_terminal,native_pid_list_empty(count,o.child_list_errno[0]),1);
    require(qualified&&o.child_rollup_read_ns>cached_leader_read&&o.child_rollup_read_ns>=o.members[1].terminal_observed_ns
        &&o.members[0].terminal_observed_ns==first_leader_terminal,
        "late child NOTE_EXIT refresh bypasses cached leader terminal RI before rollup qualification");
    require(child_ri!=0&&child_error==ESRCH&&o.child_rollup_known&&o.child_rollup_mask==31
        &&o.members[1].exit_note_witness&&!o.members[1].exit_abstime&&o.members[1].terminal_observed_ns
        &&o.child_rollup_read_ns>=o.members[1].terminal_observed_ns&&!o.child_cost_unproven,
        "actual leader-reaped child uses retained NOTE_EXIT RI_ESRCH and later covered leader rollup");
    (void)engine_retire(&o,&status);
    int settled=o.direct_reaped&&o.wait_observed&&r.group_gone&&r.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    unsigned long long child_cpu=0;pid_t reported=0;FILE *f=fopen(report,"r");
    require(f&&fscanf(f,"%d\n%llu",&reported,&child_cpu)==2&&reported==child_pid&&fclose(f)==0,"same scripted leader actually consumed its own child wait4 once");
    require(settled&&status==0&&!o.failed&&!private_failed&&r.created==2&&r.reaped==2&&r.helper_job_cpu_ns==o.terminal_cpu_ns
        &&o.terminal_cpu_ns>=child_cpu&&o.helper_cost_retained&&o.helper_cost_paid
        &&o.helper_debt_delta_ns==9*engine_helper_cpu_cost(o.terminal_cpu_ns)&&private_cost.debt==private_cost.debt_paid,
        "actual child exit witness qualifies with one authoritative subtree bill full9x debt and original group settlement");
    printf("private_child_exit_witness_process_source_known original_pid=%d child_pid=%d RI_result=%d RI_errno=%d child_exit_witness=kqueue_note_exit child_exit_abstime=0 terminal_observed_ns=%llu cached_leader_RI_ns=%llu first_leader_terminal_ns=%llu later_leader_RI_ns=%llu rollup_mask=%u child_cost_unproven=%u actual_leader_own_wait4=1 original_group_gone=1 controller_reaps_child=false leader_child_wait4_CPU_ns=%llu authoritative_subtree_CPU_ns=%llu helper_debt_delta_ns=%llu helper_debt_before_ns=%llu helper_debt_target_ns=%llu global_paid_ns=%llu GS1_fixture_projection=true installed_tool_qualification=false runtime_approved=false\n",
        o.ticket.pid,child_pid,child_ri,child_error,(unsigned long long)o.members[1].terminal_observed_ns,(unsigned long long)cached_leader_read,(unsigned long long)first_leader_terminal,(unsigned long long)o.child_rollup_read_ns,o.child_rollup_mask,o.child_cost_unproven,child_cpu,(unsigned long long)o.terminal_cpu_ns,(unsigned long long)o.helper_debt_delta_ns,(unsigned long long)o.helper_debt_before,(unsigned long long)o.helper_debt_target,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"actual child exit witness numeric evidence fully paid");
    engine_close_transferred(&o);require(close(o.ledger)==0,"actual child exit witness ledger close");free(o.capture);
    require(private_end_unit(private_cost.open_cpu),"actual child exit witness disposal fully paid");controls++;
}

// Source-known scripted verifier, never an installed Git/GPG qualification.
static struct private_git_sig_temp same_wake_sig;
static unsigned char same_wake_signature_buffer[4096];
static char same_wake_gpg_json[8194],same_wake_root[4097];
static unsigned same_wake_exec_released,same_wake_unstopped_rest,same_wake_ack_seen,same_wake_exit_released;
static uint64_t same_wake_ack_unit_cpu;
struct same_wake_handoff_row {unsigned stage,steps,phase,pending,attachment_stop,registered,exec_released,result;uint64_t open_cpu,cpu_now,used,wall_ns;};
static struct {uint64_t rows,polls,max_used;struct same_wake_handoff_row first,last;} same_wake_handoff;
static uint64_t same_wake_watch_release_ns;
static int same_wake_execute_fd=-1,same_wake_write_result=-2,same_wake_write_errno;
static struct stat same_wake_execute_stat;
static uint64_t same_wake_write_begin_CPU,same_wake_write_end_CPU;
static void same_wake_release_execute(void) {
    if(same_wake_exec_released)return;
    same_wake_write_begin_CPU=clock_ns(CLOCK_THREAD_CPUTIME_ID);errno=0;
    same_wake_write_result=(int)write(same_wake_execute_fd,"x",1);same_wake_write_errno=same_wake_write_result<0?errno:0;
    same_wake_write_end_CPU=clock_ns(CLOCK_THREAD_CPUTIME_ID);same_wake_watch_release_ns=clock_ns(CLOCK_MONOTONIC);
    require(same_wake_execute_fd>=0&&same_wake_write_result==1,"prepared original execute FIFO accepts one nonblocking byte");same_wake_exec_released=1;
}

static void same_wake_handoff_retain(struct engine_owner *o,unsigned stage,unsigned steps,uint64_t used,int result) {
    struct same_wake_handoff_row row={.stage=stage,.steps=steps,.phase=o->members[1].phase,.pending=o->child_event_pending,
        .attachment_stop=o->members[1].attachment_stop,.registered=o->event_registered_mask,.exec_released=same_wake_exec_released,
        .result=(unsigned)result,.open_cpu=private_cost.open_cpu,.cpu_now=clock_ns(CLOCK_THREAD_CPUTIME_ID),.used=used,.wall_ns=clock_ns(CLOCK_MONOTONIC)};
    if(!same_wake_handoff.rows)same_wake_handoff.first=row;
    same_wake_handoff.last=row;same_wake_handoff.rows++;if(stage==5)same_wake_handoff.polls++;
    if(used!=UINT64_MAX&&used>same_wake_handoff.max_used)same_wake_handoff.max_used=used;
}

static void same_wake_path(char out[4097],const char *root,const char *name) {
    int n=snprintf(out,4097,"%s/%s",root,name);require(n>0&&n<4097,"bounded same-wake owned path");
}
static void same_wake_create_gate(const char *name) {
    char path[4097];same_wake_path(path,same_wake_root,name);
    int fd=open(path,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require((fd>=0&&close(fd)==0)||(fd<0&&errno==EEXIST),"same-wake fixture creates only its original gate");
}
static void same_wake_watch_ready(struct engine_owner *o) {
    if(same_wake_exec_released||!engine_child_watch_matches(o))return;
    same_wake_release_execute();
}
static void same_wake_rest_observer(void) {
    struct engine_owner *o=fixture_live_owner;
    if(!o||!same_wake_exec_released||same_wake_exit_released)return;
    if(o->child_image_admitted){same_wake_create_gate("exit");same_wake_exit_released=1;return;}
    if(same_wake_ack_seen)return;
    struct image current;
    if(control_observe(o->members[1].state.pid,&current)&&current.birth==o->members[1].state.birth
        &&current.pgid==o->ticket.pgid&&current.status==SSTOP&&!current.runnable) {
        same_wake_ack_seen=1;same_wake_ack_unit_cpu=private_cost.last_unit;return;
    }
    // The adversarial scripted verifier exits during an erroneously inserted
    // pre-ACK rest. The real STOP/ACK path leaves it retained across full9x.
    same_wake_unstopped_rest=1;same_wake_create_gate("exit");same_wake_exit_released=1;
}
static void same_wake_prepare(struct engine_owner *o,const char *root,int dir) {
    strcpy(same_wake_root,root);same_wake_exec_released=same_wake_unstopped_rest=same_wake_ack_seen=same_wake_exit_released=0;same_wake_ack_unit_cpu=0;same_wake_watch_release_ns=0;memset(&same_wake_handoff,0,sizeof(same_wake_handoff));same_wake_write_result=-2;same_wake_write_errno=0;same_wake_write_begin_CPU=same_wake_write_end_CPU=0;
    strcpy(outer.output.path,root);outer.output.fd=dir;
    basis.fixture.valid_signature.path=(struct private_slice){(const unsigned char *)"\"valid-signature\"",17};
    same_wake_path(o->descendant,root,"verifier");
    int n=snprintf(same_wake_gpg_json,sizeof(same_wake_gpg_json),"\"%s\"",o->descendant);
    require(n>0&&(size_t)n<sizeof(same_wake_gpg_json),"actual copied verifier JSON path");
    basis.build.tools[1].ref.path=(struct private_slice){(const unsigned char *)same_wake_gpg_json,(size_t)n};
    memset(&same_wake_sig,0,sizeof(same_wake_sig));char scratch[4097];same_wake_path(scratch,root,"t");
    require(private_git_sig_prepare(&context,&same_wake_sig,scratch)&&private_end_unit(private_cost.open_cpu),"real same-wake GS1 scratch and selected bytes prepared and paid before helper birth");
    o->command=12;o->git_signature=&same_wake_sig;o->signature_buffer=same_wake_signature_buffer;o->signature_capacity=sizeof(same_wake_signature_buffer);
}
static void same_wake_verifier(const char *signature) {
    char root[4097],gate[4097];size_t n=strlen(signature);
    if(n<22||n>=sizeof(root))_exit(77);memcpy(root,signature,n+1);char *t=strstr(root,"/t/.git_vtag_tmp");if(!t)_exit(77);*t=0;
    int length=snprintf(gate,sizeof(gate),"%s/exit",root);if(length<=0||(size_t)length>=sizeof(gate))_exit(77);
    while(access(gate,F_OK)!=0)usleep(100);
    _exit(0);
}
static void same_wake_leader(const char *root) {
    char verifier[4097],signature[4097],execute[4097],source[4097],report[4097];
    int a=snprintf(verifier,sizeof(verifier),"%s/verifier",root),b=snprintf(signature,sizeof(signature),"%s/t/.git_vtag_tmpAa1Bb2",root);
    int c=snprintf(execute,sizeof(execute),"%s/execute",root),d=snprintf(source,sizeof(source),"%s/valid-signature",root),e=snprintf(report,sizeof(report),"%s/admission-child",root);
    if(a<=0||(size_t)a>=sizeof(verifier)||b<=0||(size_t)b>=sizeof(signature)||c<=0||(size_t)c>=sizeof(execute)||d<=0||(size_t)d>=sizeof(source)||e<=0||(size_t)e>=sizeof(report))_exit(77);
    unsigned char bytes[512];int in=open(source,O_RDONLY|O_NOFOLLOW|O_CLOEXEC),out=open(signature,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    ssize_t count=in<0?-1:read(in,bytes,sizeof(bytes));
    if(count!=228||out<0||write(out,bytes,(size_t)count)!=count||close(in)||close(out))_exit(77);
    pid_t child=fork();if(child<0)_exit(77);
    if(!child){int gate=open(execute,O_RDONLY|O_NOFOLLOW|O_CLOEXEC);char byte;ssize_t read_count;
        if(gate<0)_exit(77);do{read_count=read(gate,&byte,1);}while(read_count<0&&errno==EINTR);
        if(read_count!=1||byte!='x'||close(gate))_exit(77);
        // Actual source-known slow EXEC: remain this CLONE for at least750us
        // AFTER the retained child watch releases its original FIFO gate.
        uint64_t begin=clock_ns(CLOCK_MONOTONIC),now=begin;
        if(begin==UINT64_MAX||begin>UINT64_MAX-750000)_exit(77);
        while(now-begin<750000){usleep(100);now=clock_ns(CLOCK_MONOTONIC);if(now==UINT64_MAX||now<begin)_exit(77);}
        char delay_path[4097];int length=snprintf(delay_path,sizeof(delay_path),"%s/slow-exec-delay",root);
        if(length<=0||(size_t)length>=sizeof(delay_path))_exit(77);
        FILE *delay=fopen(delay_path,"wx");if(!delay||fprintf(delay,"%d %llu %llu\n",getpid(),(unsigned long long)begin,(unsigned long long)now)<0||fclose(delay))_exit(77);
        char *args[]={verifier,"--keyid-format=long","--status-fd=1","--verify",signature,"-",NULL};execv(verifier,args);_exit(77);}
    int status=0;struct rusage usage;pid_t got;do{got=wait4(child,&status,0,&usage);}while(got<0&&errno==EINTR);
    if(got!=child||!WIFEXITED(status)||WEXITSTATUS(status)||unlink(signature))_exit(77);
    FILE *f=fopen(report,"wx");if(!f||fprintf(f,"%d %llu\n",child,(unsigned long long)ru_cpu(&usage))<0||fclose(f))_exit(77);_exit(0);
}
static void same_wake_copy(int from,const char *source,int to,const char *target,mode_t mode) {
    int input=openat(from,source,O_RDONLY|O_NOFOLLOW|O_CLOEXEC),output=openat(to,target,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,mode);
    require(input>=0&&output>=0,"retain exact same-wake fixture copy descriptors");unsigned steps=0;unsigned char bytes[4096];
    for(;;) {
        ssize_t n=read(input,bytes,sizeof(bytes));require(n>=0,"same-wake fixture copy read");if(!n)break;
        require(write(output,bytes,(size_t)n)==n,"same-wake fixture copy write");
        uint64_t used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
        if(++steps>=64||used>=400000){require(private_end_unit(private_cost.open_cpu),"bounded same-wake fixture copy fully paid");steps=0;}
    }
    require(close(input)==0&&close(output)==0&&private_end_unit(private_cost.open_cpu),"same-wake fixture copy final cost and original descriptors paid");
}
static void same_wake_admission_control(const char *self,const char *root,int parent) {
    reset();require(mkdirat(parent,"same-wake",0700)==0,"fresh same-wake fixture directory");
    int dir=openat(parent,"same-wake",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0&&mkdirat(dir,"t",0700)==0,"owned same-wake scratch");
    char owned[4097];same_wake_path(owned,root,"same-wake");
    same_wake_copy(AT_FDCWD,self,dir,"verifier",0500);same_wake_copy(parent,"valid-signature",dir,"valid-signature",0600);
    // Fixture setup is fully paid BEFORE birth. The critical watch callback
    // only writes one nonblocking byte; no pathname creation/timer wake is
    // manufactured inside the product's bounded capture/EXEC span.
    require(same_wake_execute_fd<0&&mkfifoat(dir,"execute",0600)==0,"create only original owned execute FIFO before birth");
    same_wake_execute_fd=openat(dir,"execute",O_RDWR|O_NONBLOCK|O_NOFOLLOW|O_CLOEXEC);
    require(same_wake_execute_fd>=0&&fstat(same_wake_execute_fd,&same_wake_execute_stat)==0
        &&S_ISFIFO(same_wake_execute_stat.st_mode)&&same_wake_execute_stat.st_uid==geteuid()
        &&same_wake_execute_stat.st_nlink==1&&(same_wake_execute_stat.st_mode&0777)==0600
        &&private_end_unit(private_cost.open_cpu),"prepared original execute FIFO retained and fully paid before helper birth");
    struct engine_owner o;struct private_engine_result r;int status=-1;require(total_spawn(&o,&r,self,dir,27,owned),"same-wake original prepared leader obeys unchanged native and QoS gates");
    source_known_child_watch_bound=same_wake_watch_ready;source_known_child_handoff=same_wake_handoff_retain;private_rest_control=same_wake_rest_observer;
    const char *first_code="none",*first_detail="none";
    for(;;) {
        int done=0;require(known_exited(o.ticket.pid,&done),"same-wake original leader WNOWAIT remains owned");if(done)break;
        if(!engine_events_service(&o,1000000)){first_code=private_error;first_detail=private_detail;break;}
        if(o.child_image_admitted)break;
        // Keep the gated leader's actual RI fresh without invoking the group
        // census whose independent peak update would hide an admission bug.
        if(!engine_sample_member(&o,0,o.ticket.pid)||!engine_checkpoint(&o)){first_code=private_error;first_detail=private_detail;break;}
    }
    unsigned actual_peak_at_admission=r.members_max;
    source_known_child_watch_bound=NULL;source_known_child_handoff=NULL;private_rest_control=NULL;
    // Ensure the adversarial gap ends with THIS helper parent consuming its
    // child. Failure assertion uses existing exact fixture cleanup, not a new
    // product quarantine-exit or any fabricated child own-wait4.
    same_wake_release_execute();same_wake_create_gate("exit");await_original(&o);
    const struct same_wake_handoff_row *first=&same_wake_handoff.first,*last=&same_wake_handoff.last;
    printf("private_same_wake_handoff_diagnostic_source_known first_error=%s first_detail=\"%s\" rows=%llu zero_timeout_poll_calls=%llu max_open_CPU_ns=%llu first_stage=%u first_phase=%u first_open_CPU_ns=%llu first_CPU_ns=%llu first_wall_ns=%llu last_stage=%u last_guard_steps=%u last_phase=%u last_pending=%u last_STOP=%u registered_mask=%u exec_gate_released=%u watch_release_ns=%llu last_open_CPU_ns=%llu last_CPU_ns=%llu last_used_CPU_ns=%llu last_wall_ns=%llu last_result=%u no_timing_cause_inferred=true runtime_approved=false\n",
        first_code,first_detail,(unsigned long long)same_wake_handoff.rows,(unsigned long long)same_wake_handoff.polls,(unsigned long long)same_wake_handoff.max_used,
        first->stage,first->phase,(unsigned long long)first->open_cpu,(unsigned long long)first->cpu_now,(unsigned long long)first->wall_ns,
        last->stage,last->steps,last->phase,last->pending,last->attachment_stop,last->registered,last->exec_released,(unsigned long long)same_wake_watch_release_ns,
        (unsigned long long)last->open_cpu,(unsigned long long)last->cpu_now,(unsigned long long)last->used,(unsigned long long)last->wall_ns,last->result);
    printf("private_same_wake_execute_wakeup_source_known prepared_before_birth=true original_FIFO_dev=%llu original_FIFO_ino=%llu write_result=%d write_errno=%d write_begin_CPU_ns=%llu write_end_CPU_ns=%llu child_uses_blocking_read=true child_execute_timer_poll=false controller_write_nonblocking=true runtime_approved=false\n",
        (unsigned long long)same_wake_execute_stat.st_dev,(unsigned long long)same_wake_execute_stat.st_ino,same_wake_write_result,same_wake_write_errno,
        (unsigned long long)same_wake_write_begin_CPU,(unsigned long long)same_wake_write_end_CPU);
    printf("private_same_wake_admission_axis_source_known original_pid=%d child_pid=%d fork_events=%llu child_image_admitted=%u GS1_bound=%u attachment_STOP=%u unstopped_rest=%u own_wait4_pending=true native_process=true actual_GS1=true installed_tool_qualification=false runtime_approved=false\n",
        o.ticket.pid,o.members[1].state.pid,(unsigned long long)o.leader_fork_events,o.child_image_admitted,same_wake_sig.bound,o.members[1].attachment_stop,same_wake_unstopped_rest);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"same-wake actual admission and failed-gap evidence fully paid before verdict");
    require(o.child_image_admitted&&same_wake_sig.bound&&o.verified_descendants==1&&!same_wake_unstopped_rest&&same_wake_ack_seen&&same_wake_ack_unit_cpu<=1000000,
        "actual fast verifier keeps same-wake image admission and GS1 before any unstopped rest");
    (void)engine_retire(&o,&status);int settled=o.direct_reaped&&o.wait_observed&&r.group_gone&&r.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&status==0&&!o.failed&&!private_failed&&same_wake_sig.settled&&o.child_rollup_mask==31
        &&o.helper_cost_retained&&o.helper_cost_paid&&private_cost.debt==private_cost.debt_paid&&!o.event_fd_live,
        "same-wake actual GS1 verifier reaches unchanged GS2 subtree bill full debt ownreap groupgone and queue close");
    unsigned long long clone_begin=0,clone_end=0;int delay_pid=0;char delay_path[4097];same_wake_path(delay_path,owned,"slow-exec-delay");
    FILE *delay=fopen(delay_path,"r");require(delay&&fscanf(delay,"%d %llu %llu",&delay_pid,&clone_begin,&clone_end)==3&&fclose(delay)==0
        &&delay_pid==o.members[1].state.pid&&clone_begin&&clone_end>=clone_begin&&clone_end-clone_begin>=750000,
        "actual same-original child remains CLONE for at least750us before its observed verifier EXEC");
    printf("private_same_wake_slow_exec_source_known child_pid=%d clone_begin_ns=%llu clone_end_ns=%llu clone_delay_ns=%llu minimum_delay_ns=750000 actual_clock_reads=true actual_GS1=true native_process=true installed_tool_qualification=false runtime_approved=false\n",delay_pid,clone_begin,clone_end,clone_end-clone_begin);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"actual slow-EXEC clock evidence fully paid after original settlement");
    printf("private_same_wake_admission_completed_source_known original_pid=%d child_pid=%d child_image_admitted=1 GS1_bound=1 GS2_settled=1 rollup_mask=31 actual_own_wait4=1 group_gone=1 helper_CPU_ns=%llu helper_debt_delta_ns=%llu debt_ns=%llu paid_ns=%llu native_process=true installed_tool_qualification=false runtime_approved=false\n",
        o.ticket.pid,o.members[1].state.pid,(unsigned long long)o.terminal_cpu_ns,(unsigned long long)o.helper_debt_delta_ns,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"same-wake completed raw evidence paid");
    engine_close_transferred(&o);require(close(o.ledger)==0&&private_git_sig_release(&same_wake_sig),"same-wake original ledger and settled signature custody closed");free(o.capture);
    int execute_fd=same_wake_execute_fd;same_wake_execute_fd=-1;
    require(execute_fd>=0&&close(execute_fd)==0&&private_end_unit(private_cost.open_cpu),"original execute FIFO writer held through actual ownreap and groupgone then paid close");
    char fifo_receipt[512];int fifo_len=snprintf(fifo_receipt,sizeof(fifo_receipt),"{\"fixture_only\":true,\"original_FIFO_dev\":%llu,\"original_FIFO_ino\":%llu,\"writer_closed_after_ownwait4_groupgone\":true,\"nonblocking_write_result\":%d,\"nonblocking_write_errno\":%d}\n",
        (unsigned long long)same_wake_execute_stat.st_dev,(unsigned long long)same_wake_execute_stat.st_ino,same_wake_write_result,same_wake_write_errno);
    int receipt_fd=openat(dir,"execute-fifo.json",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(fifo_len>0&&(size_t)fifo_len<sizeof(fifo_receipt)&&receipt_fd>=0&&write(receipt_fd,fifo_receipt,(size_t)fifo_len)==fifo_len
        &&close(receipt_fd)==0&&private_end_unit(private_cost.open_cpu),"original FIFO scalar receipt separately paid after physical settlement");
    for(unsigned i=0;i<private_guards.count;i++)if(private_guards.files[i].fd>=0) {require(close(private_guards.files[i].fd)==0,"same-wake original selected input guard closed");private_guards.files[i].fd=-1;}
    private_guards.count=0;
    printf("private_same_wake_STOP_axis_source_known positive_ACK_before_first_rest=%u actual_full_unit_CPU_ns=%llu hard_unit_ns=1000000 nominal_target_ns=500000 no_OS_WCET_claim=true runtime_approved=false\n",same_wake_ack_seen,(unsigned long long)same_wake_ack_unit_cpu);
    require(fflush(stdout)==0&&close(dir)==0&&private_end_unit(private_cost.open_cpu),"same-wake exact original fixture disposal fully paid");
    // This process witness ran ONE real leader+child. Only its captured live
    // peak is inserted into the full-case CLASSIFIER projection; the other
    // fifteen-command terms are explicitly supplied, never physical claims.
    struct private_engine_result projected;nominal_result(&projected);projected.members_max=(uint8_t)actual_peak_at_admission;
    struct engine_final_conjunction final;engine_final_conjunction_capture(&context,&projected,&final);
    int final_accepted=engine_complete_result(&context,&projected);
    printf("private_same_wake_members_conjunction_source_known actual_members_max=%u census_peak_path_bypassed=true actual_GS1_GS2_settled=true actual_own_wait4_groupgone=true actual_full_bill_paid=true final_conjunction_passed=%s first_failing_term=\"%s\" final_error_code=%s other_final_terms_supplied=true full15_physical_run_claimed=false runtime_approved=false\n",
        actual_peak_at_admission,final_accepted?"true":"false",final.first_failing_term,final_accepted?"none":private_error);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"same-wake final peak projection evidence paid after original physical and FD settlement");
    require(actual_peak_at_admission==2&&final_accepted&&final.coverage_passed&&final.resources_passed&&!strcmp(final.first_failing_term,"none"),
        "actual same-wake retained-live peak reaches final conjunction after full original settlement");controls++;
}

static int same_wake_api_capture(struct engine_owner *o,unsigned member,pid_t pid) {
    require(member==1&&pid==200,"same-wake API supplied child capture recipient");
    o->child_created=1;o->members[1].phase=ENGINE_CLONE;return 1;
}
static void same_wake_admission_api(void) {
    reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    uint64_t original_epoch=private_cost.open_cpu,before=private_cost.debt;
    int capture=engine_child_capture_same_unit(&o,200,same_wake_api_capture);
    int waited=private_cost.open_cpu!=original_epoch||private_cost.debt!=before;
    // A supplied scripted transition closes the only pre-ACK window if a
    // closed/paid unit is inserted. No OS timing/GS1/reap is invented here.
    unsigned supplied_stopped_ACK=!waited,supplied_GS1=!waited,mask=30u|(supplied_GS1?1u:0u);
    int qualified=engine_child_rollup_covers(mask,5000,4000,4000,3000);
    if(!qualified)(void)engine_child_unproven(&o,"retired child lacks qualified nonduplicated same-leader rollup");
    require(private_end_unit(private_cost.open_cpu),"same-wake API actual controller full cost paid");
    printf("private_same_wake_admission_API_source_known captured=%u closed_paid_unit_before_ACK=%u supplied_stopped_ACK=%u supplied_GS1=%u supplied_rollup_mask=%u sticky_child_cost_unproven=%u native_process=false values_supplied=true actual_GS1=false actual_own_wait4=false runtime_approved=false\n",
        (unsigned)capture,(unsigned)waited,supplied_stopped_ACK,supplied_GS1,mask,o.child_cost_unproven);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"same-wake API supplied transition evidence fully paid");
    require(capture&&!waited&&supplied_stopped_ACK&&supplied_GS1&&qualified&&mask==31&&!o.child_cost_unproven,
        "same-wake admission API keeps pre-STOP window open until positive custody ACK");
    controls++;
}

static void custody_stop_identity_api(void) {
    reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    o.ticket.pid=o.ticket.pgid=100;o.ticket.birth=10;o.ticket.start=20;o.direct_custody=1;o.slots=2;o.child_created=1;
    o.event_fd_live=1;o.event_registered_mask=3;struct engine_member *m=&o.members[1];m->phase=ENGINE_CLONE;
    m->state.pid=200;m->state.ppid=m->state.pgid=100;m->state.birth=11;m->state.start_absolute=22;m->token_known=1;m->token.val[5]=200;m->token.val[7]=1;
    o.retired_count=1;o.retired[0]=(struct engine_incarnation){.pid=200,.birth=11,.start=22,.token=m->token,.current=1};
    o.event_watch[1]=(struct engine_event_watch){.pid=200,.birth=11,.start=22,.token=m->token,.bound=1};
    unsigned accepted=0;const unsigned order[]={1,0,2,3,4,5,6};
    for(unsigned j=0;j<7;j++) {
        unsigned which=order[j];struct rusage_info_v1 u;memset(&u,0,sizeof(u));u.ri_proc_start_abstime=22;
        o.failed=0;o.retired[0].current=1;o.event_watch[1].start=22;
        if(which==1)u.ri_proc_start_abstime=23; // Fresh foreign RI despite cached matching birth/watch/token.
        if(which==2)u.ri_proc_start_abstime=0;
        if(which==3)u.ri_proc_exit_abstime=33;
        if(which==4)o.failed=1;
        if(which==5)o.retired[0].current=0;
        if(which==6)o.event_watch[1].start=23;
        if(engine_custody_stop_identity(&o,&u))accepted|=1u<<which;
    }
    require(private_end_unit(private_cost.open_cpu),"custody STOP identity API actual full controller CPU paid");
    printf("private_custody_STOP_identity_API_source_known cases=7 accepted_mask=%u expected_mask=1 foreign_RI_zero_RI_terminal_RI_failed_admission_noncurrent_row_wrong_watch_refused=true native_process=false values_supplied=true actual_SIGSTOP=false runtime_approved=false\n",accepted);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"custody STOP identity API exact supplied evidence paid before verdict");
    require(accepted==1,"fresh current RI and sticky admission state gate the one verifier custody STOP");controls++;
}

static void same_wake_event_wait_api(void) {
    reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    uint64_t supplied_CPU=200000,supplied_wall=0;unsigned steps=0,admitted=0;
    while(engine_child_handoff_headroom(supplied_CPU)&&steps<64) {
        uint64_t event_wait=engine_child_handoff_event_wait(0,0);
        supplied_CPU+=20000;supplied_wall+=event_wait?750000:10000;steps++;
        if(supplied_wall>=750000){admitted=1;break;}
    }
    unsigned mask=30u|(admitted?1u:0u);if(!admitted)(void)engine_child_unproven(&o,"verifier image admission not retained in bounded same-wake unit");
    require(private_end_unit(private_cost.open_cpu),"event-wait API actual scalar-control CPU fully paid");
    printf("private_same_wake_event_wait_API_source_known admitted=%u supplied_CPU_ns=%llu supplied_elapsed_ns=%llu supplied_steps=%u supplied_rollup_mask=%u sticky_child_cost_unproven=%u native_process=false values_supplied=true OS_CPU_claimed=false runtime_approved=false\n",
        admitted,(unsigned long long)supplied_CPU,(unsigned long long)supplied_wall,steps,mask,o.child_cost_unproven);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"event-wait API supplied causal evidence fully paid before verdict");
    require(admitted&&engine_child_handoff_headroom(supplied_CPU)&&supplied_wall>=750000&&steps<=64&&mask==31&&!o.child_cost_unproven
        &&engine_child_handoff_event_wait(1,0)==0&&engine_child_handoff_event_wait(0,1)==0&&engine_child_handoff_event_wait(0,0)<=1000000,
        "bounded event wait admits slow-exec projection without exhausting the unchanged CPU guard");controls++;
}
static int same_wake_exit_api(void) {
    reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
    o.ticket.pid=o.ticket.pgid=100;o.ticket.birth=10;o.ticket.start=20;o.direct_custody=1;o.slots=2;o.child_created=1;
    o.event_fd_live=1;o.event_registered_mask=3;struct engine_member *m=&o.members[1];m->phase=ENGINE_CLONE;
    m->state.pid=200;m->state.ppid=m->state.pgid=100;m->state.birth=11;m->state.start_absolute=22;m->token_known=1;m->token.val[5]=200;m->token.val[7]=1;
    o.retired_count=1;o.retired[0]=(struct engine_incarnation){.pid=200,.birth=11,.start=22,.token=m->token,.current=1};
    o.event_watch[1]=(struct engine_event_watch){.pid=200,.birth=11,.start=22,.token=m->token,.bound=1};
    struct kevent note;EV_SET(&note,200,EVFILT_PROC,0,NOTE_EXEC,0,&o.event_watch[1]);
    int exec_note=engine_event_note(&o,&note,500),exec_only_open=engine_child_admission_terminal_guard(&o)&&!m->terminal_observed_ns;
    EV_SET(&note,200,EVFILT_PROC,0,NOTE_EXIT,0,&o.event_watch[1]);int exit_note=engine_event_note(&o,&note,1000);
    int admitted=engine_child_admission_terminal_guard(&o);
    int exact=exec_note&&exec_only_open&&exit_note&&!admitted&&o.child_cost_unproven&&!o.child_image_admitted&&!m->attachment_stop
        &&!strcmp(private_error,"coverage_unknown")&&!strcmp(private_detail,"verifier exited before admission");
    require(private_end_unit(private_cost.open_cpu),"pre-admission EXIT API actual full controller CPU paid");
    printf("private_same_wake_exit_API_source_known EXEC_only_not_terminal=%u retained_EXIT_received=%u image_admitted=%u custody_STOP=%u sticky_child_cost_unproven=%u distinct_detail=\"%s\" native_process=false values_supplied=true actual_GS1=false actual_own_wait4=false runtime_approved=false\n",
        (unsigned)exec_only_open,m->exit_note_received,o.child_image_admitted,m->attachment_stop,o.child_cost_unproven,private_detail);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"pre-admission EXIT API exact refusal evidence paid before verdict");
    require(exact,"retained child NOTE_EXIT before image STOP ACK refuses with exact pre-admission detail");controls++;return 81;
}

static void sample_unit_composition_control(void) {
    reset();
    // A deterministic COST INPUT vector, not measured phase attribution. Its
    // total equals retained row87 including the792ns ending tail. The source
    // record did not identify individual phase costs or a unique caller.
    const uint64_t supplied[3]={180000,400000,440958};uint64_t sum=0,open=0,maximum=0,projected_debt=0;unsigned units=0;
    for(unsigned i=0;i<3;i++) {
        sum+=supplied[i];open+=supplied[i];
        if(i==2||engine_unit_boundary_enabled(i)) {
            if(open>maximum)maximum=open;projected_debt+=9*(open<1000000?1000000:open);units++;open=0;
        }
    }
    require(private_end_unit(private_cost.open_cpu),"unit composition API actual controller cost paid");
    printf("private_sample_unit_composition_source_known supplied_costs=true native_process=false retained_total_ns=1020958 composed_total_ns=%llu phase_units=%u phase_max_ns=%llu nominal_target_ns=500000 hard_unit_ns=1000000 projected_debt_ns=%llu OS_WCET_claimed=false runtime_approved=false\n",
        (unsigned long long)sum,units,(unsigned long long)maximum,(unsigned long long)projected_debt);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"unit composition projection evidence fully paid");
    require(sum==1020958&&!open&&units==3&&maximum<=500000&&maximum<=1000000&&projected_debt==27000000,
        "retained-child sample composition closes every full-cost phase before the one millisecond hard bound");
    controls++;
}
static void currentness_fixture_platform(void) {
    struct utsname host;size_t length=sizeof(basis.platform.os_build);
    require(uname(&host)==0&&strlen(host.machine)<sizeof(basis.platform.machine)&&strlen(host.release)<sizeof(basis.platform.release)
        &&sysctlbyname("kern.osversion",basis.platform.os_build,&length,NULL,0)==0&&length&&length<=sizeof(basis.platform.os_build)&&!basis.platform.os_build[length-1],
        "source-known current platform fixture fields");
    strcpy(basis.platform.machine,host.machine);strcpy(basis.platform.release,host.release);
    require(private_end_unit(private_cost.open_cpu),"actual source-known platform fixture strings paid");
    task_dyld_info_data_t info;memset(&info,0,sizeof(info));mach_msg_type_number_t count=TASK_DYLD_INFO_COUNT;
    require(task_info(mach_task_self(),TASK_DYLD_INFO,(task_info_t)&info,&count)==KERN_SUCCESS&&count==TASK_DYLD_INFO_COUNT
        &&info.all_image_info_format==TASK_DYLD_ALL_IMAGE_INFO_64&&info.all_image_info_addr,
        "source-known current task dyld identity");
    require(private_end_unit(private_cost.open_cpu),"source-known platform task identity paid");
    struct dyld_all_image_infos snapshot;memset(&snapshot,0,sizeof(snapshot));mach_vm_size_t copied=0;size_t needed=offsetof(struct dyld_all_image_infos,sharedCacheUUID)+16;
    require(info.all_image_info_size>=needed&&needed<=4096&&mach_vm_read_overwrite(mach_task_self(),info.all_image_info_addr,needed,(mach_vm_address_t)&snapshot,&copied)==KERN_SUCCESS
        &&copied==needed&&snapshot.version>=13&&!snapshot.processDetachedFromSharedRegion,"source-known own shared-cache identity bytes");
    for(unsigned i=0;i<16;i++)snprintf(basis.platform.cache_uuid+2*i,3,"%02x",snapshot.sharedCacheUUID[i]);
    require(private_end_unit(private_cost.open_cpu),"source-known own cache bytes paid");
}
static void currentness_batch_control(const char *root,int dir) {
    reset();fixture(dir);currentness_fixture_platform();struct stat directory;require(fstat(dir,&directory)==0,"actual currentness fixture directory stamp");
    struct private_dir *roots[3]={&outer.prefix,&outer.repo,&outer.output};
    for(unsigned i=0;i<3;i++){strcpy(roots[i]->path,root);roots[i]->fd=dir;roots[i]->dev=directory.st_dev;roots[i]->ino=directory.st_ino;roots[i]->uid=geteuid();}
    static const char *const names[2]={"currentness-a","currentness-b"};
    static const char *const tokens[2]={"\"currentness-a\"","\"currentness-b\""};
    for(unsigned i=0;i<2;i++) {
        struct private_guard *g=&private_guards.files[i];memset(g,0,sizeof(*g));g->root=1;g->len=1;g->verified=1;
        g->path=(struct private_slice){(const unsigned char *)tokens[i],strlen(tokens[i])};
        g->fd=openat(dir,names[i],O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(g->fd>=0&&write(g->fd,i?"b":"a",1)==1&&fstat(g->fd,&g->original)==0,"actual retained currentness fixture file");
        private_hash(i?"b":"a",1,g->sha);require(private_end_unit(private_cost.open_cpu),"actual retained currentness file setup paid");
    }
    private_guards.count=2;private_guards.stage=2;uint64_t before=private_cost.currentness_units_closed;
    int valid=private_revalidate_context(&context);
    uint64_t units=private_cost.currentness_units_closed-before,checks=private_check_batch.checks;
    printf("private_currentness_batch_source_known guards=2 expected_fixed_checks=8 completed_checks=%llu units_closed=%llu old_min_units=9 max_reserved_steps=%llu unit_cpu_max_ns=%llu active_after=%u debt_ns=%llu paid_ns=%llu actual_file_checks=true platform_checks=true real_candidate_created=0 runtime_approved=false\n",
        (unsigned long long)checks,(unsigned long long)units,(unsigned long long)private_cost.currentness_step_max,(unsigned long long)private_cost.currentness_unit_cpu_max_ns,private_check_batch.active,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"actual currentness batch evidence paid");
    if(!valid) {for(unsigned i=0;i<2;i++){int fd=private_guards.files[i].fd;private_guards.files[i].fd=-1;require(private_close(fd),"failed fixture currentness FD cleanup paid");}}
    require(valid&&checks==8&&units>0&&units<9&&private_cost.currentness_step_max<=64&&private_cost.currentness_unit_cpu_max_ns<=1000000
        &&!private_check_batch.active&&private_cost.debt==private_cost.debt_paid,
        "batched currentness executes all roots retained files and platform checks before returning");
    require(pwrite(private_guards.files[0].fd,"z",1,0)==1&&private_end_unit(private_cost.open_cpu),"actual same-length retained currentness file change paid");
    int refused=!private_revalidate_context(&context);
    int closed=1;for(unsigned i=0;i<2;i++){int fd=private_guards.files[i].fd;private_guards.files[i].fd=-1;closed=private_close(fd)&&closed;}
    printf("private_currentness_batch_change_source_known actual_in_place_change=true refused=%u original_descriptors_closed=%u source_changed_count=%llu completed_checks=%u active_after=%u debt_ns=%llu paid_ns=%llu native_process=false runtime_approved=false\n",
        (unsigned)refused,(unsigned)closed,(unsigned long long)private_errors[9].count,private_check_batch.checks,private_check_batch.active,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"actual currentness tamper and cleanup evidence paid");
    require(refused&&closed&&private_errors[9].count&&!private_check_batch.active&&private_cost.debt==private_cost.debt_paid,
        "batched currentness still refuses an actual retained-file change with every full cost paid");
    private_guards.count=0;private_guards.stage=1;controls++;
}

static unsigned nested_census_fflags,nested_census_old_seen;
static int nested_census_delivery(struct engine_owner *o,unsigned via,int observe) {
    require(via==2&&observe,"source-known observing fork census callback");nested_census_old_seen=o->child_event_pending;
    if(nested_census_fflags) {
        struct kevent event;EV_SET(&event,200,EVFILT_PROC,0,nested_census_fflags,0,&o->event_watch[1]);
        return engine_event_note(o,&event,1000);
    }
    return 1;
}
static void nested_census_delivery_control(void) {
    const unsigned flags[3]={NOTE_EXEC,NOTE_EXIT,0};
    for(unsigned which=0;which<3;which++) {
        reset();struct engine_owner o;struct private_engine_result r;memset(&o,0,sizeof(o));memset(&r,0,sizeof(r));o.context=&context;o.result=&r;
        o.ticket.pid=o.ticket.pgid=100;o.ticket.birth=10;o.ticket.start=20;o.direct_custody=1;o.slots=2;
        o.event_fd_live=1;o.event_registered_mask=3;o.child_event_pending=1;
        struct engine_member *m=&o.members[1];m->state.pid=200;m->state.ppid=m->state.pgid=100;m->state.birth=11;m->state.start_absolute=22;
        m->token_known=1;m->token.val[5]=200;m->token.val[7]=1;
        o.retired_count=1;o.retired[0]=(struct engine_incarnation){.pid=200,.birth=11,.start=22,.token=m->token,.current=1};
        o.event_watch[1]=(struct engine_event_watch){.pid=200,.birth=11,.start=22,.token=m->token,.bound=1};
        nested_census_fflags=flags[which];nested_census_old_seen=2;
        int observed=engine_observing_census(&o,1,nested_census_delivery);
        unsigned expected=flags[which]!=0;
        require(private_end_unit(private_cost.open_cpu),"nested census API actual controller cost fully paid");
        printf("private_nested_census_delivery_case_source_known case=%u old_pending_at_census=%u delivered_fflags=%u new_pending_after=%u exit_note_received=%u terminal_observed_ns=%llu actual_own_wait4=0 native_process=false values_supplied=true runtime_approved=false\n",
            which,nested_census_old_seen,flags[which],o.child_event_pending,m->exit_note_received,(unsigned long long)m->terminal_observed_ns);
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"nested census API numeric evidence paid before verdict");
        require(observed&&nested_census_old_seen==0&&o.child_event_pending==expected
            &&m->exit_note_received==(flags[which]==NOTE_EXIT)&&m->terminal_observed_ns==(flags[which]==NOTE_EXIT?1000u:0u)
            &&!m->exit_note_witness&&!r.created&&!r.reaped,
            "observing census consumes only old coalesced wake and preserves every nested new delivery");
    }
    controls++;printf("private_nested_census_delivery_source_known cases=3 old_coalesced_consumed=true nested_EXEC_EXIT_retained=true native_process=false runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"nested census completion evidence fully paid");
}

static void process_create_policy_api(void) {
    reset();
    require(native_budget_allowed(NATIVE_PROCESS_CREATE,500000,34000000,600000,100000)
        &&native_budget_allowed(NATIVE_PROCESS_CREATE,2000000,250000000,3000000,1000000),"process-create policy API admits34ms and exact250ms with unchanged CPU inclusive application gates");
    require(!native_budget_allowed(NATIVE_PROCESS_CREATE,500000,251000000,600000,100000)
        &&!native_budget_allowed(NATIVE_PROCESS_CREATE,2000001,34000000,2500000,499999),"process-create policy API refuses251ms and CPU above2ms");
    require(!native_budget_allowed(NATIVE_OTHER,500000,34000000,600000,100000)
        &&!native_budget_allowed(NATIVE_PROCESS_CREATE,1500000,34000000,3000001,1000000)
        &&!native_budget_allowed(NATIVE_OTHER,1500000,25000000,3000001,1000000),"other-native34ms and every native inclusive above3ms remain refused");
    require(!native_budget_allowed(NATIVE_PROCESS_CREATE,UINT64_MAX,34000000,600000,100000)
        &&!native_budget_allowed(NATIVE_PROCESS_CREATE,500000,UINT64_MAX,600000,100000),"unknown native clocks remain refused without finite excess invention");
    struct engine_owner a,b,c;memset(&a,0,sizeof(a));memset(&b,0,sizeof(b));memset(&c,0,sizeof(c));
    engine_creation_wall_observation(&a,34000000,1);engine_creation_wall_observation(&a,34000000,1);
    engine_creation_wall_observation(&b,251000000,1);
    require(a.process_create_wall_excess_ns==9000000&&a.process_create_wall_excess_count==1
        &&b.process_create_wall_excess_ns==226000000&&b.process_create_wall_excess_count==1
        &&private_cost.process_create_wall_excess_ns==235000000&&private_cost.process_create_wall_excess_count==2,
        "process-create excess API sums each known observation once including refused251ms");
    engine_creation_wall_observation(&c,UINT64_MAX,0);
    require(!private_cost.process_create_wall_excess_known&&!c.process_create_wall_excess_known
        &&private_cost.process_create_wall_excess_ns==235000000&&private_cost.process_create_wall_excess_count==2,
        "unknown process-create wall marks aggregate unproven without erasing known excess");
    require(private_end_unit(private_cost.open_cpu),"native policy supplied-input API actual CPU fully paid");
    printf("private_process_create_policy_source_known supplied_inputs=true native_process=false other_native_wall_limit_ns=25000000 process_create_wall_limit_ns=250000000 runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"native policy API evidence paid");controls++;reset();
}
static int process_create_wall_control(const char *self,const char *root,int dir,unsigned mode) {
    require(mode<3,"three process-create native policy projection cases");
    struct engine_owner owner;struct private_engine_result result;int status=-1;
    int born=total_spawn(&owner,&result,self,dir,13+mode,root);
    if(born)await_original(&owner);
    (void)engine_retire(&owner,&status);
    int settled=owner.direct_reaped&&owner.wait_observed&&result.group_gone&&result.retired_coverage&&!engine_current;
    if(settled)fixture_live_owner=NULL;
    require(settled&&owner.native.cpu_known&&owner.native.wall_known&&owner.native.cpu<=2000000
        &&owner.native.wall<=250000000&&result.inclusive_max_ns.known
        &&result.inclusive_max_ns.value<=3000000&&private_cost.ordinary_max<=1000000
        &&owner.helper_cost_retained&&owner.helper_cost_paid&&private_cost.debt==private_cost.debt_paid,
        "process-create projection retains actual native hard gates ownwait4 groupgone and full real CPU payment");
    printf("private_process_create_axis_source_known mode=%u original_pid=%d actual_own_wait4=%u group_gone=%u actual_native_CPU_ns=%llu actual_native_wall_ns=%llu applied_CPU_ns=%llu applied_wall_ns=%llu applied_inclusive_ns=%llu process_create_wall_excess_ns=%llu process_create_wall_excess_count=%llu excess_scope=original_prepared_spawn_snapshot projection=true born=%d continued=%u debt_ns=%llu paid_ns=%llu real_candidate_created=0 runtime_approved=false\n",
        mode,owner.ticket.pid,owner.direct_reaped,result.group_gone,(unsigned long long)owner.native.cpu,(unsigned long long)owner.native.wall,
        (unsigned long long)owner.native_gate_cpu_ns,(unsigned long long)owner.native_gate_wall_ns,(unsigned long long)owner.native_gate_inclusive_ns,
        (unsigned long long)owner.process_create_wall_excess_ns,(unsigned long long)owner.process_create_wall_excess_count,born,owner.activation_ns!=0,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"process-create projected-policy raw evidence fully paid");
    if(mode==0)require(born&&!owner.failed&&owner.activation_ns&&owner.native_gate_wall_ns==34000000
        &&owner.process_create_wall_excess_ns==9000000&&owner.process_create_wall_excess_count==1
        &&private_cost.process_create_wall_excess_ns==9000000&&private_cost.process_create_wall_excess_count==1,
        "34ms process-create policy admits and records exact9ms excess on actual owned fixture");
    else if(mode==1)require(!born&&!owner.activation_ns&&private_errors[5].count&&owner.native_gate_wall_ns==251000000
        &&owner.process_create_wall_excess_ns==226000000&&owner.process_create_wall_excess_count==1,
        "251ms process-create policy refuses without CONT and retains exact226ms excess through own settlement");
    else require(!born&&!owner.activation_ns&&private_errors[5].count&&owner.native_gate_cpu_ns==2000001
        &&owner.process_create_wall_excess_ns==9000000&&owner.process_create_wall_excess_count==1,
        "process-create CPU above2ms refuses without CONT after actual owned settlement");
    engine_close_transferred(&owner);require(close(owner.ledger)==0,"projected-policy original raw ledger close");free(owner.capture);
    require(private_end_unit(private_cost.open_cpu),"projected-policy original disposal fully paid");controls++;
    printf("private_process_create_completed_source_known mode=%u actual_own_wait4=1 group_gone=1 policy_inputs_projected=true expected_exit=%u runtime_approved=false\n",mode,mode?81:0);
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"projected-policy completion evidence fully paid");
    return mode?81:0;
}

// Supplied scalar clocks/counters exercise the actual production cadence
// predicates. They are not native STOP/RI observations or timing claims.
static void stopped_gap_api_owner(struct engine_owner *o,struct private_engine_result *r) {
    reset();memset(o,0,sizeof(*o));memset(r,0,sizeof(*r));o->context=&context;o->result=r;
    o->activation_ns=100000000;o->paused=1;o->members[0].state.pid=1;o->members[0].phase=ENGINE_EXPECTED;
    o->members[0].ri_started_ns=100000000;o->pause_ack_mask=o->pause_counter_known_mask=1;
    o->pause_ack_ns=o->pause_member_ack_ns[0]=110000000;
    o->pause_ack_ri_begin_ns[0]=111000000;o->pause_ack_ri_end_ns[0]=112000000;
    o->pause_user[0]=o->pause_observed_user[0]=o->members[0].user=100;
    o->pause_system[0]=o->pause_observed_system[0]=o->members[0].system=200;
    o->pause_resume_ri_begin_ns[0]=208000000;o->pause_resume_ri_end_ns[0]=209000000;
    o->pause_validated_ri_end_ns[0]=209000000;o->pause_resume_ns=210000000;
    o->pause_resume_attempted=o->pause_resume_admitted=1;
}
static void stopped_gap_api(void) {
    struct engine_owner o;struct private_engine_result r;
    stopped_gap_api_owner(&o,&r);
    require(engine_sample_gap(&o,230000000)&&o.max_sample_gap_ns==130000000&&o.running_gap_max_ns==20000000
        &&o.stopped_interval_ns==100000000&&o.members[0].ri_started_ns==100000000,
        "stable ACKed stop over50ms retains raw gap and admits both bounded running sides");
    stopped_gap_api_owner(&o,&r);o.members[0].ri_started_ns=60000000;
    require(engine_sample_gap(&o,260000000)&&o.max_sample_gap_ns==200000000&&o.running_gap_max_ns==50000000,
        "both running sides at exactly50ms pass without summing across the stopped interval");
    stopped_gap_api_owner(&o,&r);o.pause_ack_mask=0;
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==130000000&&!o.group_kill_attempted,
        "unACKed STOP earns no cadence exclusion even with supplied stable counters");
    stopped_gap_api_owner(&o,&r);o.pause_ack_mask=o.pause_counter_known_mask=0;o.paused=0;o.running=1;
    require(!engine_sample_gap(&o,150000001)&&o.running_gap_max_ns==50000001&&o.members[0].ri_started_ns==100000000,
        "running RI gap over50ms refuses without resetting previous begin");
    stopped_gap_api_owner(&o,&r);o.pause_resume_attempted=o.pause_resume_admitted=0;
    require(!engine_pause_live_system(&o,0,201)&&o.pause_system_violation&&!(o.pause_counter_known_mask&1),
        "SYSTEM growth invalidates the immutable ACK counter seal");
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==130000000,
        "changed SYSTEM cannot retain a stopped-gap exclusion");
    stopped_gap_api_owner(&o,&r);o.pause_resume_attempted=o.pause_resume_admitted=0;
    require(!engine_pause_live_user(&o,0,101)&&o.pause_user_violation&&!engine_sample_gap(&o,230000000),
        "USER growth invalidates stopped-gap exclusion independently of SYSTEM");
    stopped_gap_api_owner(&o,&r);o.members[0].ri_started_ns=200000000;
    require(engine_ri_begin(&o,0,250000000),"post-resume query begins with forty running milliseconds");
    require(!engine_ri_end(&o,0,260000001)&&o.running_gap_max_ns==50000001&&o.members[0].ri_started_ns==200000000
        &&o.ri_previous_ns[0]==200000000&&o.ri_begin_ns[0]==250000000&&o.ri_end_ns[0]==260000001&&o.ri_gap_failed_mask==1,
        "post-resume RI query-return overshoot refuses and preserves every raw endpoint");
    stopped_gap_api_owner(&o,&r);o.members[0].ri_started_ns=59999999;
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==50000001,
        "pre-ACK running gap above50ms cannot be hidden by a later long stop");
    stopped_gap_api_owner(&o,&r);o.pause_ack_ri_begin_ns[0]=109000000;
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==130000000,
        "ACK without a fresh subsequent RI cannot supply stopped credit");
    stopped_gap_api_owner(&o,&r);o.pause_resume_ri_end_ns[0]=0;
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==130000000,
        "resume without a fresh stopped RI endpoint cannot supply stopped credit");
    stopped_gap_api_owner(&o,&r);o.pause_counter_known_mask=0;
    require(!engine_sample_gap(&o,230000000)&&o.running_gap_max_ns==130000000,
        "unknown stopped counters cannot become zero running age");
    stopped_gap_api_owner(&o,&r);
    require(engine_running_interval(&o,0,230000000,220000000)==UINT64_MAX,
        "reversed raw endpoints remain unknown despite a supplied positive stop");
    stopped_gap_api_owner(&o,&r);
    require(!engine_sample_gap(&o,UINT64_MAX)&&o.running_gap_max_ns==UINT64_MAX,
        "unavailable current clock cannot become stopped credit");
    stopped_gap_api_owner(&o,&r);o.pause_resume_attempted=o.pause_resume_admitted=0;o.pause_validated_ri_end_ns[0]=112000000;
    require(engine_sample_gap(&o,230000000)&&o.stopped_interval_ns==2000000,
        "provisional stopped accounting observation does not publish an unvalidated future interval");
    o.pause_validated_ri_end_ns[0]=230000000;engine_stopped_interval_record(&o);engine_stopped_interval_record(&o);
    require(o.stopped_interval_ns==120000000,"repeated validated stopped observations never double-count interval credit");
    reset();controls++;
    printf("private_stopped_gap_API_source_known cases=14 stopped_raw_gap_ns=130000000 stopped_running_gap_ns=20000000 exact_each_side_ns=50000000 no_ACK_refused=true running_over50_refused=true USER_SYSTEM_growth_refused=true query_return_overshoot_refused=true raw_endpoints_retained=true native_process=false values_supplied=true OS_timing_claimed=false runtime_approved=false\n");
    require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"bounded stopped-gap API evidence fully paid");
}
static void total_boundary_api(void) {
    reset();struct rusage r;memset(&r,0,sizeof(r));r.ru_utime.tv_usec=249998;
    require(engine_helper_cpu_cost(500000)==1000000&&engine_helper_cpu_cost(5000000)==5000000&&engine_helper_cpu_cost(UINT64_MAX)==3000000,"helper CPU debt API preserves one millisecond floor and conservative unknown cost");
    require(private_rest_slice_ns(0)==0&&private_rest_slice_ns(999999)==999999&&private_rest_slice_ns(9000000)==1000000,"absolute-debt rest loop requests at most one millisecond per sleep");
    require(engine_wait4_cpu(&r)==ENGINE_COMMAND_CPU_NS,"authoritative wait4 rounds each microsecond field once at exact cap");
    r.ru_utime.tv_usec=249999;require(engine_wait4_cpu(&r)>ENGINE_COMMAND_CPU_NS,"rounded terminal CPU beyond cap is retained without clipping");
    r.ru_utime.tv_usec=1000000;require(engine_wait4_cpu(&r)==UINT64_MAX,"invalid wait4 units remain unknown rather than zero");
    require(engine_pause_user_allowed(5000,6000,1000)&&!engine_pause_user_allowed(5000,6001,1000)&&!engine_pause_user_allowed(UINT64_MAX,6000,1000),"paused terminal API uses exact1us resolution and preserves unknown baseline");
    require(engine_rss_value(-1)==UINT64_MAX&&engine_rss_value(16777216)==16777216,"negative terminal RSS API remains unknown rather than zero or a finite breach");
    uint64_t sum=0;require(engine_safe_sum(ENGINE_JOB_CPU_NS-1,1,&sum)&&sum==ENGINE_JOB_CPU_NS&&engine_safe_sum(sum,1,&sum)&&sum>ENGINE_JOB_CPU_NS,"checked cumulative helper CPU retains job cap crossing");
    for(unsigned paused=0;paused<2;paused++) {
        reset();struct engine_owner o;struct private_engine_result result;memset(&o,0,sizeof(o));memset(&result,0,sizeof(result));o.context=&context;o.result=&result;
        o.activation_ns=100000000;o.running=!paused;o.paused=paused;o.members[0].state.pid=1;o.members[0].phase=ENGINE_EXPECTED;o.members[0].ri_started_ns=100000000;
        require(engine_sample_gap(&o,150000000)&&o.max_sample_gap_ns==50000000,"exact50ms actual read-start interval remains admissible");
        require(!engine_sample_gap(&o,150000001)&&o.max_sample_gap_ns==50000001&&o.members[0].ri_started_ns==100000000,"known running and unACKed paused RI gaps over50ms refuse without resetting prior evidence");
    }
    reset();controls++;printf("private_total_budget_boundary_source_known api=true supplied_clock_values=true native_process=false runtime_approved=false\n");
}
static void total_retained_guard_api(void) {
    reset();struct engine_owner o;struct private_engine_result result;memset(&o,0,sizeof(o));memset(&result,0,sizeof(result));o.context=&context;o.result=&result;
    char argument[4097],release[4097];
    int mode2=fixture_total_argument(2,"/fixture",argument,release)&&!strcmp(argument,"/fixture/rollup-2")&&!strcmp(release,"/fixture/rollup-2-release");
    int mode6=fixture_total_argument(6,"/fixture",argument,release)&&!strcmp(argument,"/fixture/release-6")&&!strcmp(release,"/fixture/release-6");
    require(mode2&&mode6,"fixture argv API retains distinct mode2 and mode6 owned release paths");
    require(native_pid_list_empty(0,0)&&!native_pid_list_empty(0,EIO)&&!native_pid_list_empty(-1,EIO)&&!native_pid_list_empty(1,0)
        &&!native_pid_list_known(0,EIO),"native census API zero with EIO is unknown rather than an empty child or group");
    o.activation_ns=100000000;o.members[0].state.pid=1;o.members[0].phase=ENGINE_EXPECTED;o.members[0].ri_started_ns=100000000;
    require(engine_ri_begin(&o,0,140000000),"RI-return API forty millisecond beginning remains admissible");
    require(!engine_ri_end(&o,0,160000000)&&o.members[0].ri_started_ns==100000000&&o.max_sample_gap_ns==60000000
        &&o.ri_previous_ns[0]==100000000&&o.ri_begin_ns[0]==140000000&&o.ri_end_ns[0]==160000000
        &&o.ri_previous_mask==1&&o.ri_begin_mask==1&&o.ri_end_mask==1&&o.ri_gap_failed_mask==1&&!o.group_kill_attempted,
        "RI-return API retains old begin through forty plus twenty milliseconds");
    require(engine_final_kill_required(1,0)&&engine_final_kill_required(0,1)&&!engine_final_kill_required(1,1),"terminal leader API cannot skip required retained-member group kill");
    require(engine_token_cleanup_required(1,1,1)&&!engine_token_cleanup_required(0,1,1)&&!engine_token_cleanup_required(1,0,1)&&!engine_token_cleanup_required(1,1,0),"escaped cleanup API requires current exact-birth outside-group recipient");
    o.child_created=1;o.child_rollup_known=1;o.child_cost_unproven=0;o.terminal_cpu_ns=12345;o.helper_cost_retained=o.helper_cost_paid=1;
    require(engine_child_cost_covered(&o),"qualified child-rollup API leaves original leader bill intact");o.child_cost_unproven=1;
    require(!engine_child_cost_covered(&o)&&o.terminal_cpu_ns==12345&&o.helper_cost_retained&&o.helper_cost_paid,"unproven child API refuses without relabelling or erasing known leader CPU debt");
    require(engine_child_rollup_covers(31,5000,4000,4000,3000)&&!engine_child_rollup_covers(27,5000,4000,4000,3000)
        &&!engine_child_rollup_covers(31,3999,4000,4000,3000)&&!engine_child_rollup_covers(31,0,0,0,0),"child disappearance API requires qualified nonduplicated terminal-counter rollup");
    reset();controls++;printf("private_total_retained_guard_source_known api=true supplied_clock_and_identity_values=true native_process=false runtime_approved=false\n");
}
int main(int argc,char **argv) {
    if(argc==6&&!strcmp(argv[1],"--keyid-format=long")&&!strcmp(argv[2],"--status-fd=1")&&!strcmp(argv[3],"--verify")&&!strcmp(argv[5],"-"))same_wake_verifier(argv[4]);
    if(argc==3&&!strcmp(argv[1],"same-wake-leader"))same_wake_leader(argv[2]);
    if(argc==3&&!strcmp(argv[1],"knote-rollup-child"))knote_rollup_child(argv[2]);
    if(argc==4&&!strcmp(argv[1],"event-gated-leader"))verifier_event_leader(argv[2],!strcmp(argv[3],"fork"));
    if(argc==5&&!strcmp(argv[1],"pause-bypass-child")){pause_bypass_child(argv[2],argv[3],argv[4]);}
    if(argc==3&&!strcmp(argv[1],"rss-child")){size_t n=20u*1024u*1024u;volatile unsigned char *m=malloc(n);if(!m)_exit(77);for(size_t i=0;i<n;i+=4096)m[i]=1;_exit(0);}
    if(argc==2&&!strcmp(argv[1],"hold-gated-child")){char byte;ssize_t n=read(STDIN_FILENO,&byte,1);_exit(n==1?0:79);}
    if(argc==3&&!strcmp(argv[1],"long-wall-cpu-child")){uint64_t begin=clock_ns(CLOCK_MONOTONIC);if(begin==UINT64_MAX||begin>UINT64_MAX-100000000)_exit(77);uint64_t until=begin+100000000;while(clock_ns(CLOCK_MONOTONIC)<until)usleep(1000);burn_cpu(strtoull(argv[2],NULL,10));}
    if(argc>=2&&!strcmp(argv[1],"burn-child")){require(argc==3,"burn child arguments");burn_cpu(strtoull(argv[2],NULL,10));}
    if(argc>=2&&!strcmp(argv[1],"rollup-child")){require(argc==3,"rollup child arguments");rollup_child(argv[0],argv[2]);}
    if(argc>=2&&!strcmp(argv[1],"exit-child"))_exit(0);
    if(argc>=2&&!strcmp(argv[1],"held-child")){require(argc==3,"held child arguments");while(access(argv[2],F_OK)!=0)usleep(1000);_exit(0);}
    unsigned boundary=argc==2&&!strcmp(argv[1],"total-boundary-api");
    unsigned stopped_api=argc==2&&!strcmp(argv[1],"stopped-gap-api");
    unsigned stopped_process=argc==2&&!strcmp(argv[1],"stopped-gap-control");
    unsigned retained=argc==2&&!strcmp(argv[1],"total-retained-guard-api");
    unsigned selected=argc==3&&!strcmp(argv[1],"total-control")?(unsigned)strtoul(argv[2],NULL,10):12;
    unsigned hold=argc==2&&!strcmp(argv[1],"hold-lifetime-control");
    unsigned charge=argc==2&&!strcmp(argv[1],"charge-diagnostic-control");
    unsigned flow=argc==2&&!strcmp(argv[1],"flow-controls");
    unsigned result_only=argc==2&&!strcmp(argv[1],"result-controls");
    unsigned final_terms=argc==2&&!strcmp(argv[1],"final-conjunction-controls");
    unsigned member_census=argc==2&&!strcmp(argv[1],"member-peak-census-control");
    unsigned diagnostics=argc==2&&!strcmp(argv[1],"terminal-diagnostic-controls");
    unsigned setup=argc==2&&!strcmp(argv[1],"diagnostic-setup-budget-control");
    unsigned deadline=argc==2&&!strcmp(argv[1],"terminal-deadline-controls");
    unsigned cpu_debt=argc==2&&!strcmp(argv[1],"helper-cpu-debt-control");
    unsigned expand=argc==2&&!strcmp(argv[1],"argv-expansion-controls");
    unsigned native_api=argc==2&&!strcmp(argv[1],"process-create-policy-api");
    unsigned native_case=argc==3&&!strcmp(argv[1],"process-create-wall-control")?(unsigned)strtoul(argv[2],NULL,10):3;
    unsigned race_case=argc==3&&!strcmp(argv[1],"live-sample-race-control")?(unsigned)strtoul(argv[2],NULL,10):7;
    unsigned event_api=argc==2&&!strcmp(argv[1],"verifier-event-api");
    unsigned child_exit_api=argc==3&&!strcmp(argv[1],"child-exit-witness-api")?(unsigned)strtoul(argv[2],NULL,10):9;
    unsigned child_exit_process=argc==2&&!strcmp(argv[1],"child-exit-witness-control");
    unsigned leader_refresh_api=argc==3&&!strcmp(argv[1],"leader-rollup-refresh-api")?(unsigned)strtoul(argv[2],NULL,10):5;
    unsigned composition=argc==2&&!strcmp(argv[1],"sample-unit-composition-control");
    unsigned batch=argc==2&&!strcmp(argv[1],"currentness-batch-control");
    unsigned nested_census=argc==2&&!strcmp(argv[1],"nested-census-delivery-control");
    unsigned same_wake=argc==2&&!strcmp(argv[1],"same-wake-admission-control");
    unsigned same_wake_api=argc==2&&!strcmp(argv[1],"same-wake-admission-api");
    unsigned event_wait_api=argc==2&&!strcmp(argv[1],"same-wake-event-wait-api");
    unsigned custody_identity=argc==2&&!strcmp(argv[1],"custody-stop-identity-api");
    unsigned exit_before_api=argc==2&&!strcmp(argv[1],"same-wake-exit-api");
    unsigned event_case=argc==3&&!strcmp(argv[1],"verifier-event-control")?(unsigned)strtoul(argv[2],NULL,10):3;
    require(argc==1||selected<12||diagnostics||setup||deadline||cpu_debt||expand||native_api||native_case<3||race_case<7||event_api||event_case<3||child_exit_api<9||child_exit_process||leader_refresh_api<5||composition||batch||nested_census||same_wake||same_wake_api||event_wait_api||custody_identity||exit_before_api||hold||flow||result_only||final_terms||member_census||charge||boundary||stopped_api||stopped_process||retained,"private source-known controls argc");
    require(atexit(dispose_owned)==0,"owned cleanup registration");
    char root[]="/private/tmp/dream-private-engine-controls.XXXXXX";require(mkdtemp(root)!=NULL,"owned fixture directory");
    int dir=open(root,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0,"owned fixture root");
    fprintf(stderr,"private_engine_fixture_source_known pid=%d retained=%s real_candidate_created=0 runtime_approved=false\n",getpid(),root);
    require(fflush(stderr)==0,"early source-known fixture marker");
    fixture(dir);reset();
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

    char self[PROC_PIDPATHINFO_MAXSIZE];require(proc_pidpath(getpid(),self,sizeof(self))>0,"actual source-known image");
    require(engine_owner_priority(),"source-known controller nice10 utility priority");
    if(charge){charge_diagnostic_control(dir);require(close(dir)==0,"owned charge root close");return 0;}
    if(hold){quarantine_hold_process_control(self,root,dir);require(close(dir)==0,"owned holder root close");return 0;}
    if(flow){flow_controls(root,dir);require(close(dir)==0,"owned flow root close");return 0;}
    if(final_terms){final_conjunction_controls(dir);require(close(dir)==0,"owned final conjunction root close");return 0;}
    if(member_census){member_peak_census_control(self,root,dir);require(close(dir)==0,"owned member census root close");return 0;}
    if(result_only){result_controls();require(close(dir)==0,"owned result root close");return 0;}
    if(boundary){total_boundary_api();require(close(dir)==0,"owned boundary API root close");return 0;}
    if(stopped_api){stopped_gap_api();require(close(dir)==0,"owned stopped-gap API root close");return 0;}
    if(stopped_process){total_control(self,root,dir,5);require(close(dir)==0,"owned stopped-gap process root close");return 0;}
    if(retained){total_retained_guard_api();require(close(dir)==0,"owned retained guard API root close");return 0;}
    if(selected<12){total_control(self,root,dir,selected);require(close(dir)==0,"owned narrow fixture root close");return 0;}
    if(diagnostics){terminal_diagnostic_controls(dir);require(close(dir)==0,"owned diagnostic root close");return 0;}
    if(custody_identity){custody_stop_identity_api();require(close(dir)==0,"owned custody STOP identity API root close");return 0;}
    if(event_wait_api){same_wake_event_wait_api();require(close(dir)==0,"owned event-wait API root close");return 0;}
    if(exit_before_api){int code=same_wake_exit_api();require(close(dir)==0,"owned pre-admission EXIT API root close");return code;}
    if(same_wake_api){same_wake_admission_api();require(close(dir)==0,"owned same-wake API root close");return 0;}
    if(same_wake){same_wake_admission_control(self,root,dir);require(close(dir)==0,"owned same-wake root close");return 0;}
    if(nested_census){nested_census_delivery_control();require(close(dir)==0,"owned nested census API root close");return 0;}
    if(composition){sample_unit_composition_control();require(close(dir)==0,"owned unit composition root close");return 0;}
    if(batch){currentness_batch_control(root,dir);require(close(dir)==0,"owned currentness batch root close");return 0;}
    if(leader_refresh_api<5){int code=leader_rollup_refresh_api(leader_refresh_api);require(close(dir)==0,"owned leader refresh API root close");return code;}
    if(child_exit_api<9){int code=child_exit_witness_api(child_exit_api);require(close(dir)==0,"owned child witness API root close");return code;}
    if(child_exit_process){child_exit_witness_control(self,root,dir);require(close(dir)==0,"owned child witness process root close");return 0;}
    if(event_api){verifier_event_api();require(close(dir)==0,"owned event API root close");return 0;}
    if(event_case<3){int code=verifier_event_control(self,root,dir,event_case);require(close(dir)==0,"owned event control root close");return code;}
    if(race_case<7){int code=live_sample_race_control(self,root,dir,race_case);require(close(dir)==0,"owned live race root close");return code;}
    if(native_api){process_create_policy_api();require(close(dir)==0,"owned native policy API root close");return 0;}
    if(native_case<3){int code=process_create_wall_control(self,root,dir,native_case);require(close(dir)==0,"owned native policy process root close");return code;}
    if(expand){argv_expansion_controls(dir);require(close(dir)==0,"owned argv expansion root close");return 0;}
    if(cpu_debt){helper_cpu_debt_control(self,root,dir);require(close(dir)==0,"owned CPU-debt root close");return 0;}
    if(deadline){terminal_deadline_controls(dir);require(close(dir)==0,"owned terminal-deadline root close");return 0;}
    if(setup){diagnostic_setup_budget_control();require(close(dir)==0,"owned setup root close");return 0;}
    argv_expansion_controls(dir);semantic_controls();result_controls();final_conjunction_controls(dir);flow_controls(root,dir);terminal_diagnostic_controls(dir);terminal_deadline_controls(dir);charge_diagnostic_control(dir);
    for(unsigned mode=0;mode<12;mode++)total_control(self,root,dir,mode);
    helper_cpu_debt_control(self,root,dir);
    process_create_policy_api();
    for(unsigned mode=0;mode<3;mode++)(void)process_create_wall_control(self,root,dir,mode);
    for(unsigned mode=0;mode<7;mode++)(void)live_sample_race_control(self,root,dir,mode);
    custody_stop_identity_api();
    same_wake_event_wait_api();(void)same_wake_exit_api();
    same_wake_admission_api();
    same_wake_admission_control(self,root,dir);
    member_peak_census_control(self,root,dir);
    nested_census_delivery_control();
    sample_unit_composition_control();currentness_batch_control(root,dir);
    for(unsigned mode=0;mode<5;mode++)(void)leader_rollup_refresh_api(mode);
    for(unsigned mode=0;mode<9;mode++)(void)child_exit_witness_api(mode);
    child_exit_witness_control(self,root,dir);
    verifier_event_api();
    for(unsigned mode=0;mode<3;mode++)(void)verifier_event_control(self,root,dir,mode);
    stopped_gap_api();total_boundary_api();
    total_retained_guard_api();
    quarantine_hold_process_control(self,root,dir);
    require(close(dir)==0,"owned fixture root close");
    printf("private_engine_controls=pass cases=%u total_budget_controls=8 synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);return 0;
}
