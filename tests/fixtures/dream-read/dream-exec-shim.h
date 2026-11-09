// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#ifndef DREAM_EXEC_SHIM_H
#define DREAM_EXEC_SHIM_H
#include <stdint.h>
#include <sys/stat.h>
#define DREAM_SHIM_MAGIC UINT64_C(0x445245414d534831)
#define DREAM_SHIM_CONFIG_FD 3
#define DREAM_SHIM_LOG_FD 4
#define DREAM_SHIM_BYTES 32768u
struct dream_shim_config {
    uint64_t magic,signature_len;struct stat scratch_stat;
    uint32_t form,argc,envc,used;uint32_t argv[64],env[16];
    char git[4097],gpg[4097],shim[4097],home[4097],scratch[4097],git_exec_path[4097],signature_sha[65];
    char bytes[DREAM_SHIM_BYTES];
};
struct dream_shim_record {
    uint64_t magic;uint32_t form;int32_t pid,priority;struct stat signature;
    char name[20],sha[65];
};
#endif
