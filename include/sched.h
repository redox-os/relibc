#ifndef _SCHED_H
#define _SCHED_H

#ifdef __cplusplus
extern "C" {
#endif

/* Scheduling algorithms.  */
#define SCHED_OTHER 0
#define SCHED_FIFO  1
#define SCHED_RR    2
#define SCHED_BATCH 3
#define SCHED_IDLE  5
#define SCHED_DEADLINE 6

typedef int pid_t;

struct sched_param {
    int sched_priority;
};

int sched_getparam(pid_t pid, struct sched_param *param);

//int sched_getscheduler(pid_t pid);

#ifdef __cplusplus
}
#endif

#endif