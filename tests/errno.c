#include <errno.h>
#include <stdio.h>
#include "test_helpers.h"

int main(int argc, char **argv) {
    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);

    argv[0] = "changed to argv[0]";
    program_invocation_name = "changed to program_invocation_name";
    program_invocation_short_name = "changed to program_invocation_short_name";

    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);
}
