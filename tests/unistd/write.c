#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int written = write(STDOUT_FILENO, "Hello World!\n", 13);
    ERROR_IF(write, written, == -1);
}
