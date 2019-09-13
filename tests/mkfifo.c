#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <stdio.h>

int main(int argc, char** argv) {
  char temp[] = "/tmp/stattest-XXXXXX";
  const char file[] = "/mkfifo_fifo";
  int len = sizeof(temp) + sizeof(file);
  char* path = malloc(len * sizeof(char));

  if (path == NULL) {
    fprintf(stderr, "Could not allocate: %s\n", strerror(errno));
    exit(1);
  }

  path = strncat(path, mktemp(temp), sizeof(temp));
  path = strncat(path, file, sizeof(file));
  if (mkdir(temp, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH) != 0) {
    fprintf(stderr, "mkdir %s: %s\n", temp, strerror(errno));
    exit(1);
  }
  if (mkfifo(path, S_IRUSR) == -1) {
    fprintf(stderr, "mkfifo %s: %s\n", path, strerror(errno));
    exit(1);
  }
  struct stat sb;
  if (stat(path, &sb) != 0) {
    fprintf(stderr, "stat: %s\n", strerror(errno));
    exit(1);
  }
  if (!(sb.st_mode & S_IFIFO)) {
    fprintf(stderr, "Not a FIFO: %d\n", sb.st_mode);
    exit(1);
  }
}
