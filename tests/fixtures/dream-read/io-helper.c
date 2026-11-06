// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Private passive process data fixture: no scheduler, controller, tool approval,
// proof schema, shell, exec, service, thread, or recursive process creation.
// pair forks exactly once; both retain this image and the inherited private group.
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static int output(int fd,const void *data,size_t len) {
    const char *p=data;
    while(len) {
        ssize_t n=write(fd,p,len);
        if(n<0&&errno==EINTR)continue;
        if(n<=0)return 81;
        p+=n;len-=(size_t)n;
    }
    return 0;
}
static int file(const char *name,const char *data,size_t len) {
    int fd=open(name,O_WRONLY|O_CREAT|O_EXCL,0600);
    if(fd<0)return 81;
    int result=output(fd,data,len);
    if(close(fd)!=0)return 81;
    return result;
}
static void held(void) {
    for(;;)pause(); // Blocking does not acknowledge STOP; the real controller must.
}
int main(int argc,char **argv) {
    if(argc<2)return 81;
    if(!strcmp(argv[1],"emit")&&argc==3) {
        size_t n=strlen(argv[2]);
        if(n>32)return 81;
        return output(STDOUT_FILENO,argv[2],n);
    }
    if(!strcmp(argv[1],"echo")&&argc==2) {
        char bytes[4096];
        for(;;) {
            ssize_t n=read(STDIN_FILENO,bytes,sizeof(bytes));
            if(n<0&&errno==EINTR)continue;
            if(n<0)return 81;
            if(n==0)return 0;
            if(output(STDOUT_FILENO,bytes,(size_t)n))return 81;
        }
    }
    if(!strcmp(argv[1],"stall")&&argc==2) {
        if(file("ready","ready",5))return 81;
        held();
    }
    if(!strcmp(argv[1],"ticks")&&argc==2) {
        if(file("ready","ready",5))return 81;
        int fd=open("ticks",O_WRONLY|O_CREAT|O_EXCL,0600);
        if(fd<0)return 81;
        for(;;)if(output(fd,"x",1))return 81;
    }
    if(!strcmp(argv[1],"pair")&&argc==2) {
        pid_t child=fork(); // One source-known fork; no child exec or further fork.
        if(child<0)return 81;
        if(child==0)held();
        char pid[32];int n=snprintf(pid,sizeof(pid),"%d\n",child);
        if(n<=0||(size_t)n>=sizeof(pid)||file("descendant",pid,(size_t)n))return 81;
        if(file("ready","ready",5))return 81;
        int status=0;pid_t joined;
        do {joined=waitpid(child,&status,0);}while(joined<0&&errno==EINTR);
        return joined==child?0:81;
    }
    return 81;
}
