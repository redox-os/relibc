#include <stdio.h>
#include <grp.h>
#include <string.h>

int main(int argc, char** argv) {
    printf("getgrent\n");
    
    for (struct group* grp = getgrent(); grp != NULL; grp = getgrent())
        printf("    %s = %d\n", grp->gr_name, grp->gr_gid);
}