#include <unistd.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    // Constants specified in https://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

    printf("_POSIX_VERSION: %ld\n", _POSIX_VERSION);
    /* TODO
    printf("_POSIX2_VERSION: %ld\n", _POSIX2_VERSION);
    printf("_POSIX2_C_VERSION: %ld\n", _POSIX2_C_VERSION);
    printf("_XOPEN_VERSION: %d\n", _XOPEN_VERSION);

    printf("_XOPEN_XCU_VERSION: %d\n", _XOPEN_XCU_VERSION);
    
    printf("_XOPEN_XPG2: %d\n", _XOPEN_XPG2);
    printf("_XOPEN_XPG3: %d\n", _XOPEN_XPG3);
    printf("_XOPEN_XPG4: %d\n", _XOPEN_XPG4);
    printf("_XOPEN_UNIX: %d\n", _XOPEN_UNIX);

    printf("_POSIX_CHOWN_RESTRICTED: %d\n", _POSIX_CHOWN_RESTRICTED);
    printf("_POSIX_NO_TRUNC: %d\n", _POSIX_NO_TRUNC);
    printf("_POSIX_VDISABLE: %d\n", _POSIX_VDISABLE);
    printf("_POSIX_SAVED_IDS: %d\n", _POSIX_SAVED_IDS);
    printf("_POSIX_JOB_CONTROL: %d\n", _POSIX_JOB_CONTROL);
    printf("_POSIX_THREADS: %ld\n", _POSIX_THREADS);
    printf("_POSIX_THREAD_ATTR_STACKADDR: %ld\n", _POSIX_THREAD_ATTR_STACKADDR);
    printf("_POSIX_THREAD_ATTR_STACKSIZE: %ld\n", _POSIX_THREAD_ATTR_STACKSIZE);
    printf("_POSIX_THREAD_PROCESS_SHARED: %ld\n", _POSIX_THREAD_PROCESS_SHARED);
    printf("_POSIX_THREAD_SAFE_FUNCTIONS: %ld\n", _POSIX_THREAD_SAFE_FUNCTIONS);

    printf("_POSIX2_C_BIND: %ld\n", _POSIX2_C_BIND);
    printf("_POSIX2_C_DEV: %ld\n", _POSIX2_C_DEV);
    printf("_POSIX2_CHAR_TERM: %ld\n", _POSIX2_CHAR_TERM);
    printf("_POSIX2_FORT_DEV: %d\n", _POSIX2_FORT_DEV);
    printf("_POSIX2_FORT_RUN: %d\n", _POSIX2_FORT_RUN);
    printf("_POSIX2_LOCALEDEF: %ld\n", _POSIX2_LOCALEDEF);
    printf("_POSIX2_SW_DEV: %ld\n", _POSIX2_SW_DEV);
    printf("_POSIX2_UPE: %d\n", _POSIX2_UPE);
    printf("_XOPEN_CRYPT: %d\n", _XOPEN_CRYPT);
    printf("_XOPEN_ENH_I18N: %d\n", _XOPEN_ENH_I18N);
    printf("_XOPEN_LEGACY: %d\n", _XOPEN_LEGACY);
    printf("_XOPEN_REALTIME: %d\n", _XOPEN_REALTIME);
    printf("_XOPEN_REALTIME_THREADS: %d\n", _XOPEN_REALTIME_THREADS);
    printf("_XOPEN_SHM: %d\n", _XOPEN_SHM);
    printf("_XBS5_ILP32_OFF32: %d\n", _XBS5_ILP32_OFF32);
    printf("_XBS5_ILP32_OFFBIG: %d\n", _XBS5_ILP32_OFFBIG);
    printf("_XBS5_LP64_OFF64: %d\n", _XBS5_LP64_OFF64);
    printf("_XBS5_LPBIG_OFFBIG: %d\n", _XBS5_LPBIG_OFFBIG);

    printf("_POSIX_ASYNCHRONOUS_IO: %ld\n", _POSIX_ASYNCHRONOUS_IO);
    printf("_POSIX_MEMLOCK: %ld\n", _POSIX_MEMLOCK);
    printf("_POSIX_MEMLOCK_RANGE: %ld\n", _POSIX_MEMLOCK_RANGE);
    printf("_POSIX_MESSAGE_PASSING: %ld\n", _POSIX_MESSAGE_PASSING);
    printf("_POSIX_PRIORITY_SCHEDULING: %ld\n", _POSIX_PRIORITY_SCHEDULING);
    printf("_POSIX_REALTIME_SIGNALS: %ld\n", _POSIX_REALTIME_SIGNALS);
    printf("_POSIX_SEMAPHORES: %ld\n", _POSIX_SEMAPHORES);
    printf("_POSIX_SHARED_MEMORY_OBJECTS: %ld\n", _POSIX_SHARED_MEMORY_OBJECTS);
    printf("_POSIX_SYNCHRONIZED_IO: %ld\n", _POSIX_SYNCHRONIZED_IO);
    printf("_POSIX_TIMERS: %ld\n", _POSIX_TIMERS);

    printf("_POSIX_FSYNC: %ld\n", _POSIX_FSYNC);
    printf("_POSIX_MAPPED_FILES: %ld\n", _POSIX_MAPPED_FILES);
    printf("_POSIX_MEMORY_PROTECTION: %ld\n", _POSIX_MEMORY_PROTECTION);

    printf("_POSIX_PRIORITIZED_IO: %ld\n", _POSIX_PRIORITIZED_IO);

    printf("_POSIX_THREAD_PRIORITY_SCHEDULING: %ld\n", _POSIX_THREAD_PRIORITY_SCHEDULING);
    printf("_POSIX_THREAD_PRIO_INHERIT: %ld\n", _POSIX_THREAD_PRIO_INHERIT);
    printf("_POSIX_THREAD_PRIO_PROTECT: %ld\n", _POSIX_THREAD_PRIO_PROTECT);

    printf("_POSIX_ASYNC_IO: %d\n", _POSIX_ASYNC_IO);
    printf("_POSIX_PRIO_IO: %d\n", _POSIX_PRIO_IO);
    printf("_POSIX_SYNC_IO: %d\n", _POSIX_SYNC_IO);
    */

    printf("NULL: %p\n", NULL);

    printf("R_OK: %d\n", R_OK);
    printf("W_OK: %d\n", W_OK);
    printf("X_OK: %d\n", X_OK);
    printf("F_OK: %d\n", F_OK);

    /* TODO: confstr() constants:
    printf("_CS_PATH: %d\n", _CS_PATH);
    printf("_CS_XBS5_ILP32_OFF32_CFLAGS: %d\n", _CS_XBS5_ILP32_OFF32_CFLAGS);
    printf("_CS_XBS5_ILP32_OFF32_LDFLAGS: %d\n", _CS_XBS5_ILP32_OFF32_LDFLAGS);
    printf("_CS_XBS5_ILP32_OFF32_LIBS: %d\n", _CS_XBS5_ILP32_OFF32_LIBS);
    printf("_CS_XBS5_ILP32_OFF32_LINTFLAGS: %d\n", _CS_XBS5_ILP32_OFF32_LINTFLAGS);
    printf("_CS_XBS5_ILP32_OFFBIG_CFLAGS: %d\n", _CS_XBS5_ILP32_OFFBIG_CFLAGS);
    printf("_CS_XBS5_ILP32_OFFBIG_LDFLAGS: %d\n", _CS_XBS5_ILP32_OFFBIG_LDFLAGS);
    printf("_CS_XBS5_ILP32_OFFBIG_LIBS: %d\n", _CS_XBS5_ILP32_OFFBIG_LIBS);
    printf("_CS_XBS5_ILP32_OFFBIG_LINTFLAGS: %d\n", _CS_XBS5_ILP32_OFFBIG_LINTFLAGS);
    printf("_CS_XBS5_LP64_OFF64_CFLAGS: %d\n", _CS_XBS5_LP64_OFF64_CFLAGS);
    printf("_CS_XBS5_LP64_OFF64_LDFLAGS: %d\n", _CS_XBS5_LP64_OFF64_LDFLAGS);
    printf("_CS_XBS5_LP64_OFF64_LIBS: %d\n", _CS_XBS5_LP64_OFF64_LIBS);
    printf("_CS_XBS5_LP64_OFF64_LINTFLAGS: %d\n", _CS_XBS5_LP64_OFF64_LINTFLAGS);
    printf("_CS_XBS5_LPBIG_OFFBIG_CFLAGS: %d\n", _CS_XBS5_LPBIG_OFFBIG_CFLAGS);
    printf("_CS_XBS5_LPBIG_OFFBIG_LDFLAGS: %d\n", _CS_XBS5_LPBIG_OFFBIG_LDFLAGS);
    printf("_CS_XBS5_LPBIG_OFFBIG_LIBS: %d\n", _CS_XBS5_LPBIG_OFFBIG_LIBS);
    printf("_CS_XBS5_LPBIG_OFFBIG_LINTFLAGS: %d\n", _CS_XBS5_LPBIG_OFFBIG_LINTFLAGS);
    */

    printf("SEEK_SET: %d\n", SEEK_SET);
    printf("SEEK_CUR: %d\n", SEEK_CUR);
    printf("SEEK_END: %d\n", SEEK_END);

    // sysconf() constants (_SC_*) are tested separately

    printf("F_LOCK: %d\n", F_LOCK);
    printf("F_ULOCK: %d\n", F_ULOCK);
    printf("F_TEST: %d\n", F_TEST);
    printf("F_TLOCK: %d\n", F_TLOCK);

    // pathconf() constants (_PC_*) are tested separately

    printf("STDIN_FILENO: %d\n", STDIN_FILENO);
    printf("STDOUT_FILENO: %d\n", STDOUT_FILENO);
    printf("STDERR_FILENO: %d\n", STDERR_FILENO);

    return 0;
}
