#include <stdio.h>

int main(int argc, char **argv) {
    FILE *fp;
    int status;
    char path[256];


    fp = popen("ls -1 example_dir", "r");
    if (fp == NULL) {
        perror("popen");
        return -1;
    }

    while (fgets(path, 256, fp) != NULL) {
        printf("%s", path);
    }


    status = pclose(fp);
    if (status == -1) {
        perror("pclose");
        return -1;
    } else {
        printf("status %x\n", status);
    }

    return 0;
}
