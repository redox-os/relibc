#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>

#include "test_helpers.h"

int main() {
    int page_size = getpagesize();
    printf("Page size: %d\n", page_size);

    puts("Mapping 1 page of memory...");
    char *map1 = mmap((void *) 0x200000000, (size_t) page_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED_NOREPLACE, -1, 0);
    puts("Mapping 3 pages of memory...");
    char *map2 = mmap(map1 + page_size, (size_t) page_size * 3, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED_NOREPLACE, -1, 0);

    ERROR_IF(mmap, map1, == MAP_FAILED);
    ERROR_IF(mmap, map2, == MAP_FAILED);

    puts("Randomizing mapping #1");
    for (int i = 0; i < page_size; ++i) {
        map1[i] = (char) (rand() & 0xFF);
    }
    puts("Randomizing mapping #2");
    for (int i = 0; i < page_size * 3; ++i) {
        map2[i] = (char) (rand() & 0xFF);
    }

    puts("Unmapping page 2 of map2");
    munmap(map2 + page_size, page_size);

    puts("Randomizing page 1 of mapping #2");
    for (int i = 0; i < page_size; ++i) {
        map2[i] = (char) (rand() % 256);
    }

    puts("Randomizing page 3 of mapping #2");
    for (int i = 0; i < page_size; ++i) {
        map2[page_size * 2 + i] = (char) (rand() % 256);
    }

    // Page fault:
    // map2[page_size] = 0;

    puts("Unmapping it all at once!");
    munmap(map1, (size_t) page_size * 4);

    // Page fault:
    // *map2 = 0;
}
