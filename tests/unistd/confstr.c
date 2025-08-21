#include <assert.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define CSPATHSZ 9
#define BADSIZ 4

int main(void) {
    char actual[CSPATHSZ] = {0};
    assert(confstr(_CS_PATH, actual, CSPATHSZ) == CSPATHSZ);
    const char expected[] = "/usr/bin";
    assert(strncmp(expected, actual, CSPATHSZ) == 0);

    // The constants other than _CS_PATH just return an empty str (no support).
    char empty[] = "";
    assert(
        confstr(
            _CS_POSIX_V6_LP64_OFF64_LIBS,
           empty,
           0
        ) == 1
    );

    // Buffers that are too small should return the expected size.
    char small[BADSIZ] = {0};
    assert(confstr(_CS_PATH, small, BADSIZ) == CSPATHSZ);

    // Null buffer and zero length should return the expected size
    // for the constant.
    assert(confstr(_CS_PATH, NULL, 0) == CSPATHSZ);

    return EXIT_SUCCESS;
}
