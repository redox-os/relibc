#define _LARGEFILE64_SOURCE

#include <fcntl.h>
#include <glob.h>
#include <stdio.h>
#include <stdlib.h>

#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/resource.h>

int main(void) {
    // These tests are just to check the defines compile.
    // It doesn't really matter if the calls themselves fail.
    struct stat stat = {0};
    stat64("largefile64.c", &stat);


    return EXIT_SUCCESS;
}
