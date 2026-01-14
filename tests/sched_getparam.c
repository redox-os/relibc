#include <stdio.h>
#include <sched.h>

int main() {
    struct sched_param param = 0;

    // 0 = current process
    if (sched_getparam(0, &param) == 0) {
        printf("Success! Priority: %d\n", param.sched_priority);
        return 0;
    } else {
        perror("sched_getparam failed");
        return -1;
    }
}
