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
static void fixture(int);
static void fixture_reset_cost(void) {
    require(!engine_current,"previous original owner must be settled before fixture reset");
    memset(&private_cost,0,sizeof(private_cost));private_reset_diagnostics();private_failed=0;private_error="input_invalid";private_detail="fixture unset";
    private_cost.entry=clock_ns(CLOCK_MONOTONIC);private_cost.deadline=private_cost.entry+30000000000ULL;
    private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);engine_cancel_signal=engine_owner_loss_signal=0;
    source_known_payment_interrupts=source_known_payment_fail_nth=source_known_schedule_interrupts=source_known_schedule_fail_nth=0;
    source_known_tail_over_cpu=source_known_final_kill_eperm=source_known_terminal_rss_unavailable=0;
    source_known_taskall_pid=0;source_known_taskall_error=source_known_taskall_clock_failure=0;
    source_known_taskall_native_result=source_known_taskall_native_errno=0;source_known_before_live_read=NULL;source_known_live_rechecks=0;source_known_live_read_error=0;
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
    o->context=&context;o->result=r;o->slots=(mode==2||mode==23||mode==24)?2:1;o->source_admitted=1;o->controller_parent=getppid();
    o->input=o->output=o->errors=-1;o->ledger=-1;o->capture=malloc(ENGINE_CAPTURE_MAX);require(o->capture!=NULL,"bounded total-budget fixture capture");
    fixture_live_owner=o;fixture_release_path[0]=0;
    char ledger[48];snprintf(ledger,sizeof(ledger),"total-%u.ndjson",mode);o->ledger=openat(dir,ledger,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(o->ledger>=0,"owned total-budget fixture ledger");
    struct engine_prepared p;memset(&p,0,sizeof(p));for(unsigned i=0;i<2;i++)p.input[i]=p.output[i]=p.errors[i]=-1;
    strcpy(p.image,self);strcpy(p.descendant,self);strcpy(p.cwd,root);strcpy(o->image,self);strcpy(o->descendant,self);
    const char *kind=(mode==23||mode==24)?"event-gated-leader":mode==2?"rollup-child":(mode==6||(mode>=16&&mode<=22)||mode==25)?"held-child":mode==7?"exit-child":mode==8?"rss-child":mode==12?"long-wall-cpu-child":(mode==9||mode==10)?"pause-bypass-child":"burn-child";
    char argument[4097];require(fixture_total_argument(mode,root,argument,fixture_release_path),"bounded fixture argument and exact original release path");
    require(engine_put(&p,self,&p.argv[p.argc++])&&engine_put(&p,kind,&p.argv[p.argc++])&&engine_put(&p,argument,&p.argv[p.argc++]),"bounded source-known helper argv");
    if(mode==23||mode==24)require(engine_put(&p,mode==23?"fork":"none",&p.argv[p.argc++]),"bounded source-known event leader choice");
    if(mode==9||mode==10){char path[4097];snprintf(path,sizeof(path),"%s/pause-%u-done",root,mode);require(engine_put(&p,path,&p.argv[p.argc++]),"bounded actual pause done marker");snprintf(path,sizeof(path),"%s/pause-%u-release",root,mode);require(engine_put(&p,path,&p.argv[p.argc++]),"bounded actual pause release marker");strcpy(fixture_release_path,path);}
    p.argv[p.argc]=NULL;
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
    struct fixture_live_pacing pacing;memset(&pacing,0,sizeof(pacing));
    if(mode==11)source_known_terminal_rss_unavailable=1;
    if(mode==9||mode==10) {
        owner.foreground=1;require(engine_pause(&owner)&&owner.paused&&owner.pause_baseline_known,"below-cap external-CONT fixture obtains immutable positive ACK before stimulus");
        char start[4097],done[4097],release[4097];snprintf(start,sizeof(start),"%s/pause-%u-start",root,mode);snprintf(done,sizeof(done),"%s/pause-%u-done",root,mode);snprintf(release,sizeof(release),"%s/pause-%u-release",root,mode);
        fixture_touch(start);
        require(engine_sample_member(&owner,0,owner.ticket.pid)&&owner.members[0].state.status==SSTOP&&owner.members[0].state.runnable==0,
            "fresh actual stopped RI before external pause stimulus preserves immutable ACK");
        require(engine_checkpoint(&owner),"external pause fixture stimulus preparation is paid");
        require(kill(-owner.ticket.pgid,SIGCONT)==0,"fixture external CONT deliberately bypasses product resume");
        require(!owner.pause_resume_admitted&&!owner.pause_resume_attempted,"external CONT never fabricates admitted product resume");
        while(access(done,F_OK)!=0){require(clock_ns(CLOCK_MONOTONIC)<outer.deadline,"original external-pause fixture deadline");require(engine_checkpoint(&owner),"bounded external-pause burn observation paid");usleep(1000);}
        if(mode==10) {
            // The adversarial fixture restores kernel STOP after its burn, so
            // a status-only guard cannot substitute for immutable RI equality.
            require(kill(-owner.ticket.pgid,SIGSTOP)==0,"fixture external reSTOP preserves original PID pin");
            uint64_t until=clock_ns(CLOCK_MONOTONIC)+ENGINE_PAUSE_NS;
            for(;;){struct image state;require(control_observe(owner.ticket.pid,&state)&&state.birth==owner.ticket.birth,"original external reSTOP identity");if(state.status==SSTOP&&state.runnable==0)break;require(clock_ns(CLOCK_MONOTONIC)<until,"external reSTOP finite ACK");require(engine_checkpoint(&owner),"external reSTOP observation paid");usleep(1000);}
            paused_read_return=engine_sample_member(&owner,0,owner.ticket.pid);
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
            uint64_t until=clock_ns(CLOCK_MONOTONIC)+20000000;
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
    if(mode==10)require(owner.pause_live_checked&&owner.pause_observed_user[0]>owner.pause_user[0]&&owner.max_sample_gap_ns<=ENGINE_SAMPLE_NS,"live paused-user causal fixture reached actual RI growth without timing refusal");
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
    engine_close_transferred(&owner);require(close(owner.ledger)==0,"owned total fixture ledger close");free(owner.capture);fixture_live_owner=NULL;controls++;
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
    require(engine_event_note(&o,&event),"coalesced original leader proc notification is accepted without child PID inference");
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
    require(engine_event_note(&o,&event)&&o.child_discovered_via==1&&!o.child_created,"later fork notification never relabels first polling discovery or invents a birth");
    require(!engine_event_note(&o,&(struct kevent){.ident=999999,.filter=EVFILT_PROC,.fflags=NOTE_EXIT}),"foreign process notification is not retained identity or settlement evidence");
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
        int status=0;pid_t got;do{got=waitpid(child,&status,0);}while(got<0&&errno==EINTR);
        if(got!=child||!WIFEXITED(status)||WEXITSTATUS(status))_exit(77);
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
    int missed_fork_observed=0;
    if(mode!=2) {
        int flags=held_before?fcntl(queue,F_GETFD):-1;
        require(born&&registered==1&&held_before&&flags>=0&&(flags&FD_CLOEXEC),"actual leader registration and owned close-on-exec queue precede first controlled continuation");
        fixture_touch(fixture_release_path);await_original(&o);
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
    if(!mode)require(missed_fork_observed,"actual queued leader fork survives child reap and terminal precheck before census");
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
        require(!engine_sample_gap(&o,150000001)&&o.max_sample_gap_ns==50000001&&o.members[0].ri_started_ns==100000000,"known live and paused RI start gaps over50ms refuse without resetting prior evidence");
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
    unsigned retained=argc==2&&!strcmp(argv[1],"total-retained-guard-api");
    unsigned selected=argc==3&&!strcmp(argv[1],"total-control")?(unsigned)strtoul(argv[2],NULL,10):12;
    unsigned hold=argc==2&&!strcmp(argv[1],"hold-lifetime-control");
    unsigned charge=argc==2&&!strcmp(argv[1],"charge-diagnostic-control");
    unsigned flow=argc==2&&!strcmp(argv[1],"flow-controls");
    unsigned result_only=argc==2&&!strcmp(argv[1],"result-controls");
    unsigned diagnostics=argc==2&&!strcmp(argv[1],"terminal-diagnostic-controls");
    unsigned setup=argc==2&&!strcmp(argv[1],"diagnostic-setup-budget-control");
    unsigned deadline=argc==2&&!strcmp(argv[1],"terminal-deadline-controls");
    unsigned cpu_debt=argc==2&&!strcmp(argv[1],"helper-cpu-debt-control");
    unsigned expand=argc==2&&!strcmp(argv[1],"argv-expansion-controls");
    unsigned native_api=argc==2&&!strcmp(argv[1],"process-create-policy-api");
    unsigned native_case=argc==3&&!strcmp(argv[1],"process-create-wall-control")?(unsigned)strtoul(argv[2],NULL,10):3;
    unsigned race_case=argc==3&&!strcmp(argv[1],"live-sample-race-control")?(unsigned)strtoul(argv[2],NULL,10):7;
    unsigned event_api=argc==2&&!strcmp(argv[1],"verifier-event-api");
    unsigned event_case=argc==3&&!strcmp(argv[1],"verifier-event-control")?(unsigned)strtoul(argv[2],NULL,10):3;
    require(argc==1||selected<12||diagnostics||setup||deadline||cpu_debt||expand||native_api||native_case<3||race_case<7||event_api||event_case<3||hold||flow||result_only||charge||boundary||retained,"private source-known controls argc");
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
    if(result_only){result_controls();require(close(dir)==0,"owned result root close");return 0;}
    if(boundary){total_boundary_api();require(close(dir)==0,"owned boundary API root close");return 0;}
    if(retained){total_retained_guard_api();require(close(dir)==0,"owned retained guard API root close");return 0;}
    if(selected<12){total_control(self,root,dir,selected);require(close(dir)==0,"owned narrow fixture root close");return 0;}
    if(diagnostics){terminal_diagnostic_controls(dir);require(close(dir)==0,"owned diagnostic root close");return 0;}
    if(event_api){verifier_event_api();require(close(dir)==0,"owned event API root close");return 0;}
    if(event_case<3){int code=verifier_event_control(self,root,dir,event_case);require(close(dir)==0,"owned event control root close");return code;}
    if(race_case<7){int code=live_sample_race_control(self,root,dir,race_case);require(close(dir)==0,"owned live race root close");return code;}
    if(native_api){process_create_policy_api();require(close(dir)==0,"owned native policy API root close");return 0;}
    if(native_case<3){int code=process_create_wall_control(self,root,dir,native_case);require(close(dir)==0,"owned native policy process root close");return code;}
    if(expand){argv_expansion_controls(dir);require(close(dir)==0,"owned argv expansion root close");return 0;}
    if(cpu_debt){helper_cpu_debt_control(self,root,dir);require(close(dir)==0,"owned CPU-debt root close");return 0;}
    if(deadline){terminal_deadline_controls(dir);require(close(dir)==0,"owned terminal-deadline root close");return 0;}
    if(setup){diagnostic_setup_budget_control();require(close(dir)==0,"owned setup root close");return 0;}
    argv_expansion_controls(dir);semantic_controls();result_controls();flow_controls(root,dir);terminal_diagnostic_controls(dir);terminal_deadline_controls(dir);charge_diagnostic_control(dir);
    for(unsigned mode=0;mode<12;mode++)total_control(self,root,dir,mode);
    helper_cpu_debt_control(self,root,dir);
    process_create_policy_api();
    for(unsigned mode=0;mode<3;mode++)(void)process_create_wall_control(self,root,dir,mode);
    for(unsigned mode=0;mode<7;mode++)(void)live_sample_race_control(self,root,dir,mode);
    verifier_event_api();
    for(unsigned mode=0;mode<3;mode++)(void)verifier_event_control(self,root,dir,mode);
    total_boundary_api();
    total_retained_guard_api();
    quarantine_hold_process_control(self,root,dir);
    require(close(dir)==0,"owned fixture root close");
    printf("private_engine_controls=pass cases=%u total_budget_controls=8 synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);return 0;
}
