// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Decision 66459 + correction + amendment 2: conditional supported-tool boundary.
// NOTE_FORK/census silence is not an execution or whole-lifetime theorem.
#define _DARWIN_C_SOURCE 1
#include "dream-exec-shim.h"
#include <CommonCrypto/CommonDigest.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <libproc.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/proc_info.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <unistd.h>
extern char **environ;
static struct dream_shim_config config;
static int refuse(void) {return 81;}
static int terminated(const char *s,size_t n) {return memchr(s,0,n)!=NULL;}
static const char *entry(uint32_t offset) {
    return offset<config.used&&terminated(config.bytes+offset,config.used-offset)?config.bytes+offset:NULL;
}
static int same(const struct stat *a,const struct stat *b) {
    return a->st_dev==b->st_dev&&a->st_ino==b->st_ino&&a->st_mode==b->st_mode&&a->st_uid==b->st_uid&&a->st_nlink==b->st_nlink&&a->st_size==b->st_size
        &&a->st_mtimespec.tv_sec==b->st_mtimespec.tv_sec&&a->st_mtimespec.tv_nsec==b->st_mtimespec.tv_nsec
        &&a->st_ctimespec.tv_sec==b->st_ctimespec.tv_sec&&a->st_ctimespec.tv_nsec==b->st_ctimespec.tv_nsec;
}
static int private_dir(int fd) {struct stat s;return fstat(fd,&s)==0&&S_ISDIR(s.st_mode)&&(s.st_mode&07777)==0700&&s.st_uid==geteuid();}
static int custody(const char *path,struct dream_shim_record *r) {
    size_t prefix=strlen(config.scratch);if(strncmp(path,config.scratch,prefix)||path[prefix]!='/')return -1;
    const char *name=path+prefix+1;
    if(strlen(name)!=19||strncmp(name,".git_vtag_tmp",13)||strchr(name,'/'))return -1;
    for(unsigned i=13;i<19;i++)if(!((name[i]>='a'&&name[i]<='z')||(name[i]>='A'&&name[i]<='Z')||(name[i]>='0'&&name[i]<='9')))return -1;
    int dir=open(config.scratch,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);if(dir<0)return -1;
    struct stat ds,named,held,after;int fd=-1,ok=0;
    if(!private_dir(dir)||fstat(dir,&ds)||ds.st_dev!=config.scratch_stat.st_dev||ds.st_ino!=config.scratch_stat.st_ino)goto done;
    fd=openat(dir,name,O_RDONLY|O_NOFOLLOW|O_NONBLOCK|O_CLOEXEC);if(fd<0||fstat(fd,&held))goto done;
    if(!S_ISREG(held.st_mode)||(held.st_mode&07777)!=0600||held.st_uid!=geteuid()||held.st_nlink!=1||held.st_dev!=ds.st_dev
        ||held.st_size<0||(uint64_t)held.st_size!=config.signature_len||!config.signature_len||config.signature_len>2097152)goto done;
    CC_SHA256_CTX hash;CC_SHA256_Init(&hash);unsigned char bytes[4096],digest[32];uint64_t length=0;
    for(;;){ssize_t n=read(fd,bytes,sizeof(bytes));if(n<0)goto done;if(!n)break;length+=(uint64_t)n;if(length>config.signature_len)goto done;CC_SHA256_Update(&hash,bytes,(CC_LONG)n);}
    CC_SHA256_Final(digest,&hash);for(unsigned i=0;i<32;i++)snprintf(r->sha+2*i,3,"%02x",digest[i]);
    if(length!=config.signature_len||strcmp(r->sha,config.signature_sha)||fstat(fd,&after)||!same(&held,&after)
        ||fstatat(dir,name,&named,AT_SYMLINK_NOFOLLOW)||!same(&held,&named))goto done;
    DIR *d=fdopendir(dup(dir));if(!d)goto done;unsigned entries=0;int valid=1;
    for(;;){errno=0;struct dirent *e=readdir(d);if(!e){if(errno)valid=0;break;}if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;if(++entries>1||strcmp(e->d_name,name)){valid=0;break;}}
    if(closedir(d)||!valid||entries!=1)goto done;
    struct stat current;if(lstat(config.scratch,&current)||current.st_dev!=ds.st_dev||current.st_ino!=ds.st_ino||!private_dir(dir))goto done;
    r->signature=held;strcpy(r->name,name);ok=1;
 done:
    close(dir);if(!ok&&fd>=0){close(fd);fd=-1;}return fd;
}
static int record(const struct dream_shim_record *r,int signature_fd) {
    struct iovec io={(void *)r,sizeof(*r)};union {struct cmsghdr align;char bytes[CMSG_SPACE(sizeof(int))];} control;
    struct msghdr message={0};message.msg_iov=&io;message.msg_iovlen=1;
    if(signature_fd>=0){memset(&control,0,sizeof(control));message.msg_control=control.bytes;message.msg_controllen=sizeof(control.bytes);
        struct cmsghdr *c=CMSG_FIRSTHDR(&message);c->cmsg_level=SOL_SOCKET;c->cmsg_type=SCM_RIGHTS;c->cmsg_len=CMSG_LEN(sizeof(int));memcpy(CMSG_DATA(c),&signature_fd,sizeof(int));}
    return sendmsg(DREAM_SHIM_LOG_FD,&message,MSG_DONTWAIT)==(ssize_t)sizeof(*r);
}
// The sealed configuration binds variable operands to this command. Fixed
// tokens are checked independently here; even a matching daemon form refuses.
static int fixed_form(int argc,char **argv) {
    static const unsigned tailc[]={2,2,3,3,3,4,5,2,5,2,2,3,4,3,2};
    static const char *const tail[15][5]={
        {"rev-parse","--is-bare-repository"},{"rev-parse","--show-toplevel"},{"rev-parse","--verify",NULL},
        {"rev-parse","--verify","refs/remotes/origin/main"},{"cat-file","-t",NULL},
        {"merge-base","--is-ancestor",NULL,NULL},{"ls-tree","-z",NULL,"--",NULL},{"show",NULL},
        {"rev-list","-1",NULL,"--",NULL},{"rev-parse",NULL},{"rev-parse",NULL},
        {"config","--get","remote.origin.url"},{"log","-1","--format=%G?%x00%GF%x00%GP",NULL},
        {"--with-colons","--show-keys","-"},{"--import","-"}};
    unsigned form=config.form,prefix=form>=13?4:16;
    if((unsigned)argc!=1+prefix+tailc[form])return 0;
    const char *git_prefix[]={"--no-pager","--no-optional-locks","-c","core.fsmonitor=false","-c","core.hooksPath=/dev/null","-c","credential.helper=","-c","protocol.allow=never","-c","gpg.format=openpgp","-c",NULL,"-c",NULL};
    const char *gpg_prefix[]={"--no-options","--no-autostart","--batch","--no-tty"};
    for(unsigned i=0;i<prefix;i++) {
        const char *expected=form>=13?gpg_prefix[i]:git_prefix[i];char program[4128];
        if(!expected){snprintf(program,sizeof(program),"%s=%s",i==13?"gpg.program":"gpg.openpgp.program",config.shim);expected=program;}
        if(strcmp(argv[1+i],expected))return 0;
    }
    for(unsigned i=0;i<tailc[form];i++)if(tail[form][i]&&strcmp(argv[1+prefix+i],tail[form][i]))return 0;
    return 1;
}
// Git2.54.0 config.c/exec-cmd.c/setup.c export these exact derived
// settings to its verifier. They are validated, then removed before GPG exec.
static int append_text(char *out,size_t cap,size_t *used,const char *text) {
    size_t n=strlen(text);if(n>=cap-*used)return 0;memcpy(out+*used,text,n);*used+=n;out[*used]=0;return 1;
}
static int quote_parameter(char *out,size_t cap,size_t *used,const char *text) {
    if(!append_text(out,cap,used,"'"))return 0;
    for(;*text;text++) {
        char escaped[5]={0};
        if(*text=='\''||*text=='!'){escaped[0]='\'';escaped[1]='\\';escaped[2]=*text;escaped[3]='\'';}
        else escaped[0]=*text;
        if(!append_text(out,cap,used,escaped))return 0;
    }
    return append_text(out,cap,used,"'");
}
static int git_parameters(char *out,size_t cap) {
    const char *keys[]={"core.fsmonitor","core.hooksPath","credential.helper","protocol.allow","gpg.format","gpg.program","gpg.openpgp.program"};
    const char *values[]={"false","/dev/null","","never","openpgp",config.shim,config.shim};
    size_t used=0;out[0]=0;if(!append_text(out,cap,&used,"GIT_CONFIG_PARAMETERS="))return 0;
    for(unsigned i=0;i<7;i++) {
        if((i&&!append_text(out,cap,&used," "))||!quote_parameter(out,cap,&used,keys[i])
            ||!append_text(out,cap,&used,"=")||!quote_parameter(out,cap,&used,values[i]))return 0;
    }
    return 1;
}
static int public_config(void) {
    char path[4097];int length=snprintf(path,sizeof(path),"%s/gpg.conf",config.home);if(length<=0||(size_t)length>=sizeof(path))return 0;
    const char expected[]="batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n";
    int fd=open(path,O_RDONLY|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);if(fd<0)return 0;struct stat st;char bytes[sizeof(expected)];
    int ok=fstat(fd,&st)==0&&S_ISREG(st.st_mode)&&(st.st_mode&07777)==0600&&st.st_uid==geteuid()&&st.st_nlink==1
        &&read(fd,bytes,sizeof(bytes))==(ssize_t)(sizeof(expected)-1)&&!memcmp(bytes,expected,sizeof(expected)-1);close(fd);return ok;
}
int main(int argc,char **argv) {
    struct stat cfg;int type=0;socklen_t type_len=sizeof(type);
    if(fstat(DREAM_SHIM_CONFIG_FD,&cfg)||!S_ISREG(cfg.st_mode)||cfg.st_uid!=geteuid()||(cfg.st_mode&07777)!=0600||cfg.st_nlink!=0||cfg.st_size!=(off_t)sizeof(config)
        ||(fcntl(DREAM_SHIM_CONFIG_FD,F_GETFL)&O_ACCMODE)!=O_RDONLY
        ||getsockopt(DREAM_SHIM_LOG_FD,SOL_SOCKET,SO_TYPE,&type,&type_len)||type!=SOCK_DGRAM
        ||pread(DREAM_SHIM_CONFIG_FD,&config,sizeof(config),0)!=(ssize_t)sizeof(config)||config.magic!=DREAM_SHIM_MAGIC
        ||config.form>=15||config.argc<2||config.argc>64||config.envc!=11||config.used>DREAM_SHIM_BYTES)return refuse();
    if(!terminated(config.git,sizeof(config.git))||!terminated(config.gpg,sizeof(config.gpg))||!terminated(config.shim,sizeof(config.shim))
        ||!terminated(config.home,sizeof(config.home))||!terminated(config.scratch,sizeof(config.scratch))||!terminated(config.git_exec_path,sizeof(config.git_exec_path))||!terminated(config.signature_sha,sizeof(config.signature_sha))
        ||config.git[0]!='/'||config.gpg[0]!='/'||config.shim[0]!='/')return refuse();
    int verify=argc==6&&!strcmp(argv[1],"--keyid-format=long")&&!strcmp(argv[2],"--status-fd=1")&&!strcmp(argv[3],"--verify")&&!strcmp(argv[5],"-");
    const char *fixed[]={"GIT_CONFIG_NOSYSTEM=1","GIT_CONFIG_GLOBAL=/dev/null","GIT_TERMINAL_PROMPT=0","GIT_ASKPASS=/usr/bin/false","SSH_ASKPASS=/usr/bin/false","LANG=C","LC_ALL=C","PATH=/usr/bin:/bin"};
    for(unsigned i=0;i<8;i++){const char *e=entry(config.env[i]);if(!e||strcmp(e,fixed[i]))return refuse();}
    for(unsigned i=8;i<11;i++){char expected[4107];snprintf(expected,sizeof(expected),"%s=%s",i==8?"HOME":i==9?"GNUPGHOME":"TMPDIR",i==10?config.scratch:config.home);const char *e=entry(config.env[i]);if(!e||strcmp(e,expected))return refuse();}
    char parameters[DREAM_SHIM_BYTES],exec_path[4120],path[4140];
    if(verify&&(config.git_exec_path[0]!='/'||!git_parameters(parameters,sizeof(parameters))))return refuse();
    snprintf(exec_path,sizeof(exec_path),"GIT_EXEC_PATH=%s",config.git_exec_path);
    snprintf(path,sizeof(path),"PATH=%s:/usr/bin:/bin",config.git_exec_path);
    const char *derived[]={parameters,exec_path,"GIT_OPTIONAL_LOCKS=0","GIT_PAGER=cat","GIT_PREFIX=",path};
    size_t total=0;unsigned envc=0,seen=0;
    for(;environ[envc];envc++) {
        if(envc>=16)return refuse();size_t n=strlen(environ[envc])+1;
        if(n>DREAM_SHIM_BYTES||total>DREAM_SHIM_BYTES-n)return refuse();total+=n;
        unsigned matches=0,bit=0;
        for(unsigned j=0;j<config.envc;j++) {
            if(verify&&j==7)continue;const char *e=entry(config.env[j]);
            if(e&&!strcmp(e,environ[envc])){matches++;bit=1u<<j;}
        }
        if(verify)for(unsigned j=0;j<6;j++)if(!strcmp(derived[j],environ[envc])){matches++;bit=1u<<(11+j);}
        if(matches!=1||(seen&bit))return refuse();seen|=bit;
    }
    if(envc!=(verify?16u:config.envc))return refuse(); // END SHIM ENV ALLOWLIST
    if(verify){if(config.form!=12||!public_config())return refuse();}
    else {if(!fixed_form(argc,argv)||(unsigned)argc!=config.argc)return refuse();for(unsigned i=0;i<config.argc;i++){const char *e=entry(config.argv[i]);if(!e||strcmp(argv[i],e))return refuse();}}
    for(int i=0;i<argc;i++){size_t n=strlen(argv[i])+1;if(n>4097||total>DREAM_SHIM_BYTES-n)return refuse();total+=n;}
    struct proc_taskallinfo task;int n=proc_pidinfo(getpid(),PROC_PIDTASKALLINFO,0,&task,sizeof(task));
    if(n!=(int)sizeof(task)||task.ptinfo.pti_priority!=4)return refuse();
    struct dream_shim_record r={0};r.magic=DREAM_SHIM_MAGIC;r.form=verify?15:config.form;r.pid=getpid();r.priority=task.ptinfo.pti_priority;
    int signature_fd=verify?custody(argv[4],&r):-1;if(verify&&signature_fd<0)return refuse();
    if(!record(&r,signature_fd)){if(signature_fd>=0)close(signature_fd);return refuse();}
    if(signature_fd>=0)close(signature_fd);
    // Config/log inheritance is necessary only for Git's one shim descendant.
    int keep=(!verify&&config.form==12)?4:2;
    for(int fd=keep+1;fd<getdtablesize();fd++)close(fd);
    struct rlimit core={0,0};if(setrlimit(RLIMIT_CORE,&core))return refuse();
    argv[0]=verify?config.gpg:(config.form>=13?config.gpg:config.git);
    char *clean_env[12]={0};for(unsigned i=0;i<config.envc;i++)clean_env[i]=(char *)entry(config.env[i]);
    execve(argv[0],argv,clean_env);return refuse();
}
