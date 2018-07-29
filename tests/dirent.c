#include <dirent.h>
#include <errno.h>
#include <stdio.h>

int main() {
    printf("%lu\n", sizeof(struct dirent));

    DIR* dir = opendir("example_dir/");

    if (dir == NULL) {
        perror("opendir");
        return 1;
    }

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
}
