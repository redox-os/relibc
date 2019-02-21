#include <unistd.h>
#include <sys/stat.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    int mk_status = mkdir("foo", 0);
    ERROR_IF(mkdir, mk_status, == -1);
    UNEXP_IF(mkdir, mk_status, != 0);

    int rm_status = rmdir("foo");
    ERROR_IF(rmdir, rm_status, == -1);
    UNEXP_IF(rmdir, rm_status, != 0);
}
