#include <stdio.h>
#include <stdlib.h>

int main(void) {
    FILE *fp;
    int status;
    char path[256];


    fp = popen("ls -1 example_dir", "r");
    if (fp == NULL) {
        perror("popen");
        return EXIT_FAILURE;
    }

    while (fgets(path, 256, fp) != NULL) {
        printf("%s", path);
    }


    status = pclose(fp);
    if (status == -1) {
        perror("pclose");
        return EXIT_FAILURE;
    } else {
        printf("status %x\n", status);
    }

    return EXIT_SUCCESS;
}
