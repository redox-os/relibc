#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    char* cwd1 = malloc(4096*sizeof(char));//(char*) calloc(4096 + 1, sizeof(char));
    getcwd(cwd1, 4096);
    printf("initial cwd: %s\n", cwd1);
    free(cwd1);
    chdir("..");
    char* cwd2 = malloc(4096*sizeof(char));//(char*) calloc(4096 + 1, sizeof(char));
    getcwd(cwd2, 4096);
    printf("final cwd: %s\n", cwd2);
    free(cwd2);
    return 0;
}
