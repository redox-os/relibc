#include <stdio.h>
#include <stdlib.h>

int main(void) {
	FILE *f = fopen("stdio/stdio.in", "r");
    if (fseek(f, 14, SEEK_CUR) < 0) {
        puts("fseek error");
        return EXIT_FAILURE;
    }
    char buffer[256];
    printf("%s", fgets(buffer, 256, f));
    printf("ftell: %ld\n", ftello(f));
}
