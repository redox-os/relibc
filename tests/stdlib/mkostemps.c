#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char* file_name = (char*) calloc(18, sizeof(char));
    strcpy(file_name, "tempXXXXXX.suffix");
    int fd = mkostemps(file_name, 7, 0);
    FILE* fp = fdopen(fd, "w+");
    printf("Start unchanged: %d\n", strncmp(file_name, "temp", 4));
    printf("End unchanged: %d\n", strcmp(file_name + 4 + 6, ".suffix"));

    char* write = "Writing to file";
    fputs(write, fp);

    char buffer[sizeof write];
    memset(buffer, 0, sizeof buffer);
    fgets(buffer, strlen(buffer), fp);
    if (strcmp(write, buffer)) {
        printf("Read & Write Successful\n");
    } else {
        printf("Read & Write Failed\n");
    }
    fclose(fp);
    remove(file_name);
}
