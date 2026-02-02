#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <grp.h>

void test_getgrnam(const char *gr_name) {
    struct group* out = getgrnam(gr_name);
    
    if (out == NULL) {
        // TODO: Check errno
        printf("Did not find a group '%s'\n", gr_name);
        return;
    }
    
    printf("getgrnam:   '%s' = %d\n", out->gr_name, out->gr_gid);
}

void test_getgrnam_r(const char *gr_name) {
    char buf[100];
    
    struct group grp;
    struct group* out;
    
    int status = getgrnam_r(gr_name, &grp, buf, sizeof(buf), &out);
    
    if (out == NULL) {
        const char *reason = (status != 0) ? strerror(status) : "(not found)";
        printf("Did not find a group '%s': %s\n", gr_name, reason);
        return;
    }
    
    printf("getgrnam_r: '%s' = %d\n", out->gr_name, out->gr_gid);
}

int main(void) {
    test_getgrnam("this_group_is_not_exist");
    test_getgrnam_r("this_group_is_not_exist");
    test_getgrnam("root");
    test_getgrnam_r("root");
}
