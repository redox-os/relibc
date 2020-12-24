#include <errno.h>
#include <stdio.h>
#include "test_helpers.h"

int main(int argc, char **argv) {
    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);

    program_invocation_name = "yes, you can change this";

    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);
}
