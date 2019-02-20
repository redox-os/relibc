#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int filter(const struct dirent* dirent) {
    return strstr(dirent->d_name, "3") == NULL;
}

int main(void) {
    struct dirent** array;
    int len = scandir("example_dir/", &array, filter, alphasort);
    if (len < 0) {
        perror("scandir");
        return EXIT_FAILURE;
    }

    for(int i = 0; i < len; i += 1) {
        puts(array[i]->d_name);
        free(array[i]);
    }
    free(array);
}
