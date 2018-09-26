#include <stdio.h>

int main() {
	FILE *f = fopen("stdio/stdio.in", "r");
    if (fseek(f, 14, SEEK_CUR) < 0) {
        puts("fseek error");
        return 1;
    }
    char buffer[256];
    printf("%s", fgets(buffer, 256, f));
    printf("ftell: %d\n", ftello(f));
}
