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
    void (*sig_dfl)(int) __attribute__((unused)) = SIG_DFL;
    void (*sig_err)(int) __attribute__((unused)) = SIG_ERR;
    void (*sig_ign)(int) __attribute__((unused)) = SIG_IGN;
    
     pthread_t pthread_num __attribute__((unused)) = 0;
    size_t size __attribute__((unused)) = 0;
     uid_t uid __attribute__((unused)) = 0;

     sig_atomic_t atomic __attribute__((unused)) = 0;
     sigset_t sig_set __attribute__((unused)) ;
     pid_t pid __attribute__((unused)) = 0;

     pthread_attr_t *attr __attribute__((unused)) = NULL;

    // struct sigevent sev;
     union sigval sv;

    sv.sival_int = (int)0;
    sv.sival_ptr = (void *)0;

    // sev.sigev_notify = (int)0;
    // sev.sigev_notify = SIGEV_NONE;
    // sev.sigev_notify = SIGEV_SIGNAL;
    // sev.sigev_notify = SIGEV_THREAD;

    // sev.sigev_signo = (int)0;
    // sev.sigev_value = sv;
    // sev.sigev_value.sival_int = (int)0;
    // sev.sigev_value.sival_ptr = (void *)0;
    // sev.sigev_notify_function = (void (*)(union sigval))0;
    // sev.sigev_notify_attributes = (pthread_attr_t *)0;

    int rt_sig_num __attribute__((unused)) = SIGRTMIN;
     rt_sig_num = SIGRTMAX;
    
    // rt_sig_num = SIG2STR_MAX;

    // sev.sigev_signo = SIGABRT;
    // sev.sigev_signo = SIGALRM;
    // sev.sigev_signo = SIGBUS;
    // sev.sigev_signo = SIGCHLD;
    // sev.sigev_signo = SIGCONT;
    // sev.sigev_signo = SIGFPE;
    // sev.sigev_signo = SIGHUP;
    // sev.sigev_signo = SIGILL;
    // sev.sigev_signo = SIGINT;
    // sev.sigev_signo = SIGKILL;
    // sev.sigev_signo = SIGPIPE;
    // sev.sigev_signo = SIGQUIT;
    // sev.sigev_signo = SIGSEGV;
    // sev.sigev_signo = SIGSTOP;
    // sev.sigev_signo = SIGTERM;
    // sev.sigev_signo = SIGTSTP;
    // sev.sigev_signo = SIGTTIN;
    // sev.sigev_signo = SIGTTOU;
    // sev.sigev_signo = SIGUSR1;
    // sev.sigev_signo = SIGUSR2;
    // sev.sigev_signo = SIGWINCH;
    // sev.sigev_signo = SIGPOLL; 
    // sev.sigev_signo = SIGPROF; 
    // sev.sigev_signo = SIGSYS;
    // sev.sigev_signo = SIGTRAP;
    // sev.sigev_signo = SIGURG;
    // sev.sigev_signo = SIGVTALRM;
    // sev.sigev_signo = SIGXCPU;
    // sev.sigev_signo = SIGXFSZ;

    struct sigaction sa;

    sa.sa_handler  = SIG_IGN;
    sa.sa_handler = SIG_DFL;
    sa.sa_handler = handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = (int)0;
    sa.sa_sigaction = action;

#ifndef SA_NOCLDSTOP
#error "Required constant not defined SA_NOCLDSTOP"
#endif
#ifndef SIG_BLOCK
#error "Required constant not defined SIG_BLOCK"
#endif
#ifndef SIG_UNBLOCK
#error "Required constant not defined SIG_UNBLOCK"
#endif
#ifndef SIG_SETMASK
#error "Required constant not defined SIG_SETMASK"
#endif
#ifndef SA_ONSTACK
#error "Required constant not defined SA_ONSTACK"
#endif
#ifndef SA_RESETHAND
#error "Required constant not defined SA_RESETHAND"
#endif
#ifndef SA_RESTART
#error "Required constant not defined SA_RESTART"
#endif
#ifndef SA_SIGINFO
#error "Required constant not defined SA_SIGINFO"
#endif
#ifndef SA_NOCLDWAIT
#error "Required constant not defined SA_NOCLDWAIT"
#endif
#ifndef SA_NODEFER
#error "Required constant not defined SA_NODEFER"
#endif
#ifndef SS_ONSTACK
#error "Required constant not defined SS_ONSTACK"
#endif
#ifndef SS_DISABLE
#error "Required constant not defined SS_DISABLE"
#endif
#ifndef MINSIGSTKSZ
#error "Required constant not defined MINSIGSTKSZ"
#endif
#ifndef SIGSTKSZ
#error "Required constant not defined SIGSTKSZ"
#endif

    // struct ucontext_t uc;
    mcontext_t mc __attribute__((unused));
    
    // add parts to ucontext_t
    // uc.uc_link = NULL;
    // uc.uc_sigmask = 0;
    // uc.uc_stack = NULL;
    // uc.uc_mcontext = NULL;
    

    stack_t st __attribute__((unused));
    st.ss_sp = NULL;
    st.ss_size = (size_t)0;
    st.ss_flags = (int)0;

    siginfo_t si __attribute__((unused));
    si.si_signo = SIGHUP;
    si.si_code = (int)0;
    si.si_errno = (int)0;
    si.si_pid = (pid_t)0;
    si.si_uid = (uid_t)0;
    si.si_addr = NULL;
    si.si_status = (int)0;
    si.si_value = sv;

    // si.si_code = ILL_ILLOPC;
    // si.si_code = ILL_ILLOPN;
    // si.si_code = ILL_ILLADR;
    // si.si_code = ILL_ILLTRP;
    // si.si_code = ILL_PRVOPC;
    // si.si_code = ILL_PRVREG;
    // si.si_code = ILL_COPROC;
    // si.si_code = ILL_BADSTK;
    // si.si_code = FPE_INTDIV;
    // si.si_code = FPE_INTOVF;
    // si.si_code = FPE_FLTDIV;
    // si.si_code = FPE_FLTOVF;
    // si.si_code = FPE_FLTUND;
    // si.si_code = FPE_FLTRES;
    // si.si_code = FPE_FLTINV;
    // si.si_code = FPE_FLTSUB;
    // si.si_code = SEGV_MAPERR;
    // si.si_code = SEGV_ACCERR;
    // si.si_code = BUS_ADRALN;
    // si.si_code = BUS_ADRERR;
    // si.si_code = BUS_OBJERR;
    // si.si_code = TRAP_BRKPT;
    // si.si_code = TRAP_TRACE;
    // si.si_code = CLD_EXITED;
    // si.si_code = CLD_KILLED;
    // si.si_code = CLD_DUMPED;
    // si.si_code = CLD_TRAPPED;
    // si.si_code = CLD_STOPPED;
    // si.si_code = CLD_CONTINUED;
    // si.si_code = SI_USER;
    // si.si_code = SI_QUEUE;
    // si.si_code = SI_TIMER;
    // si.si_code = SI_ASYNCIO;
    // si.si_code = SI_MESGQ;

    // void (*(*bs)(int, void (*)(int)))(int) = bsd_signal;
    int (*k)(pid_t, int) __attribute__((unused))= kill;
    int (*kpg)(pid_t, int) __attribute__((unused))= killpg;
    void (*psig)(const siginfo_t *, const char *) __attribute__((unused))= psiginfo;
    void (*ps)(int, const char *) __attribute__((unused))= psignal;
    int (*ptk)(pthread_t, int) __attribute__((unused))= pthread_kill;
    int (*ptsm)(int, const sigset_t *, sigset_t *) __attribute__((unused))= pthread_sigmask;
    int (*r)(int) __attribute__((unused))= raise;
    // int (*s2s)(int, char*) = sig2str;
    int (*sact)(int, const struct sigaction *restrict,
                struct sigaction *restrict) __attribute__((unused)) = sigaction;
    int (*sas)(sigset_t *, int) __attribute__((unused))= sigaddset;
    int (*sastk)(const stack_t *restrict, stack_t *restrict) __attribute__((unused))= sigaltstack;
    int (*sds)(sigset_t *, int) __attribute__((unused))= sigdelset;
    int (*ses)(sigset_t *) __attribute__((unused))= sigemptyset;
    int (*sfs)(sigset_t *) __attribute__((unused))= sigfillset;
    int (*ismem)(const sigset_t *, int) __attribute__((unused))= sigismember;
    void ( *(*sgnl)(int, void (*)(int)))(int) __attribute__((unused))= signal;
    int (*pend)(sigset_t *) __attribute__((unused))= sigpending;
    int (*spmsk)(int, const sigset_t *restrict, sigset_t *restrict) __attribute__((unused))= sigprocmask;
    int (*sigq)(pid_t, int, const union sigval) __attribute__((unused))= sigqueue;
    int (*susp)(const sigset_t *) __attribute__((unused))= sigsuspend;
    int (*stmwt)(const sigset_t *restrict, siginfo_t *restrict,
                      const struct timespec *restrict) __attribute__((unused))= sigtimedwait;
    int (*swt)(const sigset_t *restrict, int *restrict) __attribute__((unused))= sigwait;
    int (*swtinfo)(const sigset_t *restrict, siginfo_t *restrict) __attribute__((unused))= sigwaitinfo;
    // int (*str2s)(const char *restrict, int *restrict) = str2sig;

    return EXIT_SUCCESS;
}
