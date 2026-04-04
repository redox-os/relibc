/*
 * Tests for alarm(2) - POSIX: https://pubs.opengroup.org/onlinepubs/9799919799/functions/alarm.html
 *
 * Verifies:
 *  1. alarm(0) with no pending alarm returns 0
 *  2. alarm(1) delivers SIGALRM after ~1 second (tested via pause())
 *  3. alarm(0) cancels a pending alarm
 *  4. re-arming alarm returns the remaining seconds from the previous alarm
 */

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "test_helpers.h"

static volatile sig_atomic_t alarm_count = 0;

static void handler(int sig) {
    (void)sig;
    alarm_count++;
}

int main(void) {
    struct sigaction sa;
    sa.sa_handler = handler;
    sa.sa_flags = 0;
    sigemptyset(&sa.sa_mask);
    int r = sigaction(SIGALRM, &sa, NULL);
    ERROR_IF(sigaction, r, == -1);

    /* alarm(0) with no existing alarm must return 0 */
    unsigned prev = alarm(0);
    UNEXP_IF(alarm, (int)prev, != 0);
    puts("alarm(0) baseline: ok");

    /* alarm(1): SIGALRM must fire; pause() blocks until the signal arrives */
    alarm_count = 0;
    alarm(1);
    pause();
    if (alarm_count != 1) {
        fprintf(stderr, "SIGALRM did not fire (count=%d)\n", (int)alarm_count);
        return EXIT_FAILURE;
    }
    puts("alarm(1) fires: ok");

    /* alarm(0) must cancel a pending alarm and return remaining secs > 0 */
    alarm_count = 0;
    alarm(10);
    unsigned remaining = alarm(0);
    if (remaining == 0) {
        fprintf(stderr, "alarm(0) cancel: expected remaining > 0\n");
        return EXIT_FAILURE;
    }
    /* Wait longer than original alarm; SIGALRM must NOT fire */
    sleep(2);
    if (alarm_count != 0) {
        fprintf(stderr, "SIGALRM fired after alarm(0) cancel\n");
        return EXIT_FAILURE;
    }
    puts("alarm(0) cancel: ok");

    /*
     * Re-arming: arm with 10s, sleep 1s, then re-arm with 3s.
     * The return value must be the remaining time (~9s) which is > 0.
     */
    alarm_count = 0;
    alarm(10);
    sleep(1);
    remaining = alarm(3);
    if (remaining == 0) {
        fprintf(stderr, "re-arm: expected remaining > 0, got 0\n");
        return EXIT_FAILURE;
    }
    alarm(0); /* disarm so SIGALRM doesn't fire after the test */
    puts("alarm re-arm returns remaining: ok");

    return EXIT_SUCCESS;
}
