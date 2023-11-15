#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <grp.h>

void test_getgrgid(gid_t gid) {
    struct group *out = getgrgid(gid);
    
    if (out == NULL) {
        printf("Did not find a group %d\n", gid);
        return;
    }
    
    printf("getgrgid\n");
    
    printf("    %d = %s, GID: %d\n", gid, out->gr_name, out->gr_gid);
}

void test_getgrgid_r(gid_t gid) {
    char buf[100];
    
    struct group grp;
    struct group *tmp;
    
    int status = getgrgid_r(gid, &grp, buf, sizeof(buf), &tmp);
    
    if (tmp == NULL) {
        const char *reason = status != 0 ? strerror(status) : "(not found)";
        printf("Did not find a group %d: %s\n", gid, reason);
        return;
    }
    
    printf("getgrgid_r\n");   
    
    printf("    %d = %s, GID: %d\n", gid, grp.gr_name, grp.gr_gid);
}

int main(void) {
    test_getgrgid(1050);
    test_getgrgid_r(1050);
}
