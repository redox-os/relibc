#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    int status = system("echo test of system");
    ERROR_IF(system, status, == -1);
}
