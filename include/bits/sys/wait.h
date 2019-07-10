#ifndef _BITS_SYS_WAIT_H
#define _BITS_SYS_WAIT_H

#define WEXITSTATUS(s) (((s) >> 8) & 0xff)
#define WTERMSIG(s) (((s) & 0x7f) != 0)
#define WSTOPSIG(s) WEXITSTATUS(s)
#define WCOREDUMP(s) (((s) & 0x80) != 0)
#define WIFEXITED(s) (((s) & 0x7f) == 0)
#define WIFSTOPPED(s) (((s) & 0xff) == 0x7f)
#define WIFSIGNALED(s) (((((s) & 0x7f) + 1U) & 0x7f) >= 2) // Ends with 1111111 or 10000000
#define WIFCONTINUED(s) ((s) == 0xffff)

#endif /* _BITS_SYS_WAIT_H */
