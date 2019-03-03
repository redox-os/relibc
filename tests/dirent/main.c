#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    printf("%lu\n", sizeof(struct dirent));

    DIR* dir = opendir("example_dir/");
    ERROR_IF(opendir, dir, == NULL);

    struct dirent* entry;

    //int tell = 0;

    for (char counter = 0; (entry = readdir(dir)); counter += 1) {
        puts(entry->d_name);

        //if (counter == 4) {
        //    tell = telldir(dir);
        //}
    }

    puts("--- Testing rewind ---");
    rewinddir(dir);
    entry = readdir(dir);
    puts(entry->d_name);

    // puts("--- Testing seek ---");
    // // Why this doesn't cause it to actually go to the 4th element is beyond
    // // me, but glibc acts the same way.
    // seekdir(dir, tell);
    // entry = readdir(dir);
    // puts(entry->d_name);

    int c = closedir(dir);
    ERROR_IF(closedir, c, == -1);
    UNEXP_IF(closedir, c, != 0);
}
