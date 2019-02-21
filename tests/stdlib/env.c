#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    //puts(getenv("SHELL"));
    //puts(getenv("CC"));

    char* owned = malloc(26); // "TEST=Updates accordingly." + NUL
    strcpy(owned, "TEST=It's working!!");

    putenv(owned);
    puts(getenv("TEST"));

    strcpy(owned, "TEST=Updates accordingly.");
    puts(getenv("TEST"));

    // Allocation is reused
    setenv("TEST", "in place", 1);
    puts(getenv("TEST"));
    puts(owned);

    // Allocation is not reused
    setenv("TEST", "Value overwritten and not in place because it's really long", 1);
    puts(getenv("TEST"));
    puts(owned);

    // Value is not overwritten
    setenv("TEST", "Value not overwritten", 0);
    puts(getenv("TEST"));

    unsetenv("TEST");
    char* env = getenv("TEST");
    if (env) {
        puts("This should be null, but isn't");
        puts(env);
        exit(EXIT_FAILURE);
    } else {
        puts("Value deleted successfully!");
    }

    free(owned);
}
