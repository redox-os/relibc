#ifndef _SIGNALS_LIST
#define _SIGNALS_LIST 1

#include <signal.h>

#ifdef SIGSTKFLT
#endif

const int N_SIGNALS = (28
#ifdef SIGSTKFLT
                       + 1
#endif
#ifdef SIGWINCH
                       + 1
#endif
#ifdef SIGIO
                       + 1
#endif
#ifdef SIGPWR
                       + 1
#endif
#ifdef SIGUNUSED
                       + 1
#endif
);

struct signalAction
{
    int signal;
    char action;
};

const struct signalAction signals_list[] = {
    {SIGABRT, 'A'},
    {SIGALRM, 'T'},
    {SIGBUS, 'A'},
    {SIGCHLD, 'I'},
    {SIGCONT, 'C'},
    {SIGFPE, 'A'},
    {SIGHUP, 'T'},
    {SIGILL, 'A'},
    {SIGINT, 'T'},
    {SIGKILL, 'T'},
    {SIGPIPE, 'T'},
    {SIGQUIT, 'A'},
    {SIGSEGV, 'A'},
    {SIGSTOP, 'S'},
    {SIGTERM, 'T'},
    {SIGTSTP, 'S'},
    {SIGTTIN, 'S'},
    {SIGTTOU, 'S'},
    {SIGUSR1, 'T'},
    {SIGUSR2, 'T'},
    // {SIGPOLL, 'T'},
    {SIGPROF, 'T'},
    {SIGSYS, 'A'},
    {SIGTRAP, 'A'},
    {SIGURG, 'I'},
    {SIGVTALRM, 'T'},
    {SIGXCPU, 'A'},
    {SIGXFSZ, 'A'},
#ifdef SIGSTKFLT
    {SIGSTKFLT, 'T'},
#endif
#ifdef SIGWINCH
    {SIGWINCH, 'I'},
#endif
#ifdef SIGIO
    {SIGIO, 'T'},
#endif
#ifdef SIGPWR
    {SIGPWR, 'T'},
#endif
#ifdef SIGUNUSED
    {SIGUNUSED, 'A'},
#endif
};

#endif /* _SIGNALS_LIST */
