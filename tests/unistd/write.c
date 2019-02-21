#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    write(STDOUT_FILENO, "Hello World!\n", 13);
}
