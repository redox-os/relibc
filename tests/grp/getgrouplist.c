#include <stdio.h>
#include <grp.h>

int main(void) {
    gid_t groups[20];
    int ngroup = 20;
    int num_groups = getgrouplist("user", 1000, groups, &ngroup);
    
    printf("Num Groups: %d\n", num_groups);
    
    for (int i = 0; i < num_groups; i++)
        printf("i: %d\n", groups[i]);
    
    return 0;
}