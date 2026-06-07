#include <assert.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/limits.h>
#include <sched.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

#define ERROR_IF(func, status, condition)                                      \
  do {                                                                         \
    if (status condition) {                                                    \
      fprintf(stderr, "%s:%s:%d: '%s' failed: %s (%d)\n", __FILE__, __func__,  \
              __LINE__, #func, strerror(errno), errno);                        \
      _exit(EXIT_FAILURE);                                                     \
    }                                                                          \
  } while (0)

#define OPTIONALLY_ERROR(function, args, cond)                                 \
  {                                                                            \
    int x = function args;                                                     \
    if (!(x cond)) {                                                           \
      printf("%s failed with error: %s\n", #function, strerror(x));            \
      exit(EXIT_FAILURE);                                                      \
    }                                                                          \
  }

extern char **environ;

int main() {
  pid_t pid = 0;
  char cwd[PATH_MAX];
  int status = 0;
  char *__cwd = getcwd(cwd, PATH_MAX);
  ERROR_IF(getcwd, __cwd, == NULL);
  char target[PATH_MAX];
  strcpy(target, cwd);
  strcat(target, "/hello");

  // TEST: chdir and open
  char *argv1[] = {"ls", target, NULL};
  posix_spawn_file_actions_t fa;
  posix_spawn_file_actions_init(&fa);
  status = mkdir("./hello", S_IRUSR | S_IWUSR);
  ERROR_IF(mkdir, status, != 0);
  OPTIONALLY_ERROR(posix_spawn_file_actions_addchdir, (&fa, "./hello"), == 0);
  OPTIONALLY_ERROR(posix_spawn_file_actions_addopen,
                   (&fa, 2, "./hello.txt", O_CREAT | O_RDWR, S_IRUSR | S_IWUSR),
                   == 0);
  OPTIONALLY_ERROR(posix_spawnp, (&pid, "ls", &fa, NULL, argv1, environ), == 0);
  assert(pid != 0);
  waitpid(pid, NULL, 0);
  OPTIONALLY_ERROR(posix_spawn_file_actions_destroy, (&fa), == 0);

  pid = 0;

  // TEST: close
  argv1[0] = "/usr/bin/ls";
  OPTIONALLY_ERROR(posix_spawn_file_actions_init, (&fa), == 0);
  OPTIONALLY_ERROR(posix_spawn_file_actions_addclose, (&fa, 1), == 0);
  OPTIONALLY_ERROR(posix_spawn,
                   (&pid, "/usr/bin/ls", &fa, NULL, argv1, environ), == 0);
  assert(pid != 0);
  waitpid(pid, NULL, 0);
  OPTIONALLY_ERROR(posix_spawn_file_actions_destroy, (&fa), == 0);
  status = unlink("./hello/hello.txt");
  ERROR_IF(unlink, status, != 0);
  status = rmdir("./hello");
  ERROR_IF(rmdir, status, != 0);
}
