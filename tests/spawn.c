#include <spawn.h>
#include <fcntl.h>
#include <stdio.h>
#include <bits/sigset-t.h>
#include <sched.h>

#include "test_helpers.h"

#define CHECK(cond, msg)                                \
    if (!cond)                                          \
    {                                                   \
        fprintf(stderr, "Error in spawn.h: %s\n", msg); \
        _exit(EXIT_FAILURE);                            \
    }

#define CHECK_WHEN_NULL(call, name)                                                         \
    if (!(call))                                                                            \
    {                                                                                       \
        fprintf(stderr, "Error in spawn.h: %s expected to fail when passing NULL\n", name); \
        _exit(EXIT_FAILURE);                                                                \
    }

int main()
{
    posix_spawn_file_actions_t fa_t;
    CHECK((posix_spawn_file_actions_init(&fa_t) == 0), "posix_spawn_file_actions_init failed")
    CHECK_WHEN_NULL(posix_spawn_file_actions_init(NULL), "posix_spawn_file_actions_init")
    CHECK((posix_spawn_file_actions_addopen(&fa_t, 1, ".", O_RDONLY, 1) == 0), "Error while adding open action")
    CHECK((posix_spawn_file_actions_addclose(&fa_t, 1) == 0), "Error while adding close action")
    CHECK((posix_spawn_file_actions_adddup2(&fa_t, -1, 3) == EBADF), "Adding dup2 with negative fd must fail")
    CHECK((fa_t.size == 64), "Size remains unchanged")
    CHECK_WHEN_NULL(posix_spawn_file_actions_destroy(NULL), "posix_spawn_file_actions_destroy")
    CHECK_WHEN_NULL(posix_spawn_file_actions_addclose(NULL, 1), "posix_spawn_file_actions_addclose")
    CHECK((posix_spawn_file_actions_destroy(&fa_t) == 0), "posix_spawn_file_actions_destroy failed")
    CHECK((fa_t.size == 0), "Expected 0 size in posix_spawn_file_actions_t after destruction")

    posix_spawnattr_t sa;
    CHECK((posix_spawnattr_init(&sa) == 0), "posix_spawnattr_init failed")
    CHECK((sa.param.sched_priority == 0 && sa.flags == 0 && sa.pgroup == 0 && sa.policy == 0 && sa.sigdefault == 0 && sa.sigmask == 0), "All fields expected to be zero after init in posix_spawnattr_t")
    CHECK_WHEN_NULL(posix_spawnattr_init(NULL), "posix_spawnattr_init")
    CHECK_WHEN_NULL(posix_spawnattr_destroy(NULL), "posix_spawnattr_destroy")

    struct sched_param sp1;
    sp1.sched_priority = 2;
    struct sched_param sp2;
    sp2.sched_priority = 0;
    CHECK_WHEN_NULL(posix_spawnattr_setschedparam(NULL, &sp1), "posix_spawnattr_setschedparam")
    CHECK_WHEN_NULL(posix_spawnattr_setschedparam(&sa, NULL), "posix_spawnattr_setschedparam")
    CHECK((posix_spawnattr_setschedparam(&sa, &sp1) == 0), "posix_spawnattr_setschedparam failed")
    CHECK_WHEN_NULL(posix_spawnattr_getschedparam(NULL, &sp2), "posix_spawnattr_getschedparam")
    CHECK_WHEN_NULL(posix_spawnattr_getschedparam(&sa, NULL), "posix_spawnattr_getschedparam")
    CHECK((posix_spawnattr_getschedparam(&sa, &sp2) == 0 && sp2.sched_priority == 2), "posix_spawnattr_getschedparam failed")

    CHECK_WHEN_NULL(posix_spawnattr_setschedpolicy(NULL, 0), "posix_spawnattr_setschedpolicy")
    CHECK((posix_spawnattr_setschedpolicy(&sa, 1) == 0), "posix_spawnattr_setschedpolicy failed")
    int pol = 0;
    CHECK_WHEN_NULL(posix_spawnattr_getschedpolicy(NULL, &pol), "posix_spawnattr_getschedpolicy")
    CHECK_WHEN_NULL(posix_spawnattr_getschedpolicy(&sa, NULL), "posix_spawnattr_getschedpolicy")
    CHECK((posix_spawnattr_getschedpolicy(&sa, &pol) == 0 && pol == 1), "posix_spawnattr_getschedpolicy failed")

    sigset_t sst = 1;
    sigset_t sst2 = 0;
    CHECK_WHEN_NULL(posix_spawnattr_setsigdefault(NULL, &sst), "posix_spawnattr_setsigdefault")
    CHECK_WHEN_NULL(posix_spawnattr_setsigdefault(&sa, NULL), "posix_spawnattr_setsigdefault")
    CHECK((posix_spawnattr_setsigdefault(&sa, &sst) == 0), "posix_spawnattr_setsigdefault failed")
    CHECK((posix_spawnattr_getsigdefault(&sa, &sst2) == 0 && sst2 == 1), "posix_spawnattr_getsigdefault failed")
    CHECK_WHEN_NULL(posix_spawnattr_getsigdefault(NULL, &sst2), "posix_spawnattr_getsigdefault")
    CHECK_WHEN_NULL(posix_spawnattr_getsigdefault(&sa, NULL), "posix_spawnattr_getsigdefault")

    sst = 3;
    CHECK_WHEN_NULL(posix_spawnattr_setsigmask(NULL, &sst), "posix_spawnattr_setsigmask")
    CHECK_WHEN_NULL(posix_spawnattr_setsigmask(&sa, NULL), "posix_spawnattr_setsigmask")
    CHECK((posix_spawnattr_setsigmask(&sa, &sst) == 0), "posix_spawnattr_setsigmask failed")
    CHECK_WHEN_NULL(posix_spawnattr_getsigmask(NULL, &sst2), "posix_spawnattr_getsigmask")
    CHECK_WHEN_NULL(posix_spawnattr_getsigmask(&sa, NULL), "posix_spawnattr_getsigmask")
    CHECK((posix_spawnattr_getsigmask(&sa, &sst2) == 0 && sst2 == 3), "posix_spawnattr_getsigmask failed")

    CHECK_WHEN_NULL(posix_spawnattr_setflags(NULL, 0), "posix_spawnattr_setflags")
    CHECK((posix_spawnattr_setflags(&sa, 1) == 0), "posix_spawnattr_setflags failed")
    short int pf = 0;
    CHECK_WHEN_NULL(posix_spawnattr_getflags(NULL, &pf), "posix_spawnattr_getflags")
    CHECK_WHEN_NULL(posix_spawnattr_getflags(&sa, NULL), "posix_spawnattr_getflags")
    CHECK((posix_spawnattr_getflags(&sa, &pf) == 0 && pf == 1), "posix_spawnattr_getflags failed")
    CHECK((posix_spawnattr_setflags(&sa, 7565) == EINVAL), "posix_spawnattr_setflags expected to fail when passing invalid bits")

    pid_t id = 0;
    CHECK_WHEN_NULL(posix_spawnattr_setpgroup(NULL, 3), "posix_spawnattr_setpgroup")
    CHECK((posix_spawnattr_setpgroup(&sa, 3) == 0), "posix_spawnattr_setpgroup failed")
    CHECK_WHEN_NULL(posix_spawnattr_getpgroup(NULL, &id), "posix_spawnattr_getpgroup")
    CHECK_WHEN_NULL(posix_spawnattr_getpgroup(&sa, NULL), "posix_spawnattr_getpgroup")
    CHECK((posix_spawnattr_getpgroup(&sa, &id) == 0 && id == 3), "posix_spawnattr_getpgroup failed")
}
