#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    // testing shell detection
    // this means, because we don't detect if a shell actually exists but just assume it does, this test case breaks in
    // environments without `sh`. I think that is a reasonable tradeoff.
    // (And if there isn't a shell, system() won't work anyways)
    int status = system(NULL);
    printf("shell found: %i\n", status);
    fflush(stdout);
    ERROR_IF(system, status, == 0);

    // base case
    status = system("echo test of system");
    ERROR_IF(system, status, == -1);
}
