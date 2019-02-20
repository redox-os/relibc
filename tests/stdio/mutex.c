#include <stdio.h>
#include <stdlib.h>

int main(void) {
    FILE* f = fopen("stdio/stdio.in", "r");

    flockfile(f);

    // Commenting this out should cause a deadlock:
    // flockfile(f);

    if (!ftrylockfile(f)) {
        puts("Mutex unlocked but it shouldn't be");
        return EXIT_FAILURE;
    }
    funlockfile(f);

    if (ftrylockfile(f)) {
        puts("Mutex locked but it shouldn't be");
        return EXIT_FAILURE;
    }
    if (!ftrylockfile(f)) {
        puts("Mutex unlocked but it shouldn't be");
        return EXIT_FAILURE;
    }
    funlockfile(f);
}
