
#include <stdio.h>

int main(int argc, char ** argv) {
    char buffer[72];
    int ret = sprintf(
        buffer,
        "This string fits in the buffer because it is only %d bytes in length",
        68
    );

    if (ret) {
        printf("Failed! %d\n", ret);
        return -1;
    }

    ret = snprintf(
        buffer,
        72,
        "This string is way longer and does not fit in the buffer because it %d bytes in length",
        87
    );

    if (!ret) {
        return 0;
    } else {
        printf("Failed! %d", ret);
        return -1;
    }
}
