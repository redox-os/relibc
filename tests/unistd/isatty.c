#include <stdio.h>
#include <unistd.h>

int main() {
    // 1 is stdout
    if (isatty(1)) {
        puts("'Tis a tty :D");
    } else {
        puts("Whatever a tty is, it's not me");
    }
}
