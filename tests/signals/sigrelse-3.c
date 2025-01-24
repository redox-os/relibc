#include <stdio.h>
#include <signal.h>
#include <stdlib.h>
#include <errno.h>
#include "../test_helpers.h"

int main()
{
        int status;
        status = (int)sigrelse(100000);
        ERROR_IF(sigrelse, status, != -1);
        ERROR_IF(sigrelse, errno, != EINVAL);
        return EXIT_SUCCESS;
}