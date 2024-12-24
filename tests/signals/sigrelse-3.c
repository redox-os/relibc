#define _XOPEN_SOURCE 700

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>
#include <errno.h>

int main()
{
        int status;
        status = (int)sigrelse(100000);
        ERROR_IF(sigrelse, status, != -1);
        ERROR_IF(sigrelse, errno, != EINVAL);
        return EXIT_SUCCESS;
	// if ((int)sigrelse(100000) == -1) {

        //         if (EINVAL == errno) {
        //                         printf ("errno set to EINVAL\n");
        //                         return EXIT_SUCCESS;
        //                 } else {
        //                         printf ("errno not set to EINVAL\n");
        //                         exit(EXIT_FAILURE);
        //                 }
	// } 
}