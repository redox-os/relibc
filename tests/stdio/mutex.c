#include <stdio.h>

int main() {
    FILE* f = fopen("stdio/stdio.in", "r");

    flockfile(f);

    // Commenting this out should cause a deadlock:
    // flockfile(f);

    if (!ftrylockfile(f)) {
        puts("Mutex unlocked but it shouldn't be");
        return -1;
    }
    funlockfile(f);

    if (ftrylockfile(f)) {
        puts("Mutex locked but it shouldn't be");
        return -1;
    }
    if (!ftrylockfile(f)) {
        puts("Mutex unlocked but it shouldn't be");
        return -1;
    }
    funlockfile(f);
}
