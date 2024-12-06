#include "../test_helpers.h"
#include <signal.h>

/*
 * This is a test to ensure all required items for signal.h are defined.
 * The definitions follow the order described in
 * <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>
 */

void handler(int sig_num)
{
    (void)sig_num;
}

void action(int sig_num, siginfo_t *info, void *context)
{
    (void)sig_num;
    (void)info;
    (void)context;
}

int main()
{
    int (*sh)(int) __attribute__((unused)) = sighold;
    int (*sigig)(int) __attribute__((unused)) = sigignore;
    int (*sigintr)(int, int) __attribute__((unused)) = siginterrupt;
    int (*paws)(int) __attribute__((unused)) = sigpause;
    int (*srls)(int) __attribute__((unused)) = sigrelse;
    void (*(*sset)(int, void (*)(int)))(int) __attribute__((unused)) = sigset;
    
    return EXIT_SUCCESS;
}
