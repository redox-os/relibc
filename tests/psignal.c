#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

int main(void) {
    puts("------psignal------");
    psignal(SIGUSR1, "a prefix");
    puts("------  end  ------");
    puts("------psiginfo-----");
    siginfo_t info = { 0 };
    info.si_code = SI_USER;
    info.si_pid = 42;
    info.si_uid = 1337;
    info.si_addr = (void *)0xdeadbeef;
    info.si_value.sival_ptr = (void *)0xfedface;
    psiginfo(&info, "another prefix");
    puts("------        -----");
}
