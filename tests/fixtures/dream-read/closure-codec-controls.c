// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Independent offline harness of the ONE product C decoder. No candidate runs.
#define main canonical_probe_main
#include "proof-helper-probe.c"
#undef main

static int fragment(struct json_reader *r,const char *name) {
    struct private_ref ref;struct private_slice text;struct private_macho macho;
    struct private_link_map map;struct private_node node;struct private_load load;
    struct private_platform platform;char value[4097],hash[65];uint32_t count;unsigned source;
    if(!strncmp(name,"#/$defs/",8))name+=8;
    if(!strcmp(name,"ID"))return private_text(r,&text,128,1);
    if(!strcmp(name,"Text"))return private_text(r,&text,4096,1);
    if(!strcmp(name,"PathText"))return private_path_text(r,&text);
    if(!strcmp(name,"RelativePath"))return jstring(r,value,sizeof(value))&&private_relative_path(value);
    if(!strcmp(name,"Ref"))return jref(r,&ref);
    if(!strcmp(name,"StaticUse"))return private_static_use_read(r,&text,&count,hash);
    if(!strcmp(name,"Macho"))return private_macho_read(r,&macho);
    if(!strcmp(name,"BinaryMacho"))return private_macho_read(r,&macho)&&macho.subtype==0;
    if(!strcmp(name,"LinkMap"))return private_link_read(r,&map);
    if(!strcmp(name,"Image"))return private_node_read(r,&node);
    if(!strcmp(name,"BinaryImage"))return private_node_read(r,&node)&&!node.kind;
    if(!strcmp(name,"DylibImage"))return private_node_read(r,&node)&&node.kind==1;
    if(!strcmp(name,"Archive"))return private_archive_read(r,&node);
    if(!strcmp(name,"Load"))return private_load_read(r,&load);
    if(!strcmp(name,"Target"))return private_target_read(r,&load);
    if(!strcmp(name,"CandidateTarget"))return private_target_read(r,&load)&&!load.cache;
    if(!strcmp(name,"CacheTarget"))return private_target_read(r,&load)&&load.cache;
    if(!strcmp(name,"Platform"))return private_platform_read(r,&platform);
    if(!strcmp(name,"SourceID"))return private_source_index(r,&source);
    if(!strcmp(name,"Hex"))return jhex(r,hash);
    if(!strcmp(name,"Uuid"))return private_uuid(r,value);
    return 0;
}
static void additional_boundaries(void) {
    char resolved[4097];const char *prefix="/private/codec-prefix";
    require(private_normalize("/private/codec-prefix/bin","../lib/private.dylib",strlen(prefix),resolved)
        &&!strcmp(resolved,"/private/codec-prefix/lib/private.dylib"),"in-prefix loader parent resolves");
    require(!private_normalize("/private/codec-prefix/bin","../../codec-prefix/lib/private.dylib",strlen(prefix),resolved),
        "lexical escape cannot re-enter prefix");
    static struct private_basis b;struct private_load load;memset(&load,0,sizeof(load));load.cache=1;
    const unsigned char name[]="\"/System/Volumes/Data/candidate/lib/private.dylib\"";
    load.name=(struct private_slice){name,sizeof(name)-1};
    require(!private_load_resolves(&b,0,&load,"/System/Volumes/Data/candidate"),"candidate prefix cannot claim cache target");
}
int main(int argc,char **argv) {
    if(argc!=4)return 2;
    additional_boundaries();
    unsigned char bytes[49153];size_t len=0;
    while(len<sizeof(bytes)){ssize_t n=read(STDIN_FILENO,bytes+len,sizeof(bytes)-len);if(n<0)return 2;if(!n)break;len+=(size_t)n;}
    // The same paced parser is exercised without any source or helper identity.
    memset(&private_cost,0,sizeof(private_cost));private_cost.deadline=clock_ns(CLOCK_MONOTONIC)+30000000000ULL;
    private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);private_ref_hook=NULL;
    int ok=len<=49152&&private_canonical(bytes,len);
    struct json_reader r={bytes,bytes+len,0,10};
    if(ok&&!strcmp(argv[1],"schema_fragment"))ok=fragment(&r,argv[2])&&r.p==r.end;
    else if(ok&&!strcmp(argv[1],"closure")) {
        static struct private_basis b;memset(&b,0,sizeof(b));
        // Parse Platform6 separately for shape testing. Context equality belongs
        // to the signed synthetic offline corpus runner, not a runtime fallback.
        const unsigned char *platform=NULL;
        const char key[]="\"platform\":";
        for(size_t i=0;i+sizeof(key)-1<len;i++)if(!memcmp(bytes+i,key,sizeof(key)-1)){platform=bytes+i+sizeof(key)-1;break;}
        if(!platform)ok=0;
        else {struct json_reader pr={platform,bytes+len,0,10};ok=private_platform_read(&pr,&b.build.platform);}
        if(ok)ok=private_closure_read(&r,&b)&&r.p==r.end;
        if(ok){for(unsigned i=0;i<2;i++)b.build.tools[i].ref=b.nodes[i].file;
            ok=private_closure_bind_prefix(&b,argv[3]);}
    } else if(ok&&strcmp(argv[1],"canonical_grammar"))return 2;
    if(retained_group||retained_child||engine_current)return 3;
    return ok?0:81;
}
