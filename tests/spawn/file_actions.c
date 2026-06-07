#include "../test_helpers.h"
#include <assert.h>
#include <dirent.h>
#include <fcntl.h>
#include <limits.h>
#include <sched.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

static const char buf[] = "#include <unistd.h>\nint main()\n{\nwrite(900, "
                          "\"HELLO REDOX\\n\", 12);\n}";

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

  // TEST: dup2
  pid = 0;
  posix_spawn_file_actions_init(&fa);
  posix_spawn_file_actions_adddup2(&fa, 1, 900);

  FILE *f = fopen("./dup2_check.c", "w");
  if (!f) {
    printf("Failed to create dup2_check.c\n");
    exit(EXIT_FAILURE);
  }
  OPTIONALLY_ERROR(fwrite, (buf, sizeof(char), 67, f), == 67);
  fclose(f);
  char *argv2[] = {"gcc", "./dup2_check.c", "-o", "d.out", NULL};
  OPTIONALLY_ERROR(posix_spawnp, (&pid, "gcc", NULL, NULL, argv2, environ),
                   == 0);
  waitpid(pid, NULL, 0);
  assert(pid != 0);
  pid = 0;
  char *argv3[] = {"./d.out", NULL};
  OPTIONALLY_ERROR(posix_spawnp, (&pid, "./d.out", &fa, NULL, argv3, environ),
                   == 0);
  assert(pid != 0);
  waitpid(pid, NULL, 0);
  posix_spawn_file_actions_destroy(&fa);
}
