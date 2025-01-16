#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

// The raise() function shall send the signal sig to the executing [CX] [Option Start]  thread or process. [Option End] If a signal handler is called, the raise() function shall not return until after the signal handler does.

// [CX] [Option Start] The effect of the raise() function shall be equivalent to calling: pthread_kill(pthread_self(), sig);

void sig_hand(int i)

{
   if (i < 1 || i > 32){
      printf("an invalid signal was given: %d", i);
   }
   static int count = 1;          

   count++;
   printf("%d \n", count);
   if (count == 32) { 
      printf("reached 32nd signal\n");
      return;
   }
   else{
      printf("count is %d\n", count);
   }
}

void raise_test(int sig){
   signal(sig, sig_hand);
   raise(sig);
}

int main(void)
{
   for (int i = 0; i < N_SIGNALS; i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
		raise_test(sig);
	}
	return EXIT_SUCCESS;
}                            

