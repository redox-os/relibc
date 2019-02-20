#include <stdio.h>
#include <stdlib.h>

int main(void) {
    //FILE *f = fopen("/etc/ssl/certs/ca-certificates.crt", "r");
    FILE *f = fopen("stdio/stdio.in", "r");
    char line[256];

    while (1) {
        if (fgets(line, 256, f)) {
            fputs(line, stdout);
        } else {
            puts("EOF");
            if (!feof(f)) {
                puts("feof() not updated!");
                return EXIT_FAILURE;
            }
            break;
        }
    }
}
