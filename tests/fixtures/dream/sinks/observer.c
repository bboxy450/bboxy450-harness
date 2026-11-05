// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Independent macOS test observer, compiled separately from the producer. It
// observes actual libc calls/results/digests, not producer-declared operations.
#include <CommonCrypto/CommonDigest.h>
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
static __thread int nested;
static char *marker, *trace, *fault;
__attribute__((constructor)) static void init(void) {
    marker=getenv("DREAM_WP8_OBSERVER_MARKER"); trace=getenv("DREAM_WP8_OBSERVER_TRACE"); fault=getenv("DREAM_WP8_OBSERVER_FAULT");
}
static int path_for(int fd,const char *leaf,char *out) {
    char base[1024]; if(fcntl(fd,F_GETPATH,base)!=0)return 0;
    if(leaf)snprintf(out,2048,"%s/%s",base,leaf);else snprintf(out,2048,"%s",base);return 1;
}
static int active(const char *path) {
    if(!marker||!trace||nested)return 0; nested=1;
    char root[2048];int fd=open(marker,O_RDONLY);ssize_t n=fd<0?-1:read(fd,root,sizeof(root)-1);if(fd>=0)close(fd);nested=0;
    if(n<=0)return 0;root[n]=0;while(n>0&&(root[n-1]=='\n'||root[n-1]=='\r'))root[--n]=0;
    return strncmp(path,root,(size_t)n)==0&&(path[n]=='/'||path[n]==0);
}
static void emit(const char *op,long result,const char *a,const char *b,const void *bytes,size_t len) {
    if(!active(a))return; int saved=errno;nested=1;char hash[65]="-",line[8192];
    if(bytes){unsigned char digest[CC_SHA256_DIGEST_LENGTH];CC_SHA256(bytes,(CC_LONG)len,digest);for(int i=0;i<32;i++)snprintf(hash+2*i,3,"%02x",digest[i]);}
    int n=snprintf(line,sizeof(line),"%s\t%ld\t%s\t%s\t%s\t%zu\n",op,result,a,b?b:"-",hash,len);
    int fd=open(trace,O_WRONLY|O_CREAT|O_APPEND,0600);if(fd>=0){write(fd,line,(size_t)n);close(fd);}nested=0;errno=saved;
}
static int observed_openat(int dir,const char *path,int flags,...) {
    mode_t mode=0;if(flags&O_CREAT){va_list a;va_start(a,flags);mode=(mode_t)va_arg(a,int);va_end(a);}int result=openat(dir,path,flags,mode);int saved=errno;char full[2048];
    if(path[0]=='/')snprintf(full,sizeof(full),"%s",path);else if(!path_for(dir,path,full)){errno=saved;return result;}
    emit("open",result,full,NULL,NULL,(size_t)flags);errno=saved;return result;
}
static int observed_open(const char *path,int flags,...) {
    mode_t mode=0;if(flags&O_CREAT){va_list a;va_start(a,flags);mode=(mode_t)va_arg(a,int);va_end(a);}int result=open(path,flags,mode);int saved=errno;
    emit("open",result,path,NULL,NULL,(size_t)flags);errno=saved;return result;
}
static ssize_t observed_write(int fd,const void *bytes,size_t len) {
    char full[2048];if(!path_for(fd,NULL,full))return write(fd,bytes,len);
    const void *actual=bytes;void *copy=NULL;
    if(active(full)&&fault&&strcmp(fault,"wrong_temp")==0&&strstr(full,"/.dream-tmp-")&&!strstr(full,"/blobs/")&&len>0){copy=malloc(len);memcpy(copy,bytes,len);((char*)copy)[0]^=1;actual=copy;}
    ssize_t result=write(fd,actual,len);int saved=errno;emit("write",result,full,NULL,actual,result>0?(size_t)result:0);free(copy);errno=saved;return result;
}
static int observed_fsync(int fd) {
    char full[2048];if(!path_for(fd,NULL,full))return fsync(fd);
    if(active(full)&&fault&&strcmp(fault,"omit_dir_sync")==0){struct stat st;if(fstat(fd,&st)==0&&S_ISDIR(st.st_mode)){emit("omitted_fsync",0,full,NULL,NULL,0);return 0;}}
    int result=fsync(fd);int saved=errno;emit("fsync",result,full,NULL,NULL,0);errno=saved;return result;
}
static int observed_fcntl(int fd,int command,...) {
    if(command==F_FULLFSYNC || command==F_BARRIERFSYNC) {
        char full[2048]; int valid=path_for(fd,NULL,full);
        if(valid&&active(full)&&fault&&strcmp(fault,"omit_dir_sync")==0){struct stat st;if(fstat(fd,&st)==0&&S_ISDIR(st.st_mode)){emit("omitted_fsync",0,full,NULL,NULL,0);return 0;}}
        int result=fcntl(fd,command);int saved=errno;if(valid)emit("fsync",result,full,NULL,NULL,0);errno=saved;return result;
    }
    if(command==F_GETFL || command==F_GETFD)return fcntl(fd,command);
    va_list a;va_start(a,command);int result;
    if(command==F_SETFL||command==F_SETFD||command==F_DUPFD||command==F_DUPFD_CLOEXEC){int value=va_arg(a,int);result=fcntl(fd,command,value);}else{void *value=va_arg(a,void*);result=fcntl(fd,command,value);}va_end(a);return result;
}
static int observed_linkat(int d1,const char *p1,int d2,const char *p2,int flags) {
    char a[2048],b[2048];int valid=path_for(d1,p1,a)&&path_for(d2,p2,b);
    if(valid&&active(b)&&!strstr(b,"/blobs/")&&fault&&strcmp(fault,"hand_race")==0){nested=1;int fd=openat(d2,p2,O_WRONLY|O_CREAT|O_EXCL,0600);if(fd>=0){write(fd,"handwritten\n",12);fsync(fd);close(fd);}nested=0;emit("external_hand",fd>=0?0:-1,b,NULL,NULL,0);}
    int result=linkat(d1,p1,d2,p2,flags);int saved=errno;if(valid)emit("link",result,a,b,NULL,0);errno=saved;return result;
}
static int observed_unlinkat(int dir,const char *path,int flags){char full[2048];int valid=path_for(dir,path,full);int result=unlinkat(dir,path,flags);int saved=errno;if(valid)emit("unlink",result,full,NULL,NULL,0);errno=saved;return result;}
static int observed_mkdirat(int dir,const char *path,mode_t mode){char full[2048];int valid=path_for(dir,path,full);int result=mkdirat(dir,path,mode);int saved=errno;if(valid)emit("mkdir",result,full,NULL,NULL,0);errno=saved;return result;}
static int observed_rename(const char *a,const char *b){int result=rename(a,b);int saved=errno;emit("rename",result,a,b,NULL,0);errno=saved;return result;}
#define INTERPOSE(replacement,replacee) __attribute__((used)) static struct {const void *new_function;const void *old_function;} interpose_##replacee __attribute__((section("__DATA,__interpose")))={(const void*)&replacement,(const void*)&replacee};
INTERPOSE(observed_openat,openat)
INTERPOSE(observed_open,open)
INTERPOSE(observed_write,write)
INTERPOSE(observed_fsync,fsync)
INTERPOSE(observed_linkat,linkat)
INTERPOSE(observed_unlinkat,unlinkat)
INTERPOSE(observed_mkdirat,mkdirat)
INTERPOSE(observed_rename,rename)

INTERPOSE(observed_fcntl,fcntl)
