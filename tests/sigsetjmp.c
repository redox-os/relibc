#include <stdio.h>
#include <setjmp.h>
#include <signal.h>

int main() {
sigjmp_buf jb;
sigset_t set; //this is used to set up the mask for testing
sigset_t set2; //this is used to query the mask
sigprocmask(SIG_SETMASK, 0, &set2); 
printf ("Current process signal mask is: %ld\n", set2);
set = 0|SIGUSR1;
sigprocmask(SIG_BLOCK, &set, 0);
sigprocmask(SIG_SETMASK, 0, &set2); 
printf ("After blocking current process signal mask is: %ld\n", set2);
if (sigsetjmp(jb, 1)) {
printf("Jump done.\n");
sigprocmask(SIG_SETMASK, 0, &set2); 
printf ("After jumping back current process signal mask is: %ld\n", set2);
} else {
printf ("Starting jump\n");
printf ("Saved signal mask in sigjmp_buf is: %ld\n", jb[9]);
set = set|SIGUSR2;
sigprocmask(SIG_BLOCK, &set, 0);
sigprocmask(SIG_SETMASK, 0, &set2); 
printf ("Before jumping back current process signal mask is: %ld\n", set2);
siglongjmp(jb, 1);
}
return 0;
}
