#include <stdlib.h>
#include <unistd.h>

int main(void) {
    write(STDOUT_FILENO, "Hello World!\n", 13);
    return EXIT_SUCCESS;
}
