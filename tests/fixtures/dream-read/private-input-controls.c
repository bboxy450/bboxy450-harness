// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Source-free outer-codec API fixture; parsing never admits a candidate.
#define main canonical_probe_main
#include "proof-helper-probe.c"
#undef main

int main(int argc,char **argv) {
    require(argc==3,"outer codec fixture argc");
    require(!strcmp(argv[1],"accept")||!strcmp(argv[1],"reject"),"explicit parser expectation");
    int fd=open(argv[2],O_RDONLY|O_NOFOLLOW|O_CLOEXEC);
    require(fd>=0,"owned fixture bytes");
    unsigned char bytes[PRIVATE_INPUT_MAX+1];size_t length=0;
    while(length<sizeof(bytes)) {
        ssize_t n=read(fd,bytes+length,sizeof(bytes)-length);
        require(n>=0,"fixture read");if(!n)break;length+=(size_t)n;
    }
    require(close(fd)==0,"actual fixture descriptor close");
    struct private_input value;
    int accepted=private_decode(bytes,length,&value);
    require(accepted==!strcmp(argv[1],"accept"),"canonical outer parser verdict");
    require(!retained_group&&!retained_child&&!engine_current,"parsing cannot create a helper effect");
    printf("private_outer_codec=pass accepted=%s helper_created=0 runtime_approved=false\n",accepted?"true":"false");
    return 0;
}
