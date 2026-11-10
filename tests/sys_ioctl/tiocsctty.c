#define _XOPEN_SOURCE 800

#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/ioctl.h>
#include <sys/wait.h>
#include <string.h>

#include "../test_helpers.h"

void set_controlling_terminal(int subsidiaryfd, char *subsidiarydevice)
{
    char controllingterm[L_ctermid];

    // get the process id and the session id
    pid_t pid = getpid();
    pid_t session_leader = getsid((pid_t)0);
    printf("Process ID is %d, Session leader is %d\n", pid, session_leader);

    // make the child a session
    pid_t session = setsid();
    ERROR_IF(setsid, session, == -1);

    // pid and session should be equal, but it's not required by the POSIX standard
    printf("pid %d, session %d\n", pid, session);

    // set the controlling terminal
    int success = ioctl(subsidiaryfd, TIOCSCTTY, 0);
    ERROR_IF(ioctl, success, == -1);

    // get the name of the controlling terminal
    char *name = ctermid((char *)controllingterm);
    ERROR_IF(ctermid, name, != (char *)controllingterm);

    // compare against parent's name for pt
    int diff = strncmp(subsidiarydevice, controllingterm, L_ctermid);
    ERROR_IF(strncmp, diff, != 0);

    printf("Controlling term is %s\n", controllingterm);
    exit(EXIT_SUCCESS);
}

void make_pty_test(void)
{
    pid_t child_id;

    // Create pt
    int managerfd, subsidiaryfd;
    char *subsidiarydevice;

    managerfd = posix_openpt(O_RDWR | O_NOCTTY);
    ERROR_IF(posix_openpt, managerfd, == -1);
    grantpt(managerfd);
    ERROR_IF(grantpt, managerfd, == -1);
    unlockpt(managerfd);
    ERROR_IF(unlockpt, managerfd, == -1);
    subsidiarydevice = ptsname(managerfd);
    ERROR_IF(ptsname, subsidiarydevice, == NULL);

    // get the name of the subsidiary
    printf("subsidiary device is: %s\n", subsidiarydevice);

    subsidiaryfd = open(subsidiarydevice, O_RDWR | O_NOCTTY);
    ERROR_IF(open, subsidiaryfd, == -1);

    // create a child that is the subsidiary side

    child_id = fork();
    ERROR_IF(fork, child_id, == -1);
    if (child_id == 0)
    {
        set_controlling_terminal(subsidiaryfd, subsidiarydevice);
        exit(EXIT_SUCCESS);
    }
    else
    {
        waitpid(child_id, 0, 0);
        exit(EXIT_SUCCESS);
    }
}

int main(void)
{
    make_pty_test();
}
