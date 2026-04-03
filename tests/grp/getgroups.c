#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <grp.h>

int main() {
    gid_t primary_gid = getegid();
    struct group *pg = getgrgid(primary_gid);

    printf("getegid: %u (%s)\n", primary_gid, pg ? pg->gr_name : "?");

    int count = getgroups(0, NULL);
    gid_t *list = malloc(sizeof(gid_t) * count);
    getgroups(count, list);

    printf("getgroups: %d\n", count);
    for (int i = 0; i < count; i++) {
        struct group *sg = getgrgid(list[i]);
        
        printf("  - %u (%s)\n", list[i], sg ? sg->gr_name : "?");
    }

    free(list);
    return 0;
}