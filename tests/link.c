#include <unistd.h>

int main(int argc, char** argv) {
    int status = link("link.c", "link.out");
}
