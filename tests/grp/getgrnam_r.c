#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <grp.h>

bool test_getgrnam(char* gr_name) {
    struct group* out = getgrnam(gr_name);
    
    if (out == NULL) {
        printf("Did not find a group '%s'", gr_name);
        return false;
    }
    
    printf("getgrnam\n");

    char* start = out->gr_name;
    int len = strlen(out->gr_name);
    
    printf("    '%s' = %d\n", gr_name, out->gr_gid);
    
    return true;
}

bool test_getgrnam_r(char* gr_name) {
    char* buf[100];
    
    struct group grp;
    struct group* out = &grp;
    struct group* tmp;
    
    int status = getgrnam_r(gr_name, out, buf, sizeof(buf), &tmp);
    
    if (out == NULL) {
        printf("Did not find a group '%s'", gr_name);
        return false;
    }
    
    printf("getgrnam_r\n");   
    
    char* start = grp.gr_name;
    int len = strlen(grp.gr_name);

    printf("    '%s' = %d\n", gr_name, out->gr_gid);
    
    return true;
}

int main(void) {
    test_getgrnam("lcake");
    test_getgrnam_r("lcake");
}