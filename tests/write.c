#include <unistd.h>

int main(int argc, char ** argv) {
    write(STDOUT_FILENO, "Hello World!\n", 13);
    return 0;
}
