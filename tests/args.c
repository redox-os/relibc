#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

int main(int argc, char *argv[]) {
    for(int i = 0; i < argc; i++) {
        write(STDOUT_FILENO, argv[i], strlen(argv[i]));
        write(STDOUT_FILENO, " ", 1);
    }
    write(STDOUT_FILENO, "\n", 1);
}
