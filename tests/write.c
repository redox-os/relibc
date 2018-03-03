#include <unistd.h>

void _start(void) {
    write(STDOUT_FILENO, "Hello World!\n", 14);
}
