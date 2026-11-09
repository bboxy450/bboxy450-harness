// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Source-known process fixture only; never an authenticated tool or crypto result.
#define _DARWIN_C_SOURCE 1
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
static unsigned long long cpu(void) {struct timespec t;if(clock_gettime(CLOCK_PROCESS_CPUTIME_ID,&t))_exit(82);return (unsigned long long)t.tv_sec*1000000000ULL+(unsigned long long)t.tv_nsec;}
int main(int argc,char **argv) {
    (void)argc;
    if(strstr(argv[0],"burn-tool")){unsigned long long start=cpu();while(cpu()-start<350000000){}return 0;}
    if(strstr(argv[0],"fork-tool")) {
        for(unsigned i=0;i<2;i++) {
            pid_t child=fork();if(child<0)return 82;if(!child)_exit(0);if(waitpid(child,NULL,0)!=child)return 82;
            // FD4 is inherited only for the one supported signature chain.
            // The test controller acknowledges NOTE_FORK1 before fork2.
            if(i==0){char ack;if(read(4,&ack,1)!=1)return 82;}
        }
        for(;;)pause();
    }
    return 0;
}
