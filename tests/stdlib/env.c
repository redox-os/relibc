#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test_helpers.h"

extern char **environ;

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

    setenv("TEST", "Reset environment variable", 1);

    // manually unset the environ pointers
    environ = NULL;

    env = getenv("TEST");
    if (env) {
      puts("Value should be null");
      exit(EXIT_FAILURE);
    } else {
      puts("environ unset successfully!");
    }

    owned = malloc(26);
    if (owned == NULL) {
      puts("Error allocating owned!");
      exit(EXIT_FAILURE);
    } else {
      strcpy(owned, "TEST=Updates accordingly.");
      putenv(owned);
    }

    env = getenv("TEST");
    if (env == NULL) {
      puts("putenv failed to set the environment");
      exit(EXIT_FAILURE);
    } else {
      puts("putenv call successful!");
      puts(env);
    }

    setenv("TEST1", "Set environ before manual.", 1);
    *(environ + 2) = "TEST2=Manual set environment.";
    *(environ + 3) = NULL;

    size_t i = 0;
    while (environ && environ[i]) {
      puts(environ[i++]);
    }

    unsetenv("TEST2");
    unsetenv("TEST1");
    unsetenv("TEST");

    free(owned);
}
