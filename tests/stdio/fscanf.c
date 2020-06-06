//@ 1 2 3
//@ SS
#include <stdio.h>
int main() {
    FILE *f = fopen("stdio/fscanf.c", "r");
    int x, y, z;
    fscanf(f, "//@ %d %d %d",&x , &y, &z);
    printf("%d %d %d %ld\n", x, y, z, ftell(f));
    while(-1 != fscanf(f, "%*[^\n]")) {
        printf("%ld\n", ftell(f));
        getc(f);
    }
}