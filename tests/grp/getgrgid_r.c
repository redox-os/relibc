#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <grp.h>

bool test_getgrgid(gid_t gid) {
    struct group* out = getgrgid(gid);
    
    if (out == NULL) {
        printf("Did not find a group %d", gid);
        return false;
    }
    
    printf("getgrgid\n");
    
    char* start = out->gr_name;
    int len = strlen(out->gr_name);
    
    printf("    %d = %s, GID: %d\n", gid, out->gr_name, out->gr_gid);
    
    return true;
}

bool test_getgrgid_r(gid_t gid) {
    char* buf[100];
    
    struct group grp;
    struct group* out = &grp;
    struct group* tmp;
    
    int status = getgrgid_r(gid, out, buf, sizeof(buf), &tmp);
    
    if (out == NULL) {
        printf("Did not find a group %d", gid);
        return false;
    }
    
    printf("getgrgid_r\n");   
    
    char* start = grp.gr_name;
    int len = strlen(grp.gr_name);
    
    printf("    %d = %s, GID: %d\n", gid, grp.gr_name, grp.gr_gid);
    
    return true;
}

int main(void) {
    test_getgrgid(1050);
    test_getgrgid_r(1050);
}