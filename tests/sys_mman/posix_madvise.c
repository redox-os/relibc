#include <assert.h>
#include <stddef.h>
#include <sys/mman.h>
#include <unistd.h>
#include <errno.h>

int main() {
    int page_size = getpagesize();
    char *p = mmap(NULL, page_size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
    assert(p != MAP_FAILED);

    int valid_advices[5] = {POSIX_MADV_NORMAL, POSIX_MADV_RANDOM, POSIX_MADV_SEQUENTIAL, POSIX_MADV_WILLNEED, POSIX_MADV_WONTNEED};
    for (size_t idx = 0; idx < sizeof(valid_advices) / sizeof(valid_advices[0]); idx++) {
        int advice = valid_advices[idx];
        assert(posix_madvise(p, page_size, advice) == 0);
    }

    int invalid_advices[5] = {-1, 5, 6, 42, 123456};
    for (size_t idx = 0; idx < sizeof(invalid_advices) / sizeof(invalid_advices[0]); idx++) {
        int advice = invalid_advices[idx];
        assert(posix_madvise(p, page_size, advice) == EINVAL);
    }

    munmap(p, page_size);
    return 0;
}
