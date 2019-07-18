#include <stdio.h>
#include <unistd.h>

int main(int argc, char ** argv) {

    const char source[] = {0, 1, 2, 3, 4, 5, 6};
    char destination[] = {0, 0, 0, 0, 0, 0};
    const char flipped_source[] = {1, 0, 3, 2, 5, 4};
    const char first_two_source_bytes_flipped[] = {1, 0};

    swab(source, destination, /* nbytes */ -3);
    for (int i = 0; i < sizeof(destination); ++i) {
        if (destination[i] != 0) {
            puts("If nbytes is negative destionation shouldn't be modified");
            return 1;
        }
    }

    swab(source, destination, /* nbytes */ 0);
    for (int i = 0; i < sizeof(destination); ++i) {
        if (destination[i] != 0) {
            puts("If nbytes is zero destionation shouldn't be modified");
            return 1;
        }
    }

    swab(source, destination, /* nbytes */ 3);
    for (int i = 0; i < sizeof(first_two_source_bytes_flipped); ++i) {
        if (destination[i] != first_two_source_bytes_flipped[i]) {
            puts("copied bytes don't match expected values");
            return 1;
        }
    }
    // If nbytes is not even it's not specified what should happen to the
    // last byte so the third byte in destination is not checked.
    for (int i = sizeof(first_two_source_bytes_flipped) + 1; i < sizeof(destination); ++i) {
        if (destination[i] != 0) {
            puts("swab modified too many bytes in destination");
            return 1;
        }
    }

    swab(source, destination, /* nbytes */ sizeof(destination));
    for (int i = 0; i < sizeof(destination); ++i) {
        if (destination[i] != flipped_source[i]) {
            puts("copied bytes don't match expected values");
            return 1;
        }
    }

    return 0;
}
