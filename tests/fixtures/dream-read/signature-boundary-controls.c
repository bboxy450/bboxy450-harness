// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Private Darwin capability witness; neither a product helper nor an authority source.
// Component API controls only; real Git/GPG qualification is a separate gate.
#define main canonical_probe_main
#include "proof-helper-probe.c"

#undef main
static void scratch_settlement_controls(const char *root) {
    // API fixture: supply an already retained file to GS2. This does not claim
    // GS1, a verifier incarnation or actual Git leader reap was observed.
    for(unsigned variant=0;variant<3;variant++) {
        char case_path[4097];int n=snprintf(case_path,sizeof(case_path),"%s/settle%u",root,variant);
        require(n>0&&(size_t)n<sizeof(case_path)&&mkdir(case_path,0700)==0,"owned settlement case");
        struct private_input outer;memset(&outer,0,sizeof(outer));
        struct private_basis basis;memset(&basis,0,sizeof(basis));
        struct private_engine_context context={&outer,&basis,NULL};
        outer.output.fd=open(case_path,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
        require(outer.output.fd>=0&&mkdirat(outer.output.fd,"t",0700)==0,"fresh owned scratch");
        struct private_git_sig_temp sig;memset(&sig,0,sizeof(sig));sig.context=&context;
        sig.scratch_fd=openat(outer.output.fd,"t",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
        sig.signature_fd=sig.transient_fd=-1;strcpy(sig.name,".git_vtag_tmpAb3xY0");sig.len=3;
        require(sig.scratch_fd>=0&&fstat(sig.scratch_fd,&sig.scratch_stat)==0,"retained original scratch");
        int writer=openat(sig.scratch_fd,sig.name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
        require(writer>=0&&write(writer,"sig",3)==3&&close(writer)==0,"owned signature bytes");
        sig.signature_fd=openat(sig.scratch_fd,sig.name,O_RDONLY|O_NOFOLLOW|O_CLOEXEC);
        require(sig.signature_fd>=0&&fstat(sig.signature_fd,&sig.signature_stat)==0,"held signature inode");
        private_hash((const unsigned char *)"sig",3,sig.sha256);
        sig.initialized=sig.ready=sig.bound=1;sig.records=1;
        require(private_sig_scratch_current(&sig),"scratch current before causal replacement");
        if(variant!=2)require(unlinkat(sig.scratch_fd,sig.name,0)==0,"normal Git deletion API fixture");
        if(variant) {
            require(renameat(outer.output.fd,"t",outer.output.fd,"old-t")==0&&mkdirat(outer.output.fd,"t",0700)==0,"replace only owned named scratch");
            int replacement=openat(outer.output.fd,"t",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
            require(replacement>=0,"retain replacement scratch");
            int foreign=openat(replacement,sig.name,O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);
            require(foreign>=0&&write(foreign,"new",3)==3&&close(foreign)==0,"owned foreign replacement marker");
            private_failed=0;private_error="input_invalid";private_detail="unset";
            unsigned char bytes[8];
            require(!private_git_sig_gs2(&sig,1,variant==1,bytes,sizeof(bytes)),"GS2 refuses detached scratch on normal or killed settlement");
            require(!strcmp(private_error,"settlement_unknown")&&sig.quarantined&&!sig.settled&&sig.signature_fd>=0,"refusal retains original descriptor and quarantine");
            struct stat marker;require(fstatat(replacement,sig.name,&marker,AT_SYMLINK_NOFOLLOW)==0&&marker.st_size==3,"replacement entry is never unlinked");
            if(variant==2)require(fstatat(sig.scratch_fd,sig.name,&marker,AT_SYMLINK_NOFOLLOW)==0,"kill refuses before unlinking original from detached scratch");
            require(close(replacement)==0&&close(sig.signature_fd)==0,"test owner closes retained refusal descriptors");
        } else {
            private_failed=0;private_error="input_invalid";private_detail="unset";
            unsigned char bytes[8];
            require(private_git_sig_gs2(&sig,1,1,bytes,sizeof(bytes))&&sig.settled&&sig.signature_fd<0,"unchanged scratch and deleted held inode settle through actual GS2 API");
        }
        require(close(sig.scratch_fd)==0&&close(outer.output.fd)==0,"test owner closes exact scratch descriptors");
    }
    puts("scratch_settlement_controls=pass current_scratch=settled replaced_normal=quarantined replaced_killed=quarantined foreign_unlink=false real_pair=false");
}
static void reset_control(void) {
    private_failed=0;private_error="input_invalid";private_detail="component unset";
    memset(&private_cost,0,sizeof(private_cost));private_cost.entry=clock_ns(CLOCK_MONOTONIC);
    private_cost.deadline=private_cost.entry+30000000000ULL;private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);
}
int main(int argc,char **argv) {
    require(argc==2,"owned component directory");
    scratch_settlement_controls(argv[1]);
    reset_control();char out[4097];const char *bad[]={"{git_signature_temp}","prefix{git_signature_temp}","{git_signature_temp}/suffix","{signature}","prefix{signature}/suffix"};
    for(unsigned i=0;i<5;i++){strcpy(out,"unchanged");require(!engine_expand(NULL,bad[i],"h","t",out),"matcher/legacy token must never enter any expander");require(!strcmp(out,"unchanged"),"refused token cannot produce an argv path");}
    struct engine_prepared p;require(!engine_prepare_argv(NULL,15,"h","t",&p),"implicit verify_child cannot be expanded or directly prepared");
    require(!p.attr_live&&!p.actions_live&&p.input[0]<0&&p.output[0]<0,"no native preparation effect from matcher form");
    struct private_input outer;memset(&outer,0,sizeof(outer));strcpy(outer.case_name,"positive");
    struct private_basis basis;memset(&basis,0,sizeof(basis));basis.fixture.valid_signature.len=1;basis.fixture.corrupt_signature.len=ENGINE_INPUT_MAX;
    struct private_engine_context ctx={&outer,&basis,"0000000000000000000000000000000000000000000000000000000000000000"};
    require(!engine_signature_boundary_available(&ctx)&&!strcmp(private_error,"review_unbound"),"B matcher is current but independent missing relations still refuse before birth");
    reset_control();strcpy(outer.case_name,"corrupt_proof");
    require(!engine_signature_boundary_available(&ctx)&&strstr(private_detail,"input buffer"),"selected corrupt Ref receives its own prebirth bounded-buffer check");
    reset_control();strcpy(outer.case_name,"positive");engine_signature_storage.initialized=1;
    require(!engine_signature_boundary_available(&ctx)&&strstr(private_detail,"cannot be rebound"),"original signature custody cannot be replaced");engine_signature_storage.initialized=0; // no descriptor/effect was created by this component control
    reset_control();struct private_engine_result result;memset(&result,0,sizeof(result));struct engine_owner owner;memset(&owner,0,sizeof(owner));owner.context=&ctx;owner.result=&result;
    int status=0;require(!engine_command(&owner,&p,15,"h","t",NULL,0,&status),"actual command API refuses direct verify_child");
    require(!result.created&&!result.attempts&&owner.ticket.pid==0,"direct matcher rejection occurs before any reservation or birth");
    reset_control();memset(&owner,0,sizeof(owner));owner.context=&ctx;owner.result=&result;
    require(!engine_command(&owner,&p,0,"h","t",NULL,0,&status)&&!strcmp(private_error,"review_unbound"),"actual command API requires prepared B custody before any helper");
    require(!result.created&&!result.attempts&&retained_group==0&&retained_child==0,"no helper effect from missing B binding");
    mode_t original_mask=umask(0600);reset_control();
    int rejected_mask=!engine_signature_umask_allowed();mode_t retained_mask=umask(0);umask(original_mask);
    require(rejected_mask&&retained_mask==0600&&!strcmp(private_error,"review_unbound"),"forbidden mask refuses and is restored before any directory or helper");
    reset_control();require(engine_signature_umask_allowed(),"normal controller mask accepted without mutation");
    reset_control();strcpy(outer.case_name,"positive");strcpy(outer.output.path,argv[1]);
    outer.output.fd=open(argv[1],O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);require(outer.output.fd>=0,"retain owned output directory");
    require(mkdirat(outer.output.fd,"t",0700)==0,"fresh component scratch");
    struct private_git_sig_temp sig;memset(&sig,0,sizeof(sig));sig.scratch_fd=openat(outer.output.fd,"t",O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);sig.signature_fd=-1;
    require(sig.scratch_fd>=0&&fstat(sig.scratch_fd,&sig.scratch_stat)==0,"retain actual scratch metadata");
    basis.fixture.valid_signature.len=3;memset(basis.fixture.valid_signature.sha,'a',64);basis.fixture.valid_signature.sha[64]=0;
    sig.initialized=sig.ready=1;sig.context=&ctx;sig.selected=&basis.fixture.valid_signature;sig.len=3;memcpy(sig.sha256,basis.fixture.valid_signature.sha,65);
    sig.recorded_umask=umask(0);umask(sig.recorded_umask);require(!(sig.recorded_umask&0600),"observed component umask preserves0600");
    int ledger=openat(outer.output.fd,"ledger.ndjson",O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);require(ledger>=0,"retain existing ledger before helper effect");
    size_t bytes=0;unsigned records=0;
    require(engine_signature_prepare_record(&ctx,&sig,ledger,&bytes,&records),"actual one-shot prebirth receipt write and file/directory persistence");
    require(private_end_unit(private_cost.open_cpu)&&private_cost.debt>=9000000&&private_cost.debt_paid==private_cost.debt,"full ordinary receipt CPU and fresh debt paid before any effect");
    require(records==1&&bytes>0&&bytes<512,"receipt consumes existing byte/record reservation");
    char actual[512]={0},mask[64],identity[96];require(pread(ledger,actual,sizeof(actual)-1,0)==(ssize_t)bytes,"actual persisted receipt bytes");
    snprintf(mask,sizeof(mask),"\"recorded_umask\":%u",(unsigned)sig.recorded_umask);
    snprintf(identity,sizeof(identity),"\"scratch_ino\":%llu",(unsigned long long)sig.scratch_stat.st_ino);
    require(strstr(actual,"\"phase\":\"git_sig_prepare\"")&&strstr(actual,mask)&&strstr(actual,identity)
        &&strstr(actual,"\"selected_case\":\"valid\"")&&strstr(actual,sig.sha256),"receipt exposes exact retained umask/scratch/selected case");
    sig.selected=&basis.fixture.corrupt_signature;reset_control();
    require(!engine_signature_prepare_record(&ctx,&sig,ledger,&bytes,&records)&&!strcmp(private_error,"review_unbound")&&records==1,"wrong selected-case receipt cannot append or consume a record");
    sig.selected=&basis.fixture.valid_signature;
    size_t exhausted_bytes=ENGINE_CAPTURE_MAX;unsigned exhausted_records=ENGINE_RECORD_MAX;reset_control();
    require(!engine_signature_prepare_record(&ctx,&sig,ledger,&exhausted_bytes,&exhausted_records)&&!strcmp(private_error,"resource_limit"),"existing ledger exhaustion refuses before another write");
    struct stat unchanged;require(fstat(ledger,&unchanged)==0&&(uint64_t)unchanged.st_size==bytes,"failed reservation cannot rewrite or append receipt");
    require(!result.created&&!result.attempts&&retained_group==0&&retained_child==0,"receipt control launches no helper");
    require(close(ledger)==0&&close(sig.scratch_fd)==0&&close(outer.output.fd)==0,"component descriptors actually closed");
    printf("engine_B_controls=pass token_refusals=5 no_direct_verify_child=true selected_case_bound=true no_rebind=true prebirth_receipt=durable ordinary_debt=paid ledger_bound=pass forbidden_umask=refused wrong_case_receipt=refused created=0 runtime_approved=false\n");return 0;
}
