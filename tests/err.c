#include <err.h>
#include <errno.h>
#include <stdarg.h>
#include <stdlib.h>
#include <stdio.h>

__attribute__((nonnull(2)))
static void vwarn_test(int code, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vwarnc(code, fmt, ap);
    va_end(ap);
}

static void log_to_stdout(void) {
    err_set_file(stdout);
    warnc(ENOENT, "Dang it, Bobby.");
    err_set_file(NULL);
}

void user_callback(int code) {
    printf("Exiting due to error code: %d\n", code);
}

int main(void) {
    err_set_exit(user_callback);

    // Set errno to a known value for verifiable messages
    // (Also, "Owner died" is just too funny not to use)
    errno = EOWNERDEAD;

    warn("Ran out of coffee");
    warnx("%s pulled out your ethernet cable", "Cat");
    warnc(EACCES, "Eat %d cookies", 42);

    vwarn_test(EBADE, "Potato, %s", "krumpli");

    // Set the sink to stdout then back to stderr
    log_to_stdout();
    warnc(EPERM,
          "I'm sorry, Dave. I'm afraid I can't do that."
    );

    // As long as one err function works they should all work since
    // two functions handle everything internally.
    errc(EXIT_SUCCESS, EUSERS, "Bye. It's crowded.");

    // Unreachable
    puts("err did not exit");
    return EXIT_FAILURE;
}
