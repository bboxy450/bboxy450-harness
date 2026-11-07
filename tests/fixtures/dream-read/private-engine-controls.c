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
struct fixture_birth_observation {
    pid_t pid;const char *stage;uint64_t cpu_begin,cpu_end,wall_begin,wall_end,inclusive,application;
};
static struct fixture_birth_observation fixture_birth;
struct fixture_unit_observation {
    const char *kind,*site;unsigned line,ordinary;int returned;
    uint64_t open,end,supplied_actual,supplied_cost,actual,cost;
};
static struct fixture_unit_observation fixture_unit;
static struct engine_owner *fixture_unit_owner;
static struct engine_unit_observation fixture_engine_failed;
static uint64_t fixture_startup_cpu,fixture_startup_ticks;
static unsigned fixture_startup_charged;
static void (*const fixture_canonical_require)(int,const char *)=require;
static _Noreturn void fixture_setup_failure(pid_t,const char *,uint64_t,uint64_t);
static int fixture_charge_at(struct engine_owner *owner,uint64_t actual,uint64_t cost,int ordinary,const char *site,unsigned line) {
    uint64_t open=private_cost.open_cpu;int returned=engine_charge_at(owner,actual,cost,ordinary,site,line);
    fixture_engine_failed=owner->failed_unit_last;fixture_startup_cpu=owner->startup_cpu;
    fixture_startup_ticks=owner->members[0].state.raw_rusage_cpu;fixture_startup_charged=owner->startup_charged;
    fixture_unit=(struct fixture_unit_observation){"engine_charge",site,line,(unsigned)ordinary,returned,
        open,private_cost.open_cpu,actual,cost,private_cost.last_unit,owner->last_charged_cost};
    return returned;
}
static int fixture_end_unit(uint64_t at,const char *site,unsigned line) {
    uint64_t open=private_cost.open_cpu;int returned=private_end_unit(at);
    fixture_unit=(struct fixture_unit_observation){"private_end_unit",site,line,1,returned,
        open,private_cost.open_cpu,0,0,private_cost.last_unit,private_cost.last_unit};
    return returned;
}
static void fixture_require(int ok,const char *label,const char *site,unsigned line) {
    if(ok)return;
    int original_errno=errno;uint64_t wall=UINT64_MAX,cpu=UINT64_MAX;
    int wall_known=known_clock_ns(CLOCK_MONOTONIC,&wall),cpu_known=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu);
    const struct engine_owner *owner=fixture_unit_owner;
    struct engine_unit_observation failed=owner?owner->failed_unit_last:fixture_engine_failed;
    // One bounded failure record serves EVERY fixture unit guard. A negative
    // test's earlier error remains visible without being confused with this
    // assertion's own cost/clock/debt or original native allocation.
    fprintf(stderr,"private_fixture_guard_failure_source_known assertion_site=%s assertion_line=%u label=%s unit_kind=%s unit_site=%s unit_line=%u unit_return=%d ordinary=%u open_cpu_ns=%llu end_cpu_ns=%llu supplied_actual_ns=%llu supplied_cost_ns=%llu actual_ns=%llu cost_ns=%llu engine_failed_site=%s engine_failed_line=%u engine_failed_actual_ns=%llu engine_failed_cost_ns=%llu wall_known=%d wall_ns=%llu cpu_known=%d cpu_ns=%llu original_deadline_ns=%llu rest_until_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu retained_units=%llu retained_cost_ns=%llu retained_unknown_units=%llu retained_overflow=%u startup_cpu_ns=%llu startup_charged=%u startup_raw_ticks=%llu current_original_pid=%d runtime_approved=false\n",
        site,line,label,fixture_unit.kind?fixture_unit.kind:"none",fixture_unit.site?fixture_unit.site:"none",fixture_unit.line,fixture_unit.returned,fixture_unit.ordinary,
        (unsigned long long)fixture_unit.open,(unsigned long long)fixture_unit.end,(unsigned long long)fixture_unit.supplied_actual,
        (unsigned long long)fixture_unit.supplied_cost,(unsigned long long)fixture_unit.actual,(unsigned long long)fixture_unit.cost,
        failed.site?failed.site:"none",failed.line,(unsigned long long)failed.actual,(unsigned long long)failed.cost,
        wall_known,(unsigned long long)wall,cpu_known,(unsigned long long)cpu,(unsigned long long)private_cost.deadline,
        (unsigned long long)private_cost.rest_until,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,
        (unsigned long long)private_cost.retained_units,(unsigned long long)private_cost.retained_cost_ns,
        (unsigned long long)private_cost.retained_unknown_units,private_cost.retained_overflow,
        (unsigned long long)(owner?owner->startup_cpu:fixture_startup_cpu),owner?owner->startup_charged:fixture_startup_charged,
        (unsigned long long)(owner?owner->members[0].state.raw_rusage_cpu:fixture_startup_ticks),retained_group);
    (void)fflush(stderr);
    // Product/private holder callbacks keep their own established custody
    // protocol. Otherwise this fixture's original unreaped positive PID is
    // cleaned up explicitly BEFORE the unchanged assertion exits81.
    if(!dispose_private_engine&&retained_group>0) {
        errno=original_errno;fixture_setup_failure(retained_group,label,fixture_unit.open,fixture_unit.actual);
    }
    (void)private_end_unit(private_cost.open_cpu);errno=original_errno;fixture_canonical_require(0,label);
}
#define require(ok,label) fixture_require((ok),(label),__func__,__LINE__)
#define engine_charge_at(owner,actual,cost,ordinary,site,line) fixture_charge_at((owner),(actual),(cost),(ordinary),(site),(line))
#define private_end_unit(at) fixture_end_unit((at),__func__,__LINE__)
static void reset(void) {
    memset(&fixture_unit,0,sizeof(fixture_unit));memset(&fixture_engine_failed,0,sizeof(fixture_engine_failed));fixture_unit_owner=NULL;
    memset(&fixture_birth,0,sizeof(fixture_birth));
    fixture_startup_cpu=fixture_startup_ticks=fixture_startup_charged=0;
    memset(&private_cost,0,sizeof(private_cost));private_reset_diagnostics();private_failed=0;private_error="input_invalid";private_detail="fixture unset";
    private_cost.entry=clock_ns(CLOCK_MONOTONIC);private_cost.deadline=private_cost.entry+30000000000ULL;
    private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);engine_cancel_signal=0;
    source_known_payment_interrupts=source_known_payment_fail_nth=source_known_schedule_interrupts=source_known_schedule_fail_nth=0;
    source_known_tail_over_cpu=source_known_closure_over_cpu=source_known_closure_late_stop=0;
    source_known_stopped_stage_cpu=0;
    source_known_initial_ri_unavailable=source_known_initial_payment_interrupt=source_known_initial_ri_calls=0;
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
    // Scripted metadata for this low-level flow seam; no actual wait4 or
    // candidate conservation observation is claimed by these counters.
    owner->result->terminal_conservation_checked++;owner->result->terminal_conservation_passed++;
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
    r->terminal_conservation_checked=r->terminal_conservation_passed=15;
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
    reset();strcpy(outer.case_name,"positive");nominal_result(&r);r.terminal_conservation_passed=14;
    require(!engine_complete_result(&context,&r),"every completed actual command needs its own hard terminal conservation pass");controls++;
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
static _Noreturn void fixture_setup_failure(pid_t pid,const char *stage,uint64_t epoch,uint64_t raw) {
    int original_errno=errno;uint64_t charged=private_cost.debt,paid=private_cost.debt_paid;
    struct proc_bsdinfo b;memset(&b,0,sizeof(b));int got=pid>0?proc_pidinfo(pid,PROC_PIDTBSDINFO,0,&b,sizeof(b)):0;
    int identity=got==(int)sizeof(b)&&b.pbi_pid==(uint32_t)pid&&b.pbi_ppid==(uint32_t)getpid();
    uint64_t birth=identity?(uint64_t)b.pbi_start_tvsec*1000000+b.pbi_start_tvusec:0;
    fprintf(stderr,"{\"kind\":\"fixture_setup_failure\",\"stage\":\"%s\",\"pid\":%d,\"parent\":%d,\"birth_known\":%s,\"birth\":%llu,\"epoch_cpu_ns\":%llu,\"closed_raw_cpu_ns\":%llu,\"debt_charged_ns\":%llu,\"debt_paid_ns\":%llu,\"failure_code\":\"%s\",\"original_errno\":%d,\"native_record_matches\":%s,\"native_cpu_begin_ns\":%llu,\"native_cpu_end_ns\":%llu,\"native_wall_begin_ns\":%llu,\"native_wall_end_ns\":%llu,\"native_inclusive_cpu_ns\":%llu,\"native_application_cpu_ns\":%llu,\"runtime_approved\":false}\n",
        stage,pid,getpid(),identity?"true":"false",(unsigned long long)birth,(unsigned long long)epoch,(unsigned long long)raw,
        (unsigned long long)charged,(unsigned long long)paid,private_error,original_errno,pid>0&&fixture_birth.pid==pid?"true":"false",
        (unsigned long long)fixture_birth.cpu_begin,(unsigned long long)fixture_birth.cpu_end,
        (unsigned long long)fixture_birth.wall_begin,(unsigned long long)fixture_birth.wall_end,
        (unsigned long long)fixture_birth.inclusive,(unsigned long long)fixture_birth.application);
    int killed=-1,kill_error=0,status=0,done=0,own=0,own_error=0,signal_attempted=0;
    pid_t joined=0;struct rusage usage;memset(&usage,0,sizeof(usage));
    int pinned=pid>0&&(retained_group==pid||retained_child==pid);
    if(pinned) {errno=0;own=known_exited(pid,&done);own_error=own?0:errno;}
    if(pinned&&own) {
        // The exact positive fork return is STILL this parent's original
        // unreaped child. Never signal after a consuming/lost wait.
        if(!done){signal_attempted=1;errno=0;killed=kill(pid,SIGKILL);kill_error=killed?errno:0;}
        for(;;) {
            joined=wait4(pid,&status,WNOHANG,&usage);
            if(joined==pid){if(retained_group==pid)retained_group=0;if(retained_child==pid)retained_child=0;break;}
            if(joined<0&&errno!=EINTR) {
                if(errno==ECHILD){if(retained_group==pid)retained_group=0;if(retained_child==pid)retained_child=0;}
                break;
            }
            uint64_t now;if(!known_clock_ns(CLOCK_MONOTONIC,&now)||now>=private_cost.deadline)break;
            uint64_t cpu;if(known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu)&&private_cost.open_cpu!=UINT64_MAX
                &&cpu>=private_cost.open_cpu&&cpu-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
            usleep(1000);
        }
    } else if(pinned&&own_error==ECHILD) {
        if(retained_group==pid)retained_group=0;if(retained_child==pid)retained_child=0;
    }
    uint64_t user=joined==pid&&pid>0?engine_timeval(&usage.ru_utime):UINT64_MAX;
    uint64_t system=joined==pid&&pid>0?engine_timeval(&usage.ru_stime):UINT64_MAX,helper=UINT64_MAX;
    if(user!=UINT64_MAX&&system!=UINT64_MAX&&engine_safe_sum(user,system,&helper))
        if(!engine_safe_sum(helper,2000,&helper))helper=UINT64_MAX;
    struct private_engine_result cleanup_result;struct engine_owner cleanup_owner;
    memset(&cleanup_result,0,sizeof(cleanup_result));memset(&cleanup_owner,0,sizeof(cleanup_owner));cleanup_owner.context=&context;cleanup_owner.result=&cleanup_result;
    uint64_t ended=UINT64_MAX;int ended_known=known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&ended);
    uint64_t used=ended_known?engine_interval(private_cost.open_cpu,ended):UINT64_MAX,cost=used;
    if(pid>0&&!engine_safe_sum(used,helper,&cost))cost=UINT64_MAX;
    (void)engine_charge(&cleanup_owner,used,cost,1); // No residual or startup RHS credit on fixture failure.
    int group_ids[3]={0};errno=0;int group=pid>0?proc_listpgrppids(pid,group_ids,sizeof(group_ids)):-1,group_error=group<0?errno:0;
    fprintf(stderr,"{\"kind\":\"fixture_setup_cleanup\",\"pid\":%d,\"birth_known\":%s,\"birth\":%llu,\"original_unreaped_pin\":%d,\"own_WNOWAIT_known\":%d,\"own_WNOWAIT_errno\":%d,\"own_WNOWAIT_terminal\":%d,\"signal_attempted\":%d,\"signal_result\":%d,\"signal_errno\":%d,\"actual_wait4_pid\":%d,\"actual_reaped\":%s,\"wait_status\":%d,\"wait4_user_ns\":%llu,\"wait4_system_ns\":%llu,\"full_helper_cost_ns\":%llu,\"group_result\":%d,\"group_errno\":%d,\"retained_original_group_pid\":%d,\"debt_charged_ns\":%llu,\"debt_paid_ns\":%llu,\"original_deadline_ns\":%llu,\"runtime_approved\":false}\n",
        pid,identity?"true":"false",(unsigned long long)birth,pinned,own,own_error,done,signal_attempted,killed,kill_error,joined,joined==pid&&pid>0?"true":"false",status,
        (unsigned long long)user,(unsigned long long)system,(unsigned long long)helper,group,group_error,retained_group,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,(unsigned long long)private_cost.deadline);
    (void)fflush(stderr);(void)private_end_unit(private_cost.open_cpu);
    fprintf(stderr,"probe refused: %s errno=%d\n",stage,original_errno);exit(81);
}
static pid_t fixture_fork(const char *stage) {
    uint64_t preparation=private_cost.open_cpu;
    if(!private_end_unit(preparation))fixture_setup_failure(0,"fixture prebirth preparation unit refused",preparation,private_cost.last_unit);
    uint64_t native_begin=clock_ns(CLOCK_THREAD_CPUTIME_ID),wall_begin=clock_ns(CLOCK_MONOTONIC);
    pid_t pid=fork();if(pid==0)return 0;
    if(pid>0)retained_group=pid; // Retain the positive effect BEFORE either ending clock.
    uint64_t wall_end=clock_ns(CLOCK_MONOTONIC),native_end=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    struct private_engine_result result;struct engine_owner owner;memset(&result,0,sizeof(result));memset(&owner,0,sizeof(owner));
    owner.context=&context;owner.result=&result;
    uint64_t native=engine_interval(native_begin,native_end),wall=engine_interval(wall_begin,wall_end);
    uint64_t before=private_cost.open_cpu,total=engine_interval(before,clock_ns(CLOCK_THREAD_CPUTIME_ID));
    int charged=engine_charge(&owner,total,total,0);total=private_cost.last_unit;
    uint64_t application=total!=UINT64_MAX&&total>=native?total-native:UINT64_MAX;
    fixture_birth=(struct fixture_birth_observation){pid,stage,native_begin,native_end,wall_begin,wall_end,total,application};
    fprintf(stderr,"{\"kind\":\"fixture_native_birth\",\"stage\":\"%s\",\"pid\":%d,\"parent\":%d,\"native_cpu_begin_ns\":%llu,\"native_cpu_end_ns\":%llu,\"native_cpu_ns\":%llu,\"native_wall_begin_ns\":%llu,\"native_wall_end_ns\":%llu,\"native_wall_ns\":%llu,\"inclusive_cpu_ns\":%llu,\"application_cpu_ns\":%llu,\"cost_paid\":%s,\"debt_charged_ns\":%llu,\"debt_paid_ns\":%llu,\"runtime_approved\":false}\n",
        stage,pid,getpid(),(unsigned long long)native_begin,(unsigned long long)native_end,(unsigned long long)native,
        (unsigned long long)wall_begin,(unsigned long long)wall_end,(unsigned long long)wall,(unsigned long long)total,(unsigned long long)application,
        charged?"true":"false",(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    if(pid<=0)(void)private_fail("settlement_unknown","fixture fork returned no original owned child");
    else if(native>2000000||wall>25000000||total>3000000||application>1000000)
        (void)private_fail("resource_limit","fixture unchanged native or inclusive allocation exceeded");
    if(pid<=0||!charged||native>2000000||wall>25000000||total>3000000||application>1000000)
        fixture_setup_failure(pid,"fixture native birth allocation refused",before,total);
    return pid;
}
static struct image fixture_stopped(pid_t pid) {
    // The fixture child must have reached its OWN initial stop before the
    // tight production observation. Pacing this setup avoids starving a newly
    // forked utility-priority child in a controller busy loop. No native/1ms
    // production gate or existing100ms readiness bound is widened.
    uint64_t start=fixture_birth.pid==pid?fixture_birth.wall_end:clock_ns(CLOCK_MONOTONIC);
    require(start<=UINT64_MAX-100000000ULL,"fixture original readiness deadline checked sum");
    uint64_t until=start+100000000ULL;
    for(;;) {
        siginfo_t info;memset(&info,0,sizeof(info));
        require(waitid(P_PID,pid,&info,WSTOPPED|WNOHANG|WNOWAIT)==0,"fixture original nonreaping stop notification");
        if(info.si_pid==pid&&info.si_code==CLD_STOPPED) {
            struct image first;require(control_observe(pid,&first),"fixture original stopped birth");
            (void)control_stop_ack(pid,first.birth);
            uint64_t before=private_cost.open_cpu;
            if(!private_end_unit(before))fixture_setup_failure(pid,"fixture initial stopped-observation unit refused",before,private_cost.last_unit);
            return first;
        }
        if(clock_ns(CLOCK_MONOTONIC)>=until) {
            fixture_setup_failure(pid,"fixture initial stop readiness exhausted",private_cost.open_cpu,private_cost.last_unit);
        }
        uint64_t used=clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu;
        uint64_t before=private_cost.open_cpu;
        if(used>=200000&&!private_end_unit(before)) {
            fixture_setup_failure(pid,"fixture initial stop setup unit refused",before,private_cost.last_unit);
        }
        usleep(100);
    }
}
static void closure_stage_controls(const char *self,int dir) {
    for(unsigned variant=0;variant<2;variant++) {
        reset();outer.deadline=private_cost.deadline;strcpy(outer.case_name,"positive");
        require(engine_owner_priority(),"closure control bounded setup and priority");
        pid_t pid=fixture_fork("closure_stage_controls");
        if(pid==0){if(setpgid(0,0)!=0||raise(SIGSTOP)!=0)_exit(79);volatile unsigned work=1;for(;;)work=work*33+7;}
        require(pid>0,"owned source-known closure fork");retained_group=pid;
        struct image first=fixture_stopped(pid);
        struct private_engine_result result;memset(&result,0,sizeof(result));result.created=1;result.phase="git_reads";result.outcome="refused";result.observed_proof="not_reached";
        struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
        owner.input=owner.output=owner.errors=-1;owner.slots=1;owner.closed=1;
        owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;strcpy(owner.image,self);
        int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"closure source-known original final image admitted stopped");
        owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
        owner.ledger=openat(dir,variant?"closure-over-cpu.ndjson":"closure-late-stop.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(owner.ledger>=0&&private_end_unit(private_cost.open_cpu),"closure controller preparation is an ordinary paid unit");
        uint64_t before=private_cost.debt;
        source_known_closure_late_stop=!variant;source_known_closure_over_cpu=variant;
        require(!engine_grant(&owner),"actual overlong closure is refused by same grant machinery");
        source_known_closure_late_stop=source_known_closure_over_cpu=0;
        require(owner.failed&&owner.closure_limit_failed&&(owner.grant.charge>1000000||owner.grant.windows>25000000)&&private_errors[5].count
            &&private_cost.debt>before&&private_cost.debt==private_cost.debt_paid,
            "unsplit over-wall or over-controller closure remains refused and paid9x");
        const struct engine_stage_observation *stage=&owner.closure_stage[variant?ENGINE_STAGE_ROSTER:ENGINE_STAGE_STOP];
        require(stage->calls&&stage->wall_end>=stage->wall_begin&&stage->cpu_end>=stage->cpu_begin
            &&stage->max_wall>=1100000&&(!variant||stage->max_cpu>=1100000),
            "fixed actual closure stage clocks expose delayed STOP and over-CPU work");
        uint64_t W=owner.grant.windows,C=owner.grant.controller,Q=owner.grant.charge;
        int status=-1;(void)engine_retire(&owner,&status);
        unsigned held=owner.settlement_quarantined;
        if(held) {
            // A deliberately huge original grant also incurs its full9x
            // debt. Actual polling CPU can correctly refuse a subsequent
            // ordinary settlement unit. That is a custody transition, not an
            // immediate physical-reap guarantee: drive THIS original owner's
            // normal hold explicitly instead of relying on atexit cleanup.
            fprintf(stderr,"private_closure_custody_source_known variant=%u pid=%d birth=%llu prior_reaped=%u quarantined=%u last_actual_cpu_ns=%llu last_cost_ns=%llu debt_charged_ns=%llu debt_paid_ns=%llu runtime_approved=false\n",
                variant,pid,(unsigned long long)owner.ticket.birth,owner.direct_reaped,owner.settlement_quarantined,
                (unsigned long long)owner.failed_unit_last.actual,(unsigned long long)owner.failed_unit_last.cost,
                (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
            require(fflush(stdout)==0,"closure prior fixture stdout flush precedes direct refusal write");
            engine_quarantine_hold(&owner);
        }
        require(owner.direct_reaped&&result.group_gone&&result.retired_coverage&&result.reaped==1,
            "overlong closure refusal keeps original owner through actual reap and group proof");
        require(owner.failed&&private_errors[5].count&&(!held||owner.quarantine_settled),
            "physical settlement never upgrades the original closure resource refusal");
        require(fsync(owner.ledger)==0&&close(owner.ledger)==0,"owned closure raw ledger persisted");
        require(private_end_unit(private_cost.open_cpu),"closure held settlement evidence and original descriptor disposal are separately paid");
        printf("private_closure_stage_source_known variant=%u pid=%d birth=%llu actual_signal=true W=%llu C=%llu Q=%llu stage_wall_ns=%llu stage_cpu_ns=%llu lower_bound_refused=true unsplit=true debt_charged_ns=%llu debt_paid_ns=%llu explicit_same_owner_hold=%u own_reap=true wait4_observed=%u group_gone=true refused=true runtime_approved=false\n",
            variant,pid,(unsigned long long)owner.ticket.birth,(unsigned long long)W,(unsigned long long)C,(unsigned long long)Q,(unsigned long long)stage->max_wall,(unsigned long long)stage->max_cpu,
            (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,held,owner.wait_observed);controls++;
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"closure stdout formatting and actual flush are paid before the next fixture epoch");
    }
    reset();
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
static unsigned hold_scheduling_only;
static void hold_payment_stage(const struct engine_owner *owner,const char *stage) {
    printf("private_hold_payment_stage stage=%s synthetic_source_known=true debt_charged_ns=%llu debt_paid_ns=%llu retained_units=%llu retained_cost_ns=%llu retained_raw_ns=%llu retained_unknown_units=%llu closed_units=%llu booked_units=%llu prior_payment_pending=%u terminal_samples=%llu roster_checks=%llu wait4_calls=%llu group_checks=%llu final_ledger_written=%u quarantine_settled=%u\n",
        stage,(unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid,
        (unsigned long long)private_cost.retained_units,(unsigned long long)private_cost.retained_cost_ns,
        (unsigned long long)private_cost.retained_raw_ns,(unsigned long long)private_cost.retained_unknown_units,
        (unsigned long long)private_cost.closed_units,(unsigned long long)private_cost.booked_units,
        owner->hold_prior_payment_pending,(unsigned long long)owner->hold_terminal_samples,(unsigned long long)owner->hold_roster_checks,
        (unsigned long long)owner->hold_wait4_calls,(unsigned long long)owner->hold_group_checks,owner->hold_settled_recorded,owner->quarantine_settled);
}
static int retained_known_unit(void) {
    uint64_t raw=private_cost.retained_raw_ns;
    return private_cost.retained_units==1&&!private_cost.retained_unknown_units&&!private_cost.retained_overflow
        &&raw>=private_cost.last_unit&&private_cost.retained_cost_ns==(raw>1000000?raw:1000000);
}
static void interrupted_hold_payment_control(struct engine_owner *owner) {
    // Reuse this fixture's ACTUALLY reaped, final-image-bound original helper;
    // no new child/grant, controller, timer or debt reset.
    require(owner->direct_reaped&&owner->result->group_gone&&owner->result->retired_coverage,"payment control starts from actual owned terminal proof");
    (void)engine_quarantine(owner,"source-known interrupted hold-payment observation control");
    if(!hold_scheduling_only) {
        require(private_schedule_cost(1000),"source-known prior charged debt without assumed payment");
        source_known_payment_interrupts=2;
        require(!engine_hold_observe(owner)&&private_cost.debt>private_cost.debt_paid
            &&!owner->quarantine_settled&&owner->hold_terminal_samples==0&&owner->hold_roster_checks==0
            &&owner->hold_wait4_calls==0&&owner->hold_group_checks==0,
            "interrupted prior payment forbids every new hold observation");
        hold_payment_stage(owner,"scheduled_unpaid");
        uint64_t charged=private_cost.debt;source_known_payment_interrupts=1;
        require(!engine_hold_observe(owner)&&private_cost.debt>=charged&&owner->hold_terminal_samples==0
            &&owner->hold_group_checks==0&&!owner->quarantine_settled,
            "interrupted recovery cannot infer payment from debt increase or fallback sleep");
        hold_payment_stage(owner,"recovery_interrupted");
        source_known_schedule_interrupts=1;
        (void)engine_hold_unknown_cost(owner,"source-known unknown unit whose scheduling was interrupted");
        require(private_cost.retained_units==1&&private_cost.retained_cost_ns==3000000
            &&private_cost.retained_unknown_units==1&&!private_cost.retained_raw_ns,
            "shared cost owner retains raw unknown and canonical3ms after failed scheduling");
        hold_payment_stage(owner,"unknown_schedule_interrupted");
        uint64_t booked=private_cost.booked_units,before=private_cost.debt;
        source_known_payment_interrupts=2;
        require(!engine_hold_observe(owner)&&!private_cost.retained_units
            &&private_cost.booked_units==booked+2&&private_cost.debt==before+54000000
            &&private_cost.debt_paid<private_cost.debt&&owner->hold_terminal_samples==0&&owner->hold_group_checks==0,
            "retained unknown books once then interrupted payment forbids observations");
        hold_payment_stage(owner,"retained_booked_payment_interrupted");
        require(!engine_hold_observe(owner)&&!private_cost.retained_units&&private_cost.debt_paid==private_cost.debt
            &&owner->hold_terminal_samples==0&&!owner->quarantine_settled,
            "known recovery pays retained booked debt before allowing observations");
        hold_payment_stage(owner,"retained_booked_payment_known");
    }
    if(hold_scheduling_only!=2) {
        source_known_schedule_interrupts=1;
        require(!engine_hold_observe(owner)&&retained_known_unit()&&owner->hold_terminal_samples>0
            &&owner->hold_group_checks>0&&!owner->hold_settled_recorded&&!owner->quarantine_settled,
            "ordinary hold scheduling failure retains exact inclusive closed cost before another observation");
        uint64_t samples=owner->hold_terminal_samples,groups=owner->hold_group_checks,late=owner->hold_late_charged;
        hold_payment_stage(owner,"ordinary_schedule_interrupted");
        require(!engine_hold_observe(owner)&&!private_cost.retained_units&&private_cost.debt_paid==private_cost.debt
            &&owner->hold_terminal_samples==samples&&owner->hold_group_checks==groups
            &&owner->hold_late_charged==late&&!owner->quarantine_settled,
            "ordinary retained cost is booked and paid before further observations without repeating late cost");
        hold_payment_stage(owner,"ordinary_retained_cost_known_paid");
    }
    // First schedule closes the actual observation; second closes the actual
    // once-written settled-refusal ledger/fsync. The latter must survive its
    // failed scheduling even though hold_settled_recorded is already sticky.
    source_known_schedule_fail_nth=2;
    require(!engine_hold_observe(owner)&&owner->hold_settled_recorded&&retained_known_unit()
        &&!owner->quarantine_settled&&!owner->result->group_gone&&!owner->result->retired_coverage,
        "final ledger scheduling failure retains its exact once-written cost and keeps custody held");
    uint64_t samples=owner->hold_terminal_samples,groups=owner->hold_group_checks,records=owner->record_count;
    hold_payment_stage(owner,"final_ledger_schedule_interrupted");
    require(!engine_hold_observe(owner)&&!private_cost.retained_units&&private_cost.debt_paid==private_cost.debt
        &&owner->hold_terminal_samples==samples&&owner->hold_group_checks==groups&&owner->record_count==records
        &&owner->hold_settled_recorded&&!owner->quarantine_settled,
        "final retained ledger cost is paid before new observations without rewriting its onceflag");
    hold_payment_stage(owner,"final_ledger_retained_cost_known_paid");
    while(!owner->quarantine_settled&&clock_ns(CLOCK_MONOTONIC)<outer.deadline) {
        (void)engine_hold_observe(owner);if(!owner->quarantine_settled)usleep(1000);
    }
    require(owner->quarantine_settled&&!private_cost.retained_units&&!private_cost.retained_overflow
        &&private_cost.debt_paid==private_cost.debt&&owner->result->group_gone&&owner->result->retired_coverage
        &&!strcmp(owner->result->outcome,"refused"),
        "only known-paid prior fresh and final cost permits settled refusal");
    printf("private_hold_interrupted_payment_source_known interrupted_payment=%s failed_schedule=true ordinary_and_final_cost_retained=true closed_units=%llu booked_units=%llu retained_units=0 prior_unpaid_observations=0 recovered_terminal_samples=%llu recovered_group_checks=%llu debt_charged_ns=%llu debt_paid_ns=%llu settled_refusal=true new_helpers=0 runtime_approved=false\n",
        hold_scheduling_only?"not_in_this_narrow_mode":"true",(unsigned long long)private_cost.closed_units,(unsigned long long)private_cost.booked_units,
        (unsigned long long)owner->hold_terminal_samples,(unsigned long long)owner->hold_group_checks,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
}
static void process_control(const char *self,int live) {
    reset();require(engine_owner_priority(),"source-known controller priority");
    // A fixture-only fork isolates this terminal-classification witness from
    // dyld/exec latency. It is never a candidate helper-birth qualification.
    // The native fork still obeys the existing2msCPU/25mswall bounds; all
    // original-token/RI/WNOWAIT/roster/window/actual-reap code is canonical.
    pid_t pid=fixture_fork(live?"process_control_live":"process_control_terminal");
    if(pid==0) {
        if(setpgid(0,0)!=0)_exit(71);
        if(raise(SIGSTOP)!=0)_exit(72);
        if(live){volatile uint64_t work=1;for(;;)work=work*33+7;}
        _exit(0);
    }
    if(pid>0)retained_group=pid;
    require(pid>0,"source-known original fork return");
    struct image first=fixture_stopped(pid);
    struct private_engine_result result;memset(&result,0,sizeof(result));result.created=1;result.phase="git_reads";result.outcome="refused";result.observed_proof="not_reached";
    (void)control_stop_ack(pid,first.birth);
    struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
    owner.input=owner.output=owner.errors=owner.ledger=-1;owner.slots=1;owner.ticket=(struct engine_ticket){pid,pid,first.birth,0,1};
    strcpy(owner.image,self);owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;
    (void)control_stop_ack(pid,first.birth);
    int ack=0;require(engine_sample_member(&owner,0,pid,&ack)&&ack,"original stopped attach before user work");
    owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;
    uint64_t attached=private_cost.open_cpu;
    if(!private_end_unit(attached))fixture_setup_failure(pid,"fixture original attach unit refused",attached,private_cost.last_unit);
    struct pending_stdin_fixture transport;pending_stdin_prepare(&owner,&transport);
    uint64_t transport_prepared=private_cost.open_cpu;
    if(!private_end_unit(transport_prepared))fixture_setup_failure(pid,"fixture pending transport preparation unit refused",transport_prepared,private_cost.last_unit);
    owner.grant.operation=owner.ticket.start;owner.grant.sequence=1;owner.grant.slots=1;owner.grant.source_complete=1;owner.grant.cpu_start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
    uint64_t cpu=0,wall=0;
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
    } else require(owner.stop_terminal_candidate&&!owner.stop_terminal&&owner.grant.end[0]>=failed_at,
        "independent terminal candidate never substitutes RI or errno for actual wait4");
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
    if(!live)require(owner.stop_terminal&&owner.stop_terminal_ns>=failed_at,"terminal accounting follows original actual wait4, never RI alone");
    if(!live)interrupted_hold_payment_control(&owner);
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
    uint64_t disposed=private_cost.open_cpu;
    if(!private_end_unit(disposed))fixture_setup_failure(0,"fixture terminal diagnostics and disposal unit refused",disposed,private_cost.last_unit);
    reset();controls++;
}
static void pending_exec_control(const char *self,int dir) {
    reset();outer.deadline=private_cost.deadline;strcpy(outer.case_name,"positive");
    int gate[2],ready[2];require(pipe(gate)==0&&pipe(ready)==0,"owned source-known exec gate and completion");
    pid_t pid=fixture_fork("pending_exec_control");
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
    unsigned kills=owner.retired[1].kill_attempted;
    require(!engine_hold_observe(&owner)&&owner.retired[1].kill_attempted==kills&&owner.stop_attempts==1,
        "one held observation never repeats pending STOP, signals or releases an exec-unknown owner");
    if(!owner.direct_reaped) {
        int done=0;while(!done&&clock_ns(CLOCK_MONOTONIC)<outer.deadline){require(known_exited(pid,&done),"fixture original WNOWAIT after safe exec cleanup");if(!done)usleep(100);}
        require(done&&waitpid(pid,NULL,0)==pid,"fixture actual exec negative reap within original deadline");
    }
    int group[3];require(proc_listpgrppids(pid,group,sizeof(group))==0,"fixture exec negative actual group gone");retained_group=0;
    printf("private_pending_exec_source_known actual_exec=true pid=%d birth=%llu original_version=%u successor_version=%u sticky_refusal=true original_terminal_proved=false actual_cleanup_reaped=true group_gone=true runtime_approved=false\n",
        pid,(unsigned long long)owner.ticket.birth,old_version,owner.retired[1].token.val[7]);
    // API fault-injection cleanup is performed by this source-known fixture;
    // it is not a product-owner exit or proof that the lost source premise healed.
    require(close(owner.ledger)==0,"owned exec ledger close");engine_current=NULL;dispose_private_engine=NULL;reset();controls++;
}
static void settlement_control(const char *self,int dir,unsigned variant) {
    unsigned unknown_deadline=variant==1,persistence_failure=variant==2;
    reset();outer.output.fd=dir;outer.deadline=private_cost.deadline;strcpy(outer.case_name,"positive");
    pid_t pid=fixture_fork("settlement_control");
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
    uint64_t exit_begin=clock_ns(CLOCK_MONOTONIC);
    if(!unknown_deadline)require(!engine_hold_observe(&owner),"unknown-slot observation cannot release a held owner");
    uint64_t exit_end=clock_ns(CLOCK_MONOTONIC);
    require(owner.direct_reaped==direct_reaped&&r.reaped==reported_reaps&&owner.group_kill_attempted==group_kills
        &&!r.group_gone&&!r.retired_coverage&&exit_end<outer.deadline,"single held observation retains quarantine without replaying effects or asserting completion");
    if(variant==0) {
        uint64_t before=private_cost.debt;source_known_hold_clock_unknown=1;
        require(!engine_hold_observe(&owner),"unknown hold entry clock cannot settle or release ownership");source_known_hold_clock_unknown=0;
        require(owner.hold_unknown_units==1&&owner.hold_conservative_sleep&&private_cost.debt>=before+27000000
            &&private_cost.debt_paid==private_cost.debt&&!r.group_gone&&!r.retired_coverage,
            "unknown observation retains raw unknown plus canonical3ms27ms paid debt, never zero");
    }
    if(!owner.direct_reaped) {
        if(unknown_deadline)require(engine_token_signal(&owner.members[0].token,SIGKILL)==0,"external API fixture cleanup of its injected-deadline helper");
        int done=0;while(!done&&clock_ns(CLOCK_MONOTONIC)<outer.deadline){require(known_exited(pid,&done),"fixture original WNOWAIT after consumed kill");if(!done)usleep(100);}
        require(done&&waitpid(pid,NULL,0)==pid,"fixture owner actually reaps its original within unchanged deadline");
    }
    int group[3];require(proc_listpgrppids(pid,group,sizeof(group))==0,"fixture negative actual physical group gone");retained_group=0;
    printf("private_settlement_source_known injected_unknown_deadline=%u injected_unknown_slot=%u pid=%d original_deadline=%llu return_wall_ns=%llu atexit_wall_ns=%llu reported_created=%u reported_reaped=%u reported_group_gone=false reported_retired_coverage=false quarantine_retained_in_process=true full_audit_receipt=%s live_kernel_custody_after_exit=false recovery_handoff_proved=false fixture_cleanup_actual_reaped=true runtime_approved=false\n",
        unknown_deadline,!unknown_deadline,pid,(unsigned long long)outer.deadline,(unsigned long long)(ended-began),(unsigned long long)(exit_end-exit_begin),r.created,r.reaped,owner.quarantine_durable?"true":"false");
    // Remove only this fixture's injected metadata after its separate exact
    // physical cleanup; the production unknown-slot branch remains held.
    require(close(owner.ledger)==0,"owned quarantine raw ledger close");engine_current=NULL;dispose_private_engine=NULL;reset();controls++;
}
static void hold_observer_checkpoint(void) {
    if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)
        require(private_end_unit(private_cost.open_cpu),"source-known holder observer ordinary unit and rest");
}
struct hold_witness_info {pid_t helper;uint64_t birth;int fd;uint64_t dev,ino,deadline;};
struct hold_fixture_cleanup_state {
    pid_t controller,helper;int controller_reaped,controller_status,controller_status_known,release_fd,identity_known;
    uint64_t birth,start,deadline;audit_token_t token;
};
static struct hold_fixture_cleanup_state hold_fixture_cleanup={.release_fd=-1};
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
static void hold_fixture_failure_cleanup(void) {
    struct hold_fixture_cleanup_state *s=&hold_fixture_cleanup;
    dispose_private_engine=NULL;
    int released=0;
    if(s->release_fd>=0) {
        int fd=s->release_fd;s->release_fd=-1;
        // EOF releases only this fixture's existing one-byte gate. The
        // original controller can reap normally; an orphaned negative helper
        // also exits. No PID/group signal or invented wait4 follows.
        released=close(fd)==0;
    }
    int terminal=0,gone=0,status=0,expired=0;
    struct engine_incarnation original={.pid=s->helper,.birth=s->birth,.start=s->start,.token=s->token,.current=1};
    for(;;) {
        if(!s->controller_reaped) {
            pid_t joined=hold_fixture_wait(&status);
            if(joined<0&&errno!=EINTR)break;
        }
        if(s->identity_known)terminal=engine_physical_exit(&original);
        int pids[3]={0};gone=s->helper>0&&proc_listpgrppids(s->helper,pids,sizeof(pids))==0;
        if(s->controller_reaped&&terminal&&gone)break;
        if(s->controller_reaped&&!s->identity_known)break; // Unknown identity cannot improve by waiting.
        uint64_t now;if(!known_clock_ns(CLOCK_MONOTONIC,&now)||now>=s->deadline){expired=1;break;}
        uint64_t cpu;if(known_clock_ns(CLOCK_THREAD_CPUTIME_ID,&cpu)&&private_cost.open_cpu!=UINT64_MAX
            &&cpu>=private_cost.open_cpu&&cpu-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
        usleep(1000);
    }
    (void)private_end_unit(private_cost.open_cpu);
    fprintf(stderr,"private_hold_fixture_cleanup_source_known controller=%d controller_reaped=%d controller_status_known=%d controller_status_raw=%d retained_controller_pid=%d helper=%d helper_birth=%llu helper_start=%llu helper_identity_known=%d original_helper_terminal_observed=%d original_group_gone=%d release_gate_closed=%d signals_sent=0 helper_reaped_by_observer=false deadline_exhausted=%d debt_charged_ns=%llu debt_paid_ns=%llu runtime_approved=false\n",
        s->controller,s->controller_reaped,s->controller_status_known,s->controller_status,retained_child,s->helper,
        (unsigned long long)s->birth,(unsigned long long)s->start,s->identity_known,terminal,gone,released,expired,
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
        pid_t helper=fixture_fork("quarantine_hold_helper");require(helper>=0,"source-known held helper fork");
        if(helper==0) {
            close(status_pipe[1]);if(setpgid(0,0)!=0||raise(SIGSTOP)!=0)_exit(78);
            char byte;if(read(release[0],&byte,1)!=1)_exit(79);close(release[0]);_exit(0);
        }
        retained_group=helper;close(release[0]);struct image first=fixture_stopped(helper);
        struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&r;
        owner.controller_parent=getppid();owner.input=sentinel;owner.output=owner.errors=-1;
        owner.ledger=openat(dir,"hold-ledger.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(owner.ledger>=0,"source-known holder ledger");
        owner.slots=1;owner.ticket=(struct engine_ticket){helper,helper,first.birth,0,1};owner.members[0].state.pid=helper;owner.members[0].state.birth=first.birth;
        strcpy(owner.image,self);int ack=0;require(engine_sample_member(&owner,0,helper,&ack)&&ack,"same-controller admitted original holder member");
        owner.ticket.start=owner.members[0].state.start_absolute;owner.direct_custody=1;r.attempts=r.created=1;
        owner.grant.operation=owner.ticket.start;owner.grant.sequence=owner.grant.slots=owner.grant.source_complete=1;
        owner.grant.cpu_start=private_cost.open_cpu;uint64_t raw,u,k;require(engine_actual_ticks(&owner,&raw,&u,&k),"holder original raw CPU baseline");owner.grant_tick_before=raw;
        owner.grant.t0=clock_ns(CLOCK_MONOTONIC);require(kill(-helper,SIGCONT)==0,"holder one original CONT before refusal");
        while(clock_ns(CLOCK_MONOTONIC)<outer.deadline) {
            if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
            usleep(1000);
        }
        int code=-1;require(!engine_retire(&owner,&code)&&owner.settlement_quarantined&&!owner.direct_reaped,"actual deadline enters unreaped owned quarantine before further effects");
        struct hold_witness_info info={helper,first.birth,sentinel,(uint64_t)identity.st_dev,(uint64_t)identity.st_ino,outer.deadline};
        require(write(status_pipe[1],&info,sizeof(info))==sizeof(info)&&close(status_pipe[1])==0,"source-known holder observation identity");
        (void)private_emit(&outer,input_hash,&r);
        require(owner.refusal_attempted&&owner.refusal_complete,"same controller emits the existing refusal before holding");
        engine_quarantine_hold(&owner);
        require(owner.quarantine_settled&&owner.direct_reaped&&r.reaped==1&&r.group_gone&&r.retired_coverage
            &&owner.owner_lost&&owner.owner_lost_recorded&&!strcmp(r.outcome,"refused")&&owner.group_kill_attempted==0,
            "same held controller settles itself after release and remains refused/owner-lost");
        require(fcntl(sentinel,F_GETFD)>=0,"original FD remains held through actual terminal proof");
        engine_close_transferred(&owner);require(private_close(owner.ledger),"settled holder ledger close");owner.ledger=-1;
        int output_closed=0;(void)private_close_observed(dir,&output_closed);outer.output.fd=-1;
        private_finish_diagnostics("settled",r.created,r.reaped,output_closed);exit(81);
    }
    retained_child=controller;
    memset(&hold_fixture_cleanup,0,sizeof(hold_fixture_cleanup));hold_fixture_cleanup.controller=controller;
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
    int status=0;require(hold_fixture_wait(&status)==0,"same controller remains alive after answering refusal");
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
        if(!lost){require(hold_fixture_wait(&status)==0,"owner-loss observation requires same held controller alive");usleep(1000);}
    }
    require(lost&&hold_fixture_wait(&status)==0,"owner loss is durable refusal while same controller remains alive");
    // This second independent snapshot is causally AFTER the holder entered
    // its observation loop, catching FD release after the prompt wire write.
    require(proc_pidfdinfo(controller,info.fd,PROC_PIDFDVNODEPATHINFO,&held_fd,sizeof(held_fd))==sizeof(held_fd)
        &&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_dev==info.dev&&(uint64_t)held_fd.pvip.vip_vi.vi_stat.vst_ino==info.ino,
        "independent kernel FD snapshot proves original descriptor retained after refusal");
    int release_fd=hold_fixture_cleanup.release_fd;hold_fixture_cleanup.release_fd=-1;
    ssize_t release_count=write(release_fd,"x",1);int release_closed=close(release_fd);
    require(release_count==1&&release_closed==0,"external fixture release follows custody observations; controller sends no new stdin");
    pid_t joined=0;
    while(!joined&&clock_ns(CLOCK_MONOTONIC)<observer_deadline){joined=hold_fixture_wait(&status);require(joined>=0,"owned holder join observation");hold_observer_checkpoint();if(!joined)usleep(1000);}
    require(joined==controller&&WIFEXITED(status)&&WEXITSTATUS(status)==81,"same controller exits only after its own settled refusal");retained_child=0;
    ssize_t n=read(wire[0],refusal,sizeof(refusal));require(n==0,"holder emits exactly one existing result, never a second success");
    n=pread(ledger,log,sizeof(log)-1,0);require(n>=0,"settled refusal ledger read");log[n]=0;
    require(strstr(log,"\"phase\":\"settled_refusal_owner_lost\"")!=NULL,"same controller records settled refusal without upgrading the result");
    int group[3];require(proc_listpgrppids(info.helper,group,sizeof(group))==0,"independent actual helper group gone after own-controller reap");
    dispose_private_engine=NULL;
    require(close(ledger)==0&&close(wire[0])==0&&close(status_pipe[0])==0,"owned live-holder observer close");
    printf("private_quarantine_hold_source_known controller=%d helper=%d original_deadline=%llu refusal_received=%llu prompt_before_release=true controller_alive_after_refusal=true original_fd_retained=true child_parent_pin_retained=true owner_lost_recorded=true released_then_controller_own_reap=true group_gone=true exit=81 result_count=1 runtime_approved=false\n",
        controller,info.helper,(unsigned long long)info.deadline,(unsigned long long)received);controls++;
}
struct quantum_child_report {pid_t child;uint64_t user,system;};
static void quantum_burn(void) {
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID);volatile unsigned work=1;
    while(clock_ns(CLOCK_THREAD_CPUTIME_ID)-at<2200000)work=work*33+7;
}
static void *quantum_thread(void *unused) {(void)unused;volatile unsigned work=1;for(;;)work=work*33+7;return NULL;}
static struct image quantum_stopped(pid_t pid) {
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000;struct image current;
    for(;;) {
        require(control_observe(pid,&current),"quantum fixture original task remains observable");
        if(current.status==SSTOP&&current.runnable==0)return current;
        require(clock_ns(CLOCK_MONOTONIC)<until,"quantum fixture original stopped readiness");
        if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)
            require(private_end_unit(private_cost.open_cpu),"quantum fixture bounded readiness observation is paid");
        usleep(100);
    }
}
static void quantum_read(int fd,void *bytes,size_t len) {
    struct pollfd pending={fd,POLLIN,0};require(poll(&pending,1,100)==1,"quantum fixture bounded owned pipe readiness");
    require(read(fd,bytes,len)==(ssize_t)len,"quantum fixture exact atomic owned pipe record");
}
static struct window_ledger quantum_projection(const struct engine_owner *owner,unsigned slots) {
    // Deliberately small guard projection around real retained processes. It
    // isolates eligibility/coverage from scheduler-dependent CPU quantities;
    // it is never an admitted command or a fabricated native measurement.
    struct window_ledger g={.operation=1,.sequence=1,.t0=100,.t_close=300,.cpu_start=1,.cpu_close=2,
        .end={200,250},.actual_cpu=40,.slots=slots,.ack_mask=(1u<<slots)-1,.identity_mask=(1u<<slots)-1,
        .roster_closed=1,.source_complete=1,.baseline_mask=(1u<<slots)-1,.counter_mask=(1u<<slots)-1,
        .post_ack_mask=(1u<<slots)-1,.counter_identity_mask=(1u<<slots)-1,.conversion_mask=(1u<<slots)-1,
        .timebase_numer=1,.timebase_denom=1};
    g.slot_cpu[0]=slots==1?40:20;g.slot_cpu[1]=slots==2?20:0;
    for(unsigned i=0;i<slots;i++) {
        g.baseline_read[i]=90;g.counter_read[i]=290;g.baseline_ticks[i]=10;g.endpoint_ticks[i]=10+g.slot_cpu[i];
        g.baseline_version[i]=g.counter_version[i]=owner->members[i].token.val[7];
        g.counter_kind[i]=COUNTER_SOURCE_POST_ACK_RI;
    }
    return g;
}
static void quantum_native_refusal_cleanup(struct engine_owner *owner,unsigned mode,pid_t pid,
    uint64_t native,uint64_t native_wall,uint64_t inclusive,int event,int gate) {
    // The positive fork return is this fixture's still-unreaped original
    // child. It has never received CONT, so this branch cannot own a forked
    // descendant. No missing/reaped ownership authorizes a numeric PID signal.
    fprintf(stderr,"private_quantum_native_refusal_source_known mode=%u original_pid=%d native_cpu_ns=%llu native_wall_ns=%llu inclusive_cpu_ns=%llu non_native_cpu_ns=%llu native_cpu_limit_ns=2000000 native_wall_limit_ns=25000000 inclusive_limit_ns=3000000 non_native_limit_ns=1000000 original_CONT_count=0 debt_charged_ns=%llu debt_paid_ns=%llu runtime_approved=false\n",
        mode,pid,(unsigned long long)native,(unsigned long long)native_wall,(unsigned long long)inclusive,
        (unsigned long long)(inclusive>=native?inclusive-native:UINT64_MAX),
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    (void)fflush(stderr);uint64_t used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
    (void)engine_charge(owner,used,used,1);
    owner->ticket.pid=owner->ticket.pgid=pid;
    int done=0;errno=0;int own=known_exited(pid,&done),own_error=own?0:errno,signal_attempted=0,signal_result=0,signal_error=0;
    if(own&&!done) {signal_attempted=1;errno=0;signal_result=kill(pid,SIGKILL);signal_error=signal_result?errno:0;}
    if(!own&&own_error==ECHILD){owner->direct_custody=0;retained_group=0;}
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000;
    while(own&&!owner->direct_reaped&&clock_ns(CLOCK_MONOTONIC)<until) {
        if(engine_observe_wait4(owner))break;
        if(!owner->direct_custody)break;
        if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000) {
            used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));(void)engine_charge(owner,used,used,1);
        }
        usleep(100);
    }
    uint64_t helper_cost=UINT64_MAX,cost;used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
    if(owner->wait_observed&&engine_safe_sum(owner->wait_user,owner->wait_system,&helper_cost))
        if(!engine_safe_sum(helper_cost,2000,&helper_cost))helper_cost=UINT64_MAX;
    if(!engine_safe_sum(used,helper_cost,&cost))cost=UINT64_MAX;
    (void)engine_charge(owner,used,cost,1); // Full observed helper cost, never a success/credit claim.
    int pids[3]={0};errno=0;int group=proc_listpgrppids(pid,pids,sizeof(pids)),group_error=group<0?errno:0;
    int event_closed=close(event)==0,gate_closed=close(gate)==0;
    fprintf(stderr,"private_quantum_native_refusal_cleanup_source_known mode=%u original_pid=%d original_WNOWAIT_known=%d original_WNOWAIT_errno=%d original_WNOWAIT_terminal=%d signal_attempted=%d signal_result=%d signal_errno=%d actual_own_wait4=%u status_raw=%d retained_original_group_pid=%d wait4_user_ns=%llu wait4_system_ns=%llu full_helper_cost_ns=%llu group_result=%d group_errno=%d original_event_closed=%d original_gate_closed=%d debt_charged_ns=%llu debt_paid_ns=%llu fixture_cleanup_only=true original_native_guard_failed=true runtime_approved=false\n",
        mode,pid,own,own_error,done,signal_attempted,signal_result,signal_error,owner->wait_observed,owner->retained_status,retained_group,
        (unsigned long long)owner->wait_user,(unsigned long long)owner->wait_system,(unsigned long long)helper_cost,group,group_error,event_closed,gate_closed,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    (void)fflush(stderr);used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));(void)engine_charge(owner,used,used,1);
}
static void quantum_controls(const char *self,int dir,unsigned selected) {
    for(unsigned mode=0;mode<9;mode++) {
        if(selected<9&&selected!=mode)continue;
        reset();outer.deadline=private_cost.deadline;basis.source_coverage=127;basis.command_coverage=65535;
        strcpy(outer.case_name,"positive");require(engine_owner_priority()&&private_end_unit(private_cost.open_cpu),"quantum fixture paced private preparation");
        int event[2],gate[2];require(pipe(event)==0&&pipe(gate)==0,"quantum fixture owned causal pipes");
        uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID),wall=clock_ns(CLOCK_MONOTONIC);pid_t pid=fork();
        if(pid==0) {
            close(event[0]);close(gate[1]);if(setpgid(0,0)!=0)_exit(79);
            if(mode==5){pthread_t thread;if(pthread_create(&thread,NULL,quantum_thread,NULL)!=0)_exit(79);}
            if(raise(SIGSTOP)!=0)_exit(79);
            if(mode==0||mode==2||mode>=6) {
                pid_t child=fork();if(child<0)_exit(79);
                if(child==0){pid_t own=getpid();char byte;if(write(event[1],&own,sizeof(own))!=sizeof(own)||read(gate[0],&byte,1)!=1)_exit(79);quantum_burn();_exit(0);}
                pid_t other=0;
                if(mode==6) {
                    other=fork();if(other<0)_exit(79);
                    if(other==0){pid_t own=getpid();char byte;if(write(event[1],&own,sizeof(own))!=sizeof(own)||read(gate[0],&byte,1)!=1)_exit(79);quantum_burn();_exit(0);}
                }
                struct rusage usage;int status;if(wait4(child,&status,0,&usage)!=child||!WIFEXITED(status)||WEXITSTATUS(status))_exit(79);
                struct quantum_child_report report={child,engine_timeval(&usage.ru_utime),engine_timeval(&usage.ru_stime)};
                if(other>0) {
                    if(wait4(other,&status,0,&usage)!=other||!WIFEXITED(status)||WEXITSTATUS(status))_exit(79);
                    report.user+=engine_timeval(&usage.ru_utime);report.system+=engine_timeval(&usage.ru_stime);
                }
                if(write(event[1],&report,sizeof(report))!=sizeof(report))_exit(79);
            } else if(mode!=4&&mode!=5) {
                char byte='r';if(write(event[1],&byte,1)!=1||read(gate[0],&byte,1)!=1)_exit(79);quantum_burn();
            }
            if(mode!=5&&raise(SIGSTOP)!=0)_exit(79);
            _exit(0);
        }
        require(pid>0,"quantum source-known actual owned process");retained_group=pid;
        uint64_t native_end=clock_ns(CLOCK_THREAD_CPUTIME_ID),native_wall_end=clock_ns(CLOCK_MONOTONIC);
        uint64_t native=engine_interval(at,native_end),native_wall=engine_interval(wall,native_wall_end);
        require(close(event[1])==0&&close(gate[0])==0,"quantum parent closes unowned pipe ends");
        struct private_engine_result result;memset(&result,0,sizeof(result));result.created=1;
        struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&context;owner.result=&result;
        owner.input=owner.output=owner.errors=-1;owner.slots=mode==0||mode==2||mode>=6?2:1;owner.direct_custody=1;owner.closed=1;
        fixture_unit_owner=&owner;
        uint64_t inclusive=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
        fixture_birth=(struct fixture_birth_observation){pid,"quantum",at,native_end,wall,native_wall_end,inclusive,inclusive>=native?inclusive-native:UINT64_MAX};
        (void)engine_charge(&owner,inclusive,inclusive,0);
        if(!(native<=2000000&&native_wall<=25000000&&inclusive<=3000000&&inclusive>=native&&inclusive-native<=1000000))
            quantum_native_refusal_cleanup(&owner,mode,pid,native,native_wall,inclusive,event[0],gate[1]);
        require(native<=2000000&&native_wall<=25000000&&inclusive<=3000000&&inclusive>=native&&inclusive-native<=1000000,
            "quantum fixture unchanged native application and inclusive gates after full cost payment");
        struct image initial=quantum_stopped(pid);owner.ticket=(struct engine_ticket){pid,pid,initial.birth,initial.start_absolute,1};
        owner.members[0].state.pid=pid;owner.members[0].state.birth=initial.birth;strcpy(owner.image,self);
        unsigned thread_rejected=0;int ack=0;
        if(mode==5)thread_rejected=!engine_sample_member(&owner,0,pid,&ack)&&private_errors[2].count;
        else require(engine_sample_member(&owner,0,pid,&ack)&&ack,"quantum original stopped identity and RI baseline");
        uint64_t startup=0,initial_user=0,initial_system=0;
        if(mode!=5)require(engine_actual(&owner,&startup,&initial_user,&initial_system),"actual initial source-known CPU baseline");
        struct quantum_child_report report={0,0,0};pid_t child=0,other=0;unsigned actual_child_count=0;uint64_t pre_exit_rollup_read=0,child_exit_observed=0;
        uint64_t grant_start=0,grant_cpu=0,ack_time=0,read_time=0,close_time=0,close_cpu=0,pre_ack_ticks=0,pre_ack_time=0,post_ticks=0;
        uint64_t post_ack_cpu=0;mach_timebase_info_data_t grant_scale;
        require(mach_timebase_info(&grant_scale)==KERN_SUCCESS&&grant_scale.numer&&grant_scale.denom,"quantum fixture retained pre-CONT timebase");
        uint64_t baseline_ticks=owner.members[0].state.raw_rusage_cpu,baseline_time=owner.members[0].counter_read_ns;
        uint32_t version=owner.members[0].token.val[7];
        require(private_end_unit(private_cost.open_cpu),"quantum initial task counters and pipe setup are bounded paid ordinary work");
        if(mode!=5) {
            int children[3];require(proc_listchildpids(pid,children,sizeof(children))==0,"quantum initial census precedes actual fork");
            grant_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);grant_start=clock_ns(CLOCK_MONOTONIC);
            require(kill(-pid,SIGCONT)==0,"quantum original CONT once for source-known work");
            if(mode==0||mode==2||mode>=6) {
                quantum_read(event[0],&child,sizeof(child));require(child>0,"quantum actual late child identity");
                if(mode==6)quantum_read(event[0],&other,sizeof(other));
                actual_child_count=(unsigned)proc_listchildpids(pid,children,sizeof(children));
                require(actual_child_count==(mode==6?2u:1u)&&(children[0]==child||(mode==6&&children[1]==child)),"actual child appears only after initial census");
                struct image member;require(control_observe(child,&member),"actual late child retained before its release");
                owner.members[1].state.pid=child;owner.members[1].state.birth=member.birth;
                owner.members[1].token=control_token(child,member.birth);owner.members[1].token_known=1;
                struct rusage_info_v1 usage;require(proc_pid_rusage(child,RUSAGE_INFO_V1,(rusage_info_t *)&usage)==0&&engine_usage(&owner,1,&usage),"late child actual raw counters");
                if(mode>=6) {
                    struct rusage_info_v1 early_rollup;
                    require(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&early_rollup)==0,"actual leader rollup read while original child is still live");
                    pre_exit_rollup_read=clock_ns(CLOCK_MONOTONIC);
                }
                const char release[2]={'x','x'};size_t releases=mode==6?2:1;
                require(write(gate[1],release,releases)==(ssize_t)releases,"quantum fixture releases its existing late child");quantum_read(event[0],&report,sizeof(report));
                require((report.child==child||(mode==6&&report.child==other))&&report.user+report.system>=2000000,"actual leader wait4 records at least two milliseconds descendant CPU");
                if(mode>=6) {
                    require(proc_listchildpids(pid,children,sizeof(children))==0,"actual original child exit precedes fresh count-zero rollup read");
                    child_exit_observed=clock_ns(CLOCK_MONOTONIC);
                }
            } else if(mode!=4) {
                char byte;quantum_read(event[0],&byte,1);struct image early;
                require(control_observe(pid,&early)&&early.status!=SSTOP,"pre-ACK RI sample is physically before stopped acknowledgement");
                struct rusage_info_v1 early_usage;require(proc_pid_rusage(pid,RUSAGE_INFO_V1,(rusage_info_t *)&early_usage)==0,"actual pre-ACK RI counter read");
                pre_ack_ticks=early_usage.ri_user_time+early_usage.ri_system_time;pre_ack_time=clock_ns(CLOCK_MONOTONIC);
                require(write(gate[1],"x",1)==1,"quantum fixture releases actual CPU after pre-ACK observation");
            }
            if(mode==4)usleep(30000); // Actual ACK observation is delayed, never backdated.
            (void)quantum_stopped(pid);
            require(engine_sample_member(&owner,0,pid,&ack)&&ack,"actual source-known positive current-task ACK");
            ack_time=clock_ns(CLOCK_MONOTONIC);
            require(engine_sample_member(&owner,0,pid,&ack)&&ack,"actual source-known RI read after positive ACK");
            read_time=owner.members[0].counter_read_ns;post_ticks=owner.members[0].state.raw_rusage_cpu;
            require(post_ticks>=baseline_ticks&&engine_scaled_ticks(post_ticks-baseline_ticks,grant_scale.numer,grant_scale.denom,1,&post_ack_cpu),"actual stopped helper interval is converted before closure clock");
            close_time=clock_ns(CLOCK_MONOTONIC);close_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);
            if(mode==0) {
                owner.grant.t0=grant_start;owner.grant.source_complete=1;owner.grant.unknown_mask=2;int closed=0;
                require(!engine_roster(&owner,1,&closed)||!closed||owner.grant.unknown_mask,
                    "actual late child cannot disappear into a measured closed roster");
            }
            require(private_end_unit(private_cost.open_cpu),"quantum stopped observation and counter snapshots are bounded paid work");
            require(kill(pid,SIGKILL)==0,"quantum fixture exact owned terminal cleanup after refused work");
        } else require(kill(pid,SIGKILL)==0,"quantum thread-negative exact owned process cleanup");
        uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000;int done=0;
        while(!done){require(known_exited(pid,&done),"quantum original WNOWAIT ownership");require(clock_ns(CLOCK_MONOTONIC)<until,"quantum fixture actual exit bound");if(!done)usleep(100);}
        if(mode!=5)require(engine_member_exit(&owner,0),"quantum retained original final RI counter source");
        require(engine_observe_wait4(&owner)&&owner.direct_reaped,"quantum controller owns actual original wait4");
        require(close(event[0])==0&&close(gate[1])==0,"quantum fixture original pipe cleanup");
        int pids[3];require(proc_listpgrppids(pid,pids,sizeof(pids))==0,"quantum actual owned group gone");
        uint64_t actual_wait_cpu;
        require(engine_safe_sum(owner.wait_user,owner.wait_system,&actual_wait_cpu)&&engine_safe_sum(actual_wait_cpu,2000,&actual_wait_cpu),"quantum actual wait4 upper cost is retained");
        // These are deliberately invalid/over-budget fixture executions, not
        // admitted commands. Pay ALL actually observed helper CPU once even
        // when the deliberately incomplete certification ledger below refuses.
        uint64_t observed_controller=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID)),actual_cost;
        require(engine_safe_sum(observed_controller,actual_wait_cpu,&actual_cost),"quantum observed helper and controller checked cost");
        uint64_t cost_unit=private_cost.closed_units;
        (void)engine_charge(&owner,observed_controller,actual_cost,1);
        engine_residual_payment_retained(&owner,actual_wait_cpu,cost_unit,actual_cost);
        struct window_ledger g=quantum_projection(&owner,mode==0||mode==5||mode>=6?2:1);
        enum ledger_result verdict;
        if(mode==0) {
            g.unknown_mask=2;g.end[1]=g.counter_read[1]=g.t_close;verdict=window_charge(&g);
            require(!g.measured,"unknown late-child slot never obtains measured basis");
        } else if(mode==1) {
            mach_timebase_info_data_t scale;require(mach_timebase_info(&scale)==KERN_SUCCESS,"pre-ACK source-known timebase");
            g.t0=grant_start;g.end[0]=ack_time;g.t_close=clock_ns(CLOCK_MONOTONIC);g.cpu_start=g.cpu_close=grant_cpu;
            g.baseline_ticks[0]=baseline_ticks;g.endpoint_ticks[0]=pre_ack_ticks;g.baseline_read[0]=baseline_time;
            g.counter_read[0]=pre_ack_time;g.baseline_version[0]=g.counter_version[0]=version;
            g.timebase_numer=scale.numer;g.timebase_denom=scale.denom;
            require(pre_ack_ticks>=baseline_ticks&&engine_scaled_ticks(pre_ack_ticks-baseline_ticks,scale.numer,scale.denom,1,&g.actual_cpu),"pre-ACK observed CPU delta conversion");
            g.slot_cpu[0]=g.actual_cpu;g.post_ack_mask=0;verdict=window_charge(&g);
            require(!g.measured,"counter read before positive ACK never obtains measured basis");
            owner.startup_cpu=startup;owner.startup_known=owner.conservation_known=1;owner.charged_member_cpu=g.actual_cpu;
            require(!engine_conservation(&owner)&&owner.conservation_excess>0&&private_errors[5].count,
                "actual wait4 catches CPU burned after the deliberately early counter read");
        } else if(mode==4) {
            g.t0=grant_start;g.end[0]=ack_time;g.t_close=close_time;g.cpu_start=grant_cpu;g.cpu_close=close_cpu;
            g.baseline_ticks[0]=baseline_ticks;g.endpoint_ticks[0]=post_ticks;g.baseline_read[0]=baseline_time;g.counter_read[0]=read_time;
            g.baseline_version[0]=g.counter_version[0]=version;g.timebase_numer=grant_scale.numer;g.timebase_denom=grant_scale.denom;
            g.actual_cpu=post_ack_cpu;
            g.slot_cpu[0]=g.actual_cpu;
            verdict=window_charge(&g);
            require(verdict==LEDGER_LIMIT&&g.measured&&g.charge<1000000&&g.windows>25000000,
                "low measured CPU cannot waive actual delayed ACK twenty-five millisecond wall gate");
        } else if(mode==5) {
            g.end[0]=110;g.actual_cpu=40;verdict=window_charge(&g);
            require(thread_rejected&&verdict==LEDGER_COVERAGE,"actual second thread and per-slot CPU over wall both refuse coverage");
        } else if(mode>=6) {
            // The native processes and both read stages above are real. This
            // finite guard projection deliberately supplies the named bad
            // provenance, never a fabricated measured command receipt.
            g.rollup_mask=2;g.post_ack_mask=1;g.counter_kind[1]=COUNTER_SOURCE_LEADER_ROLLUP;
            g.rollup_unique=actual_child_count==1;g.rollup_child_retired=1;
            g.rollup_child_exit_ns=260;g.rollup_child_zero_ns=270;g.rollup_leader_ack_ns=250;g.rollup_read_ns=g.counter_read[1]=290;
            g.rollup_child_cpu_ns=g.slot_cpu[1];
            require(pre_exit_rollup_read&&child_exit_observed>pre_exit_rollup_read&&read_time>=child_exit_observed,
                "actual rollup provenance retains distinct pre-exit and post-ACK read stages");
            if(mode==7)g.rollup_read_ns=g.counter_read[1]=255;
            if(mode==8)g.live_child_cpu_ns=g.slot_cpu[1];
            verdict=window_charge(&g);
            if(mode==6)require(!g.measured,"mixed second child rollup cannot certify measured basis");
            if(mode==7)require(!g.measured,"rollup read before original child exit cannot acquire later proof");
            if(mode==8) {
                require(verdict==LEDGER_COVERAGE,"live child and post-exit leader rollup cannot both be charged");
                g.live_child_cpu_ns=0;g.t0=100;g.end[0]=1200100;g.end[1]=1000100;g.t_close=1300100;
                g.counter_read[0]=g.counter_read[1]=g.rollup_read_ns=1300000;
                g.rollup_child_exit_ns=1000100;g.rollup_child_zero_ns=1100100;g.rollup_leader_ack_ns=1200100;
                // The finite timing projection is not a native receipt, but
                // terminal conservation below uses the actual retained Mach
                // scale. Do not make that check fail earlier on the fixture's
                // synthetic1/1 scale and mistake it for numeric undercharge.
                g.timebase_numer=grant_scale.numer;g.timebase_denom=grant_scale.denom;
                require(engine_scaled_ticks(1,g.timebase_numer,g.timebase_denom,1,&g.slot_cpu[0]),"provisional projection uses the actual retained terminal scale");
                g.slot_cpu[1]=g.rollup_child_cpu_ns=g.slot_cpu[0];g.actual_cpu=g.slot_cpu[0]+g.slot_cpu[1];
                g.endpoint_ticks[0]=g.baseline_ticks[0]+1;g.endpoint_ticks[1]=g.baseline_ticks[1]+1;
                require(window_charge(&g)==LEDGER_OK&&g.measured,"qualified source remains provisional before actual terminal conservation");
                owner.grant=g;require(engine_provisional_add(&owner)&&owner.provisional_count==1&&owner.provisional_rest_difference>0,"bounded provisional full-window fallback retained without rewriting original receipt");
                owner.startup_known=owner.conservation_known=1;owner.startup_cpu=owner.charged_member_cpu=0;
                uint64_t debt=private_cost.debt,booked=private_cost.booked_adjustments;
                source_known_schedule_interrupts=1;
                int conserved=engine_conservation(&owner);
                fprintf(stderr,"private_rollup_revocation_source_known pid=%d child=%d actual_wait4=%u conserved=%d excess_ns=%llu original_scale=%u/%u revoked=%u retained_adjustments=%llu retained_adjustment_ns=%llu expected_rest_difference_ns=%llu debt_before_ns=%llu debt_after_ns=%llu original_measured=%u runtime_approved=false\n",
                    pid,child,owner.wait_observed,conserved,(unsigned long long)owner.conservation_excess,g.timebase_numer,g.timebase_denom,owner.provisional_revoked,
                    (unsigned long long)private_cost.retained_adjustments,(unsigned long long)private_cost.retained_adjustment_cost_ns,
                    (unsigned long long)owner.provisional_rest_difference,(unsigned long long)debt,(unsigned long long)private_cost.debt,owner.grant.measured);
                require(!conserved&&owner.provisional_revoked&&owner.conservation_excess>0
                    &&private_cost.retained_adjustments==1&&private_cost.retained_adjustment_cost_ns==owner.provisional_rest_difference
                    &&private_cost.debt==debt,"terminal mismatch retains once-only fallback debt even when scheduling fails");
                require(private_pay_pending()&&!private_has_retained_cost()&&private_cost.debt_paid==private_cost.debt
                    &&private_cost.debt==debt+owner.provisional_debt_added&&private_cost.booked_adjustments==booked+1,
                    "revoked effective window pays exact normalized rest difference with no second minimum floor");
                debt=private_cost.debt;require(!engine_conservation(&owner)&&private_cost.debt==debt&&owner.grant.measured,
                    "terminal revocation is idempotent and preserves original measured receipt");
            }
        } else {
            uint64_t total,user,system;
            require(engine_actual(&owner,&total,&user,&system)&&total>=startup,"actual nonduplicated terminal RI interval");
            owner.startup_cpu=startup;owner.startup_known=owner.conservation_known=1;
            owner.charged_member_cpu=total-startup;
            // No wait4-derived residual may fund the SAME wait4's RHS. The
            // independent RI snapshot is retained; completeness is tested by
            // conservation, never assumed as a fixture prerequisite.
            int conserved=engine_conservation(&owner);
            if(mode==2) {
                owner.child_created=1;unsigned direct_reaps=0,qualified_reaps=0;
                require(engine_reaped_for_teardown(&owner,&direct_reaps,&qualified_reaps)==1&&direct_reaps==1&&!qualified_reaps,
                    "unproven child cannot mint exit-teardown process credit");
                // The live RI snapshot need not contain every later exit
                // kernel increment. This witness tests actual descendant
                // inclusion, not an unsupported complete-RI premise.
                uint64_t self=owner.members[0].state.rusage_cpu,child_ns=UINT64_MAX;
                require(engine_safe_sum(report.user,report.system,&child_ns)&&child_ns>=2000000,
                    "actual leader-owned child wait4 supplies the independent numeric control input");
                __uint128_t scaled=(__uint128_t)child_ns*grant_scale.denom/grant_scale.numer;
                require(scaled<=UINT64_MAX,"source-known rollup arithmetic projection conversion bound");
                // Explicit arithmetic projection from the leader's ACTUAL
                // child wait4, not a forged leader RI observation. It changes
                // neither the original owner/receipts nor conservation credit.
                struct engine_owner projection=owner;
                projection.members[0].state.raw_waited=(uint64_t)scaled;
                uint64_t expected_raw=UINT64_MAX,actual_raw=UINT64_MAX,projected_user=UINT64_MAX,projected_system=UINT64_MAX;
                uint64_t live=projection.members[1].state.raw_rusage_cpu;
                uint64_t rolled=projection.members[0].state.raw_waited;
                require(rolled>live&&engine_safe_sum(projection.members[0].state.raw_rusage_cpu,rolled,&expected_raw)
                    &&engine_actual_ticks(&projection,&actual_raw,&projected_user,&projected_system)&&actual_raw==expected_raw,
                    "actual reaped-descendant rollup is included in terminal charged conservation");
                fprintf(stderr,"private_rollup_arithmetic_projection_source_known pid=%d child=%d actual_child_wait4_ns=%llu observed_RI_rollup_ticks=%llu projected_rollup_ticks=%llu observed_live_ticks=%llu projected_total_ticks=%llu arithmetic_projection_only=true counter_source_approval=false runtime_approved=false\n",
                    pid,child,(unsigned long long)child_ns,(unsigned long long)owner.members[0].state.raw_waited,
                    (unsigned long long)rolled,(unsigned long long)live,(unsigned long long)actual_raw);
                uint64_t target=owner.wait4_upper>startup?owner.wait4_upper-startup:0;
                require(target<=UINT64_MAX-1000000,"controller-post exclusion finite target arithmetic");
                target+=1000000;if(target<3000000)target=3000000;
                unsigned unit=0;
                for(;unit<64&&(!result.post_cpu_ns.observed||result.post_cpu_ns.value<target);unit++) {
                    uint64_t start=clock_ns(CLOCK_THREAD_CPUTIME_ID);
                    while(clock_ns(CLOCK_THREAD_CPUTIME_ID)-start<250000){}
                    uint64_t used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
                    require(engine_charge(&owner,used,used,1),"controller-only post units retain unchanged one-millisecond gate and fresh paid debt");
                    engine_sum(&result.post_cpu_ns,used);
                }
                require(result.post_cpu_ns.value>=target,"large actual controller post work was separately paid");
                fprintf(stderr,"private_controller_post_isolation_source_known pid=%d child=%d original_conserved=%d original_excess_ns=%llu independent_rollup_ns=%llu omitted_helper_upper_ns=%llu paid_controller_post_ns=%llu bounded_units=%u runtime_approved=false\n",
                    pid,child,conserved,(unsigned long long)owner.conservation_excess,(unsigned long long)(total-self),
                    (unsigned long long)owner.wait4_upper,(unsigned long long)result.post_cpu_ns.value,unit);
                owner.charged_member_cpu=0;owner.conservation_checked=0;
                require(!engine_conservation(&owner)&&owner.conservation_excess>0,
                    "already-paid controller post CPU cannot cover omitted actual helper rollup");
                // Explicit finite API projection after the ACTUAL original
                // leader/child execution and own wait4 above. These supplied
                // source-proof premises are not a real Git qualification.
                struct engine_owner physical=owner;struct private_engine_result physical_result=result;
                struct private_git_sig_temp physical_sig;memset(&physical_sig,0,sizeof(physical_sig));
                physical.result=&physical_result;physical.command=12;physical.verified_descendants=physical.child_created=1;
                physical.members[1].phase=ENGINE_EXITED;physical.members[1].exit_abstime=1;
                physical_sig.bound=1;physical_sig.pid=child;physical_sig.pidversion=physical.members[1].token.val[7];physical.git_signature=&physical_sig;
                physical.child_reap_proof=(struct engine_child_reap_proof){.pid=child,.version=physical.members[1].token.val[7],.valid=1,
                    .birth=physical.members[1].state.birth,.start=physical.members[1].state.start_absolute,
                    .exit_ns=1,.zero_ns=2,.ack_ns=3,.read_ns=4,.rollup_ticks=physical.members[0].state.raw_waited};
                physical.provisional_count=1;physical.provisional_finalized=physical.provisional_revoked=0;
                physical.provisional_rest_difference=0;
                require(engine_reaped_for_teardown(&physical,&direct_reaps,&qualified_reaps)==2&&qualified_reaps==1,
                    "finite qualified physical child proof supplies one extra fixed ceiling");
                engine_quarantine_provisional(&physical);
                require(!physical.rollup_source_failed&&physical.provisional_revoked
                    &&engine_reaped_for_teardown(&physical,&direct_reaps,&qualified_reaps)==2&&qualified_reaps==1,
                    "accounting quarantine revokes rebates without erasing independent child reap proof");
                physical.rollup_source_failed=1;
                require(engine_reaped_for_teardown(&physical,&direct_reaps,&qualified_reaps)==1&&!qualified_reaps,
                    "actual source failure excludes child fixed ceiling despite retained old proof");
                fprintf(stderr,"private_quarantine_reap_guard_projection_source_known original_leader=%d original_child=%d actual_leader_wait4=true physical_premises_finite_projection=true accounting_quarantine_keeps_child_count=true source_failure_drops_child_count=true real_Git_qualification=false runtime_approved=false\n",pid,child);
            }
            if(mode==3) {
                mach_timebase_info_data_t scale;uint64_t raw=owner.members[0].state.raw_rusage_cpu,converted=UINT64_MAX;
                require(mach_timebase_info(&scale)==KERN_SUCCESS&&engine_ticks_ns(raw,&converted),"actual Mach conversion and retained platform scale");
                __uint128_t exact=(__uint128_t)raw*scale.numer;exact=exact/scale.denom+(exact%scale.denom!=0);
                require(converted==exact,"actual Mach tick interval uses explicit retained timebase");
                fprintf(stderr,"private_timebase_isolation_source_known pid=%d actual_ri_ticks=%llu converted_ns=%llu scale=%u/%u terminal_conserved=%d terminal_excess_ns=%llu complete_RI_assumed=false runtime_approved=false\n",
                    pid,(unsigned long long)raw,(unsigned long long)converted,scale.numer,scale.denom,conserved,(unsigned long long)owner.conservation_excess);
            }
            verdict=window_charge(&g);
        }
        char name[32];snprintf(name,sizeof(name),"quantum-%u.ndjson",mode);
        require(private_end_unit(private_cost.open_cpu),"quantum guard projection and conservation classification are separately paid");
        owner.ledger=openat(dir,name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);owner.grant=g;
        require(owner.ledger>=0&&engine_record(&owner,"source_known_quantum")&&fsync(owner.ledger)==0&&close(owner.ledger)==0,"quantum bounded owned raw evidence persistence");
        require(private_end_unit(private_cost.open_cpu),"quantum final owned formatter persistence and disposal are paid ordinary work");
        printf("private_quantum_source_known mode=%u pid=%d child=%d other_child=%d observed_child_count=%u pre_exit_rollup_read_ns=%llu child_exit_observed_ns=%llu post_ACK_read_ns=%llu actual_wait4=true group_gone=true guard_projection_only=%s qualification=false basis=%s W=%llu A=%llu C=%llu Q=%llu verdict=%d conservation_checked=%u conservation_passed=%u excess_ns=%llu native_cpu_ns=%llu native_wall_ns=%llu runtime_approved=false\n",
            mode,pid,child,other,actual_child_count,(unsigned long long)pre_exit_rollup_read,(unsigned long long)child_exit_observed,(unsigned long long)read_time,mode==4?"false":"true",g.measured?"measured":"window",(unsigned long long)g.windows,(unsigned long long)g.actual_cpu,(unsigned long long)g.controller,(unsigned long long)g.charge,verdict,
            owner.conservation_checked,owner.conservation_ok,(unsigned long long)owner.conservation_excess,(unsigned long long)native,(unsigned long long)native_wall);controls++;
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"quantum final stdout formatting and actual flush are paid before the next fixture epoch");
    }
    reset();
}
static void exit_teardown_checkpoint(struct engine_owner *owner,const char *stage) {
    // Deliberate helper over-budget/refusal is sticky, but it does not make a
    // subsequent ordinary unit fail automatically. Preserve that unit's OWN
    // clocks/cost/payment verdict, including the inclusive bookkeeping tail.
    uint64_t used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
    int paid=engine_charge_at(owner,used,used,1,stage,__LINE__);
    require(paid,"exit teardown disjoint ordinary stage keeps its own known one-millisecond cost and paid debt");
}
static void exit_teardown_control(const char *self,int dir,unsigned selected) {
    reset();outer.deadline=private_cost.deadline;basis.source_coverage=127;basis.command_coverage=65535;
    require(engine_owner_priority()&&private_end_unit(private_cost.open_cpu),"exit teardown fixture paced preparation");
    uint64_t at=clock_ns(CLOCK_THREAD_CPUTIME_ID),wall=clock_ns(CLOCK_MONOTONIC);pid_t pid=fork();
    if(pid==0) {
        if(setpgid(0,0)!=0||raise(SIGSTOP)!=0)_exit(79);
        quantum_burn();_exit(0);
    }
    require(pid>0,"exit teardown actual original owned child");retained_group=pid;
    uint64_t native_end=clock_ns(CLOCK_THREAD_CPUTIME_ID),native_wall_end=clock_ns(CLOCK_MONOTONIC);
    uint64_t native=engine_interval(at,native_end),native_wall=engine_interval(wall,native_wall_end);
    struct private_engine_result result;struct engine_owner owner;memset(&result,0,sizeof(result));memset(&owner,0,sizeof(owner));
    owner.context=&context;owner.result=&result;owner.input=owner.output=owner.errors=owner.ledger=-1;
    owner.direct_custody=1;owner.slots=1;owner.closed=1;result.created=1;
    fixture_unit_owner=&owner;
    uint64_t inclusive=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
    fixture_birth=(struct fixture_birth_observation){pid,"exit_teardown",at,native_end,wall,native_wall_end,inclusive,inclusive>=native?inclusive-native:UINT64_MAX};
    (void)engine_charge(&owner,inclusive,inclusive,0);
    require(native<=2000000&&native_wall<=25000000&&inclusive<=3000000&&inclusive>=native&&inclusive-native<=1000000,
        "exit teardown unchanged native application and inclusive limits after full payment");
    struct image first=quantum_stopped(pid);owner.ticket=(struct engine_ticket){pid,pid,first.birth,first.start_absolute,1};
    owner.members[0].state.pid=pid;owner.members[0].state.birth=first.birth;strcpy(owner.image,self);int ack=0;
    require(engine_sample_member(&owner,0,pid,&ack)&&ack,"exit teardown retained last original stopped identity and RI sample");
    owner.ticket.start=owner.members[0].state.start_absolute;
    require(private_end_unit(private_cost.open_cpu),"exit teardown stopped identity observation is separately charged before helper prefix billing");
    // Reuse the production already-stopped census/member checkpoints and its
    // isolated final raw prefix charge. No grant exists yet, no old endpoint
    // moves, and every observed C/startup increment remains charged once.
    require(engine_grant_baseline(&owner)&&owner.startup_known&&owner.startup_charged,
        "exit teardown production stopped stages and independent prefix are paid before actual hidden CPU");
    // Deliberately omit every later helper sample. The actual child burns
    // >2ms AFTER this last retained observation, then exits; no residual is
    // allowed to turn into independent credit or a data-dependent ceiling.
    owner.grants=1;require(kill(-pid,SIGCONT)==0,"exit teardown one original fixture CONT");
    uint64_t until=clock_ns(CLOCK_MONOTONIC)+100000000;int done=0;
    while(!done) {
        require(known_exited(pid,&done),"exit teardown original own-WNOWAIT ownership");
        require(clock_ns(CLOCK_MONOTONIC)<until,"exit teardown bounded actual child exit");
        if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)
            require(engine_charge(&owner,engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID)),
                engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID)),1),"exit teardown bounded controller observations are independently paid");
        if(!done)usleep(100);
    }
    require(engine_observe_wait4(&owner)&&owner.direct_reaped,"exit teardown original own actual wait4");
    int pids[3]={0};require(proc_listpgrppids(pid,pids,sizeof(pids))==0,"exit teardown original actual group gone before every causal assertion");
    require(engine_residual_obligation(&owner,0)&&owner.residual_required_cost>ENGINE_EXIT_TEARDOWN_CEILING_NS,
        "exit teardown actual post-sample CPU exceeds the fixed independent ceiling");
    unsigned missing_debit_refused=!engine_residual_cost_covers(&owner);
    uint64_t required=owner.residual_required_cost,used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID)),cost;
    require(engine_safe_sum(used,required,&cost),"exit teardown full actual residual and controller checked cost");
    uint64_t booked=private_cost.closed_units;(void)engine_charge(&owner,used,cost,1);
    engine_residual_payment_retained(&owner,required,booked,cost);
    require(private_cost.debt==private_cost.debt_paid&&!private_has_retained_cost()&&owner.residual_retained_cost>=required,
        "exit teardown over-budget residual is fully retained and paid without helper-credit relabeling");
    int conserved=engine_conservation(&owner);
    exit_teardown_checkpoint(&owner,"teardown_conservation_classification");
    owner.ledger=openat(dir,"exit-teardown.ndjson",O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
    require(owner.ledger>=0&&engine_record(&owner,"exit_teardown_source_known"),
        "exit teardown original raw evidence and per-process cost provenance written");
    exit_teardown_checkpoint(&owner,"teardown_raw_record_format_and_write");
    require(fsync(owner.ledger)==0&&close(owner.ledger)==0,"exit teardown original raw file persistence and close");
    exit_teardown_checkpoint(&owner,"teardown_raw_fsync_and_disposal");
    fprintf(stderr,"private_exit_teardown_source_known pid=%d birth=%llu actual_own_wait4=true group_gone=true independent_prefix_ns=%llu raw_wait4_cpu_ns=%llu residual_required_cost_ns=%llu residual_retained_cost_ns=%llu residual_cost_paid=%u missing_debit_refused=%u T_ns=%llu N_direct=%u N_child=%u N_times_T_ns=%llu excess_ns=%llu conserved=%d debt_charged_ns=%llu debt_paid_ns=%llu runtime_approved=false\n",
        pid,(unsigned long long)owner.ticket.birth,(unsigned long long)owner.startup_cpu,(unsigned long long)owner.raw_wait4_cpu,
        (unsigned long long)owner.residual_required_cost,(unsigned long long)owner.residual_retained_cost,owner.residual_cost_paid,missing_debit_refused,
        (unsigned long long)owner.exit_teardown_limit_ns,owner.teardown_direct_reaps,owner.teardown_child_reaps,
        (unsigned long long)owner.exit_teardown_allowance_ns,(unsigned long long)owner.conservation_excess,conserved,
        (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
    require(fflush(stderr)==0,"exit teardown final diagnostic output flush");
    exit_teardown_checkpoint(&owner,"teardown_final_diagnostic_output");
    if(selected!=1)require(!conserved&&owner.conservation_excess>0&&owner.residual_cost_paid,
        "fixed exit teardown ceiling rejects actual CPU burned after the last sample");
    if(selected!=0)require(missing_debit_refused,
        "terminal residual without its actual retained paid debit is refused");
    controls++;reset();
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
static void initial_stopped_controls(const char *self,int dir,unsigned selected) {
    for(unsigned mode=0;mode<4;mode++) {
        if(selected<4&&selected!=mode)continue;
        reset();outer.deadline=private_cost.deadline;basis.source_coverage=127;basis.command_coverage=65535;
        require(engine_owner_priority(),"initial stopped fixture owner priority");
        struct private_engine_result result;struct engine_owner owner;memset(&result,0,sizeof(result));memset(&owner,0,sizeof(owner));
        owner.context=&context;owner.result=&result;owner.slots=1;owner.input=owner.output=owner.errors=-1;owner.source_admitted=1; // Source-known premise, never candidate authentication.
        result.phase="git_reads";result.outcome="refused";result.observed_proof="not_reached";
        const char *names[]={"stopped-stages.ndjson","startup-post-exit.ndjson","startup-unavailable.ndjson","startup-unpaid.ndjson"};
        const char *name=names[mode];
        owner.ledger=openat(dir,name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        posix_spawnattr_t attr;posix_spawn_file_actions_t actions;
        require(owner.ledger>=0&&posix_spawnattr_init(&attr)==0&&posix_spawn_file_actions_init(&actions)==0
            &&posix_spawnattr_setflags(&attr,POSIX_SPAWN_START_SUSPENDED|POSIX_SPAWN_SETPGROUP|POSIX_SPAWN_CLOEXEC_DEFAULT)==0
            &&posix_spawnattr_setpgroup(&attr,0)==0&&posix_spawn_file_actions_addopen(&actions,0,"/dev/null",O_RDONLY,0)==0
            &&posix_spawn_file_actions_adddup2(&actions,1,1)==0&&posix_spawn_file_actions_adddup2(&actions,2,2)==0,
            "initial stopped bounded actual suspended-effect preparation");
        char *args[]={(char *)self,"exit-child",NULL};char *env[]={"PATH=/usr/bin:/bin","LC_ALL=C","LANG=C",NULL};
        require(private_end_unit(private_cost.open_cpu),"initial stopped preparation is separately paid before native effect");
        struct prepared_spawn_outcome raw;prepared_spawn(self,args,env,&actions,&attr,1,&raw);owner.native=raw;
        if(raw.pid>0){owner.ticket.pid=owner.ticket.pgid=raw.pid;owner.direct_custody=1;result.created=1;}
        int actions_disposed=posix_spawn_file_actions_destroy(&actions),attr_disposed=posix_spawnattr_destroy(&attr);
        uint64_t used=engine_interval(private_cost.open_cpu,clock_ns(CLOCK_THREAD_CPUTIME_ID));
        int paid=engine_charge(&owner,used,used,0);used=private_cost.last_unit;
        uint64_t app=used!=UINT64_MAX&&raw.cpu!=UINT64_MAX&&used>=raw.cpu?used-raw.cpu:UINT64_MAX;
        if(!paid||raw.pid<=0||raw.error||!raw.cpu_known||!raw.wall_known||actions_disposed||attr_disposed
            ||raw.cpu>2000000||raw.wall>25000000||used>3000000||app>1000000)
            fixture_setup_failure(raw.pid,"initial suspended fixture native/inclusive allocation refused",private_cost.open_cpu,used);
        struct image first;
        require(control_observe(raw.pid,&first)&&first.ppid==getpid()&&first.pgid==raw.pid,"initial stopped original positive effect identity");
        owner.ticket.birth=first.birth;owner.ticket.sequence=1;owner.members[0].state.pid=raw.pid;owner.members[0].state.birth=first.birth;strcpy(owner.image,self);
        int ack=0;uint64_t until=raw.wall_end+25000000;
        do {
            require(engine_sample_member(&owner,0,raw.pid,&ack),"initial suspended actual current-token source observation");
            if(ack)break;
            require(clock_ns(CLOCK_MONOTONIC)<until,"initial suspended unchanged bounded readiness");
            if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)require(private_end_unit(private_cost.open_cpu),"initial suspended readiness ordinary observation paid");
            usleep(100);
        } while(!ack);
        require(owner.members[0].user==0,"actual START_SUSPENDED fixture has no user code or CONT");
        owner.ticket.start=owner.members[0].state.start_absolute;owner.closed=1;
        require(private_end_unit(private_cost.open_cpu),"initial stopped identity attachment separately paid before corroboration");
        source_known_stopped_stage_cpu=mode?0:150000;
        int corroborated=engine_stopped_corroboration(&owner);source_known_stopped_stage_cpu=0;
        unsigned stages=owner.stopped_units;uint64_t max_stage=owner.stopped_max_cpu;
        (void)engine_failure(&owner,"cancelled","source-known cleanup of never-continued initial stopped effect");
        source_known_initial_ri_unavailable=mode>=2;source_known_initial_payment_interrupt=mode==3;
        int status=-1;(void)engine_retire(&owner,&status);
        unsigned first_reaped=owner.direct_reaped,first_quarantined=owner.settlement_quarantined;
        unsigned first_attempt_paid=owner.post_exit_attempt_paid;
        if(owner.settlement_quarantined&&owner.post_exit_initial&&!owner.post_exit_initial_source_valid&&!owner.post_exit_ri_attempted) {
            // This fixture's own positive spawn return is STILL our unreaped
            // child; no other waiter exists. Preserve the failed product
            // predicates, then do separately labelled API-fixture cleanup.
            // Do not turn a known-unresolvable source predicate into a45s
            // product-hold timeout, or claim that this cleanup certified it.
            const struct engine_initial_physical_observation *p=&owner.initial_physical;
            fprintf(stderr,"private_initial_physical_refusal_source_known mode=%u pid=%d birth=%llu stage=%u predicate_mask=%u returned_BSD_required_mask=255 group_result=%d group_errno=%d child_result=%d child_errno=%d audit_result=%d audit_errno=%d audit_path_expected=%u bsd_known=%u bsd_pid=%u bsd_parent=%u bsd_pgid=%u bsd_status=%u bsd_uid=%u bsd_birth=%llu original_product_quarantined=true runtime_approved=false\n",
                mode,raw.pid,(unsigned long long)owner.ticket.birth,p->stage,p->predicate_mask,p->group_result,p->group_error,p->child_result,p->child_error,
                p->audit_result,p->audit_error,p->audit_path_expected,p->bsd_known,p->pid,p->ppid,p->pgid,p->status,p->uid,(unsigned long long)p->birth);
            (void)fsync(owner.ledger);(void)private_end_unit(private_cost.open_cpu);
            struct rusage cleanup_usage;memset(&cleanup_usage,0,sizeof(cleanup_usage));int cleanup_status=0,done=0;pid_t joined=0;
            int original_wait=known_exited(raw.pid,&done);
            if(original_wait&&done)for(;;) {
                joined=wait4(raw.pid,&cleanup_status,WNOHANG,&cleanup_usage);
                if(joined==raw.pid){retained_group=0;break;}
                if(joined<0&&errno!=EINTR)break;
                uint64_t now;if(!known_clock_ns(CLOCK_MONOTONIC,&now)||now>=outer.deadline)break;
                if(clock_ns(CLOCK_THREAD_CPUTIME_ID)-private_cost.open_cpu>=200000)(void)private_end_unit(private_cost.open_cpu);
                usleep(1000);
            }
            int cleanup_error=joined<0?errno:0;
            // Never signal a reaped or lost numeric identity on failure.
            if(joined<0&&cleanup_error==ECHILD)retained_group=0;
            int ids[3]={0};int group=proc_listpgrppids(raw.pid,ids,sizeof(ids));
            fprintf(stderr,"private_initial_physical_fixture_cleanup mode=%u pid=%d original_WNOWAIT_known=%d original_WNOWAIT_terminal=%d actual_wait4_pid=%d actual_wait4_errno=%d actual_wait_status=%d fixture_cleanup_actual_reaped=%s group_result=%d product_reported_reaped=%u product_reported_group_gone=%u signals_sent=0 wait4_usage_known=%s wait4_user_ns=%llu wait4_system_ns=%llu runtime_approved=false\n",
                mode,raw.pid,original_wait,done,joined,cleanup_error,cleanup_status,joined==raw.pid?"true":"false",group,owner.direct_reaped,result.group_gone,joined==raw.pid?"true":"false",
                (unsigned long long)engine_timeval(&cleanup_usage.ru_utime),(unsigned long long)engine_timeval(&cleanup_usage.ru_stime));
            (void)private_end_unit(private_cost.open_cpu);
            if(joined==raw.pid||(joined<0&&cleanup_error==ECHILD)) {
                engine_current=NULL;dispose_private_engine=NULL;
                (void)close(owner.ledger);owner.ledger=-1;
            }
            require(0,"initial stopped physical predicate refused before terminal RI; explicit fixture cleanup does not certify product settlement");
        }
        if(owner.settlement_quarantined){require(fflush(stdout)==0,"initial stopped prior stdout flush before held refusal");engine_quarantine_hold(&owner);}
        source_known_initial_ri_unavailable=source_known_initial_payment_interrupt=0;
        require(owner.direct_reaped&&owner.wait_observed&&result.reaped==1&&result.group_gone&&result.retired_coverage,
            "initial stopped fixture actual original own reap and group proof precede causal assertions");
        if(!mode) {
            // Frozen finite guard projections AFTER actual original cleanup.
            // These do not claim another native query or grant any CPU credit.
            struct engine_initial_physical_observation projected=owner.initial_physical;
            projected.bsd_known=0;projected.bsd_result=0;projected.bsd_error=ESRCH;projected.predicate_mask=1;
            require(engine_initial_physical_basis(&projected,1)==2,"original own-WNOWAIT retained source permits only the explicit unavailable-BSD physical basis");
            projected.own_wnowait_terminal=0;
            require(engine_initial_physical_basis(&projected,1)==0,"BSD ESRCH alone cannot replace original own-WNOWAIT terminal proof");
            projected.own_wnowait_terminal=1;
            require(engine_initial_physical_basis(&projected,0)==0,"unavailable BSD cannot waive initial source and original identity eligibility");
            projected.bsd_known=1;projected.predicate_mask=127;
            require(engine_initial_physical_basis(&projected,1)==0,"any returned BSD identity mismatch remains refused");
            projected.bsd_known=0;projected.predicate_mask=1;projected.child_result=-1;projected.child_error=ESRCH;
            require(engine_initial_physical_basis(&projected,1)==0,"unavailable child census is not silently promoted to zero children");
        }
        if(!mode)require(corroborated&&stages>=8&&max_stage<=1000000&&private_cost.debt==private_cost.debt_paid,
            "stopped corroboration bounded stages pay fresh debt instead of pooling ordinary work");
        else if(mode==1)require(corroborated&&owner.startup_charged&&owner.post_exit_observed&&owner.post_exit_initial
            &&source_known_initial_ri_calls==1&&owner.post_exit_ri_attempted&&!owner.grants&&!owner.grant.t0
            &&((owner.post_exit_ri_unavailable&&!owner.conservation_known&&!owner.justified_late_cpu)
                ||(owner.post_exit_delta!=UINT64_MAX&&owner.justified_late_cpu==owner.post_exit_delta&&owner.post_exit_after[0]>=owner.post_exit_before[0])),
            "charged never-continued startup obtains an independent paid original terminal RI before own wait4");
        else {
            require(corroborated&&owner.startup_charged&&owner.post_exit_observed&&owner.post_exit_initial
                &&owner.post_exit_initial_source_valid&&owner.post_exit_ri_unavailable&&owner.post_exit_attempt_retained
                &&source_known_initial_ri_calls==1&&owner.post_exit_ri_attempted
                &&owner.post_exit_attempt_paid&&!owner.conservation_known&&!owner.conservation_ok&&!owner.justified_late_cpu
                &&owner.failed&&private_cost.debt==private_cost.debt_paid,
                "unavailable initial RI is conservatively paid before actual own reap and remains strict refusal");
            if(mode==2)require(first_reaped&&!first_quarantined&&first_attempt_paid,
                "paid unavailable initial RI never deadlocks original wait4 on a missing counter");
            else require(!first_reaped&&first_quarantined&&!first_attempt_paid,
                "unpaid initial terminal attempt cannot release original wait4 ownership");
        }
        uint64_t reads=owner.counter_reads,debt=private_cost.debt;
        (void)engine_independent_post_exit(&owner);
        require(owner.counter_reads==reads&&private_cost.debt==debt,
            "initial terminal observation and raw late credit are once-only after actual reap");
        require(fsync(owner.ledger)==0&&close(owner.ledger)==0&&private_end_unit(private_cost.open_cpu),"initial stopped raw evidence and descriptor disposal separately paid");
        owner.ledger=-1;
        printf("private_initial_stopped_source_known mode=%u pid=%d birth=%llu no_CONT=true stages=%u nominal_stage_cpu_ns=%u max_stage_cpu_ns=%llu startup_charged=%u timebase=%u/%u post_exit_observed=%u post_exit_initial=%u raw_before_ticks=%llu raw_after_ticks=%llu justified_late_ns=%llu wait4_user_ns=%llu wait4_system_ns=%llu conservation_passed=%u conservation_excess_ns=%llu RI_unavailable=%u actual_RI_calls=%u attempt_retained=%u attempt_paid=%u first_reaped=%u first_quarantined=%u first_attempt_paid=%u physical_basis=%u BSD_known=%u BSD_result=%d BSD_errno=%d own_WNOWAIT_known=%u own_WNOWAIT_terminal=%u actual_reaped=true group_gone=true debt_charged_ns=%llu debt_paid_ns=%llu real_candidate_created=0 runtime_approved=false\n",
            mode,raw.pid,(unsigned long long)owner.ticket.birth,stages,mode?0:150000,(unsigned long long)max_stage,owner.startup_charged,
            owner.startup_timebase_numer,owner.startup_timebase_denom,owner.post_exit_observed,owner.post_exit_initial,
            (unsigned long long)owner.post_exit_before[0],(unsigned long long)owner.post_exit_after[0],(unsigned long long)owner.justified_late_cpu,
            (unsigned long long)owner.wait_user,(unsigned long long)owner.wait_system,owner.conservation_ok,(unsigned long long)owner.conservation_excess,
            owner.post_exit_ri_unavailable,source_known_initial_ri_calls,owner.post_exit_attempt_retained,owner.post_exit_attempt_paid,first_reaped,first_quarantined,first_attempt_paid,
            owner.initial_physical.physical_basis,owner.initial_physical.bsd_known,owner.initial_physical.bsd_result,owner.initial_physical.bsd_error,
            owner.initial_physical.own_wnowait_known,owner.initial_physical.own_wnowait_terminal,
            (unsigned long long)private_cost.debt,(unsigned long long)private_cost.debt_paid);
        require(fflush(stdout)==0&&private_end_unit(private_cost.open_cpu),"initial stopped stdout formatter and actual flush paid before fixture epoch ends");controls++;
    }
    reset();
}
int main(int argc,char **argv) {
    if(argc==2&&!strcmp(argv[1],"exit-child"))_exit(0);
    if(argc==3&&!strcmp(argv[1],"pending-exec-child")){
        int fd=atoi(argv[2]);if(fd<3||write(fd,"x",1)!=1||close(fd)!=0)_exit(77);
        volatile unsigned work=1;for(;;)work=work*33+7;
    }
    if(argc==2&&!strcmp(argv[1],"busy-child")){volatile uint64_t work=1;for(;;)work=work*33+7;}
    // Deliberate mutants run their existing causal witness directly, so an
    // unrelated process readiness gate cannot substitute for that assertion.
    unsigned payment_only=argc==2&&!strcmp(argv[1],"hold-payment-control");
    unsigned lifetime_only=argc==2&&!strcmp(argv[1],"hold-lifetime-control");
    unsigned diagnostics_only=argc==2&&!strcmp(argv[1],"terminal-diagnostic-controls");
    unsigned closure_only=argc==2&&!strcmp(argv[1],"closure-stage-controls");
    unsigned setup_budget_only=argc==2&&!strcmp(argv[1],"diagnostic-setup-budget-control");
    unsigned quantum_only=argc==3&&!strcmp(argv[1],"quantum-control");
    unsigned charge_only=argc==2&&!strcmp(argv[1],"charge-diagnostic-control");
    unsigned initial_only=argc==3&&!strcmp(argv[1],"initial-stopped-control");
    unsigned teardown_only=argc==3&&!strcmp(argv[1],"exit-teardown-control");
    if(argc==2&&!strcmp(argv[1],"hold-scheduling-control")){payment_only=1;hold_scheduling_only=1;}
    if(argc==2&&!strcmp(argv[1],"hold-final-scheduling-control")){payment_only=1;hold_scheduling_only=2;}
    require(argc==1||payment_only||lifetime_only||diagnostics_only||closure_only||setup_budget_only||quantum_only||charge_only||initial_only||teardown_only,"private source-known controls argc");require(atexit(dispose_owned)==0,"owned cleanup registration");
    char root[]="/private/tmp/dream-private-engine-controls.XXXXXX";require(mkdtemp(root)!=NULL,"owned fixture directory");
    int dir=open(root,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(dir>=0,"owned fixture root");
    fprintf(stderr,"private_engine_fixture_source_known pid=%d retained=%s real_candidate_created=0 runtime_approved=false\n",getpid(),root);
    require(fflush(stderr)==0,"early source-known fixture marker flush");
    if(setup_budget_only){diagnostic_setup_budget_control();require(close(dir)==0,"owned diagnostic budget root close");return 0;}
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
    reset();fixture(dir);
    char self[PROC_PIDPATHINFO_MAXSIZE];require(proc_pidpath(getpid(),self,sizeof(self))>0,"actual source-known image");
    if(diagnostics_only){terminal_diagnostic_controls(dir);require(close(dir)==0,"owned diagnostic-only root close");return 0;}
    if(closure_only){basis.source_coverage=127;basis.command_coverage=65535;closure_stage_controls(self,dir);require(close(dir)==0,"owned closure-only root close");return 0;}
    if(quantum_only){unsigned mode=(unsigned)strtoul(argv[2],NULL,10);require(mode<9,"exact nine source-known quantum controls");quantum_controls(self,dir,mode);require(close(dir)==0,"owned quantum-only root close");return 0;}
    if(charge_only){charge_diagnostic_control(dir);require(close(dir)==0,"owned charge-only root close");return 0;}
    if(initial_only){unsigned mode=(unsigned)strtoul(argv[2],NULL,10);require(mode<4,"exact four initial stopped source controls");initial_stopped_controls(self,dir,mode);require(close(dir)==0,"owned initial-only root close");return 0;}
    if(teardown_only){unsigned mode=(unsigned)strtoul(argv[2],NULL,10);require(mode<2,"exact two exit-teardown causal assertions");exit_teardown_control(self,dir,mode);require(close(dir)==0,"owned exit-teardown root close");return 0;}
    if(payment_only||lifetime_only) {
        require(engine_owner_priority(),"source-known narrow controller priority");
        basis.source_coverage=127;basis.command_coverage=65535;
        if(payment_only)process_control(self,0);
        else quarantine_hold_process_control(self,root,dir);
        require(close(dir)==0,"owned narrow fixture root close");
        printf("private_engine_controls=narrow_pass cases=%u synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);
        return 0;
    }
    semantic_controls();result_controls();flow_controls(root,dir);terminal_diagnostic_controls(dir);
    quantum_controls(self,dir,9);
    charge_diagnostic_control(dir);
    initial_stopped_controls(self,dir,4);
    exit_teardown_control(self,dir,2);
    process_control(self,0);process_control(self,1);
    closure_stage_controls(self,dir);pending_exec_control(self,dir);settlement_control(self,dir,0);settlement_control(self,dir,1);settlement_control(self,dir,2);
    quarantine_hold_process_control(self,root,dir);
    require(close(dir)==0,"owned fixture root close");
    printf("private_engine_controls=pass cases=%u synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=%s\n",controls,root);
    return 0;
}
