#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>

int check_mtime(char *path, int expected_sec, int expected_nsec, int err_gap) {
  // Checks whether the moditication time of the file located at *path* match the provided times.
  // When err_gap is set, only checks for a match on sec with a margin of error of +/- err_gap.
  struct stat sb;
  if (stat(path, &sb) != 0) {
    fprintf(stderr, "stat: %s\n", strerror(errno));
    return -1;
  }
  if (err_gap > 0) {
    if (sb.st_mtim.tv_sec < expected_sec + err_gap && sb.st_mtim.tv_sec > expected_sec - err_gap) {
      return 0;
    }
  } else {
    if (sb.st_mtim.tv_sec == expected_sec && sb.st_mtim.tv_nsec == expected_nsec) {
      return 0;
    }
  }
  fprintf(stderr, "Wrong modified time: %d.%d\n", sb.st_mtim.tv_sec, sb.st_mtim.tv_nsec);
  return -1;
}


int main(void) {
  char temp[] = "/tmp/stattest-XXXXXX";
  const char file[] = "/mkfifo_fifo";
  int len = sizeof(temp) + sizeof(int);
  char* path = calloc(len, sizeof(char));

  if (path == NULL) {
    fprintf(stderr, "Could not allocate: %s\n", strerror(errno));
    exit(1);
  }

  #pragma GCC diagnostic push
  #pragma GCC diagnostic ignored "-Wdeprecated-declarations"
  strncat(path, mktemp(temp), sizeof(temp));
  #pragma GCC diagnostic pop
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
  if (check_mtime(path, 20, 0, 0) != 0) {
    exit(1);
  }
  // Access times are not flushed to disk, so atime checks can't be (currently) performed

  const struct timespec omit_times[] = { { 25, UTIME_OMIT }, { 12, UTIME_OMIT } };
  if (futimens(fd, omit_times) == -1) {
    fprintf(stderr, "futimens: %s\n", strerror(errno));
    exit(1);
  }
  if (check_mtime(path, 20, 0, 0) != 0) {
    exit(1);
  }

  const struct timespec now_times[] = { { 25, UTIME_NOW }, { 12, UTIME_NOW } };
  if (futimens(fd, now_times) == -1) {
    fprintf(stderr, "futimens: %s\n", strerror(errno));
    exit(1);
  }
  int now_ts = time(NULL);
  if (check_mtime(path, now_ts, 0, 1) != 0) {
    exit(1);
  }
}

