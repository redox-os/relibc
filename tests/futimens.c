#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <stdio.h>
#include <unistd.h>

int main(int argc, char** argv) {
  char temp[] = "/tmp/stattest-XXXXXX";
  const char file[] = "/mkfifo_fifo";
  int len = sizeof(temp) + sizeof(int);
  char* path = calloc(len, sizeof(char));

  if (path == NULL) {
    fprintf(stderr, "Could not allocate: %s\n", strerror(errno));
    exit(1);
  }

  strncat(path, mktemp(temp), sizeof(temp));
  strncat(path, file, sizeof(file));
  if (mkdir(temp, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH) != 0) {
    fprintf(stderr, "mkdir %s: %s\n", temp, strerror(errno));
    exit(1);
  }

  int tmp = open(path, O_CREAT | O_CLOEXEC | O_RDONLY | S_IRWXU | S_IRWXG | S_IRWXO);
  if (tmp == -1) {
    fprintf(stderr, "touch %s: %s\n", path, strerror(errno));
    exit(1);
  }
  if (close(tmp) == -1) {
    fprintf(stderr, "close %s: %s\n", path, strerror(errno));
    exit(1);
  }

  int fd = open(path, 0, 0);
  if (fd == -1) {
    fprintf(stderr, "open %s: %s\n", path, strerror(errno));
    exit(1);
  }

  const struct timespec times[] = { { .tv_sec = 10 }, { .tv_sec = 20 } };
  if (futimens(fd, times) == -1) {
    fprintf(stderr, "futimens: %s\n", strerror(errno));
    exit(1);
  }

  struct stat sb;
  if (stat(path, &sb) != 0) {
    fprintf(stderr, "stat: %s\n", strerror(errno));
    exit(1);
  }
  if (sb.st_mtim.tv_sec != 20 || sb.st_mtim.tv_nsec != 0) {
    fprintf(stderr, "Wrong modified time: %d.%d\n", sb.st_mtim.tv_sec, sb.st_mtim.tv_nsec);
    exit(1);
  }
  // Access times are not flushed to disk, so this check can't be (currently) performed
  /*
   * if (sb.st_atim.tv_sec != 10 || sb.st_atim.tv_nsec != 0) {
   *  fprintf(stderr, "Wrong accessed time: %d.%d\n", sb.st_atim.tv_sec, sb.st_atim.tv_nsec);
   *  exit(1);
   * }
   */
}
