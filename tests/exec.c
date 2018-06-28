#include <unistd.h>

int main(int argc, char** argv) {
    char *const args[1] = {"arg"};
    execv("write", args);
    perror("execv");
    return 0;
}
