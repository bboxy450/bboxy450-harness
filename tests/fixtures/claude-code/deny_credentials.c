// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// macOS process witness: exit on credential reads/Keychain APIs in the harness.
// DYLD interposition is local to the disposable test process, with no privileges.
#include <Security/Security.h>
#include <fcntl.h>
#include <spawn.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#ifdef PROBE
int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "file") == 0) open("/tmp/.claude/.credentials.json", O_RDONLY);
    else SecItemCopyMatching(NULL, NULL);
    return 0;
}
#else
#define INTERPOSE(replacement, original) \
    __attribute__((used)) static struct { const void *r; const void *o; } interpose_##original \
    __attribute__((section("__DATA,__interpose"))) = { (const void *)&replacement, (const void *)&original };
static void check_path(const char *path) {
    if (path && (strstr(path, "/.claude/") || strstr(path, "/Keychains/") || strstr(path, ".keychain"))) _exit(91);
}
static int checked_open(const char *path, int flags, ...) {
    check_path(path);
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, int); va_end(args); }
    return open(path, flags, mode);
}
static int checked_openat(int fd, const char *path, int flags, ...) {
    check_path(path);
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, int); va_end(args); }
    return openat(fd, path, flags, mode);
}
static FILE *checked_fopen(const char *path, const char *mode) {
    check_path(path);
    return fopen(path, mode);
}
static OSStatus checked_keychain(CFDictionaryRef query, CFTypeRef *result) {
    (void)query; (void)result;
    _exit(92);
}
static int checked_spawn(pid_t *pid, const char *path, const posix_spawn_file_actions_t *actions,
                         const posix_spawnattr_t *attrs, char *const argv[], char *const env[]) {
    const char *base = strrchr(path, '/');
    if (strcmp(base ? base + 1 : path, "security") == 0) _exit(93);
    return posix_spawn(pid, path, actions, attrs, argv, env);
}
INTERPOSE(checked_open, open)
INTERPOSE(checked_openat, openat)
INTERPOSE(checked_fopen, fopen)
INTERPOSE(checked_keychain, SecItemCopyMatching)
INTERPOSE(checked_spawn, posix_spawn)

#endif
