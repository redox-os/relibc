//1234 a
#include <stdio.h>
int main() {
        FILE *f = fopen("stdio/fscanf_offby1.c", "r");
        int x;
        fscanf(f, "//%d", &x);
        printf("%d, %ld, %d\n", x, ftell(f), fgetc(f));
}
