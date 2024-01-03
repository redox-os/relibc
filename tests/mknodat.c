#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <stdio.h>

int main(void) {
  char temp[] = "/tmp/stattest-XXXXXX";
  const char separator[] = "/";
  const char file[] = "mknod"; // relative
  int len = sizeof(temp) + sizeof(file) + sizeof(separator);
  char* path = malloc(len * sizeof(char));

  if (path == NULL) {
    fprintf(stderr, "Could not allocate: %s\n", strerror(errno));
    exit(1);
  }

  if(!mktemp(temp)) {
    fprintf(stderr, "Unable to create a unique dir name %s: %s\n", temp, strerror(errno));
    exit(1);
  }

  if (mkdir(temp, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH) != 0) {
    fprintf(stderr, "mkdir %s: %s\n", temp, strerror(errno));
    exit(1);
  }

  int dir_fd = open(temp, O_RDONLY);
  if(dir_fd == -1) {
    fprintf(stderr, "unable to open temp directory: %s with error: %s\n", temp, strerror(errno));
    exit(1);
  }

  if (mknodat(dir_fd, file, S_IFREG, S_IRUSR) == -1) {
    fprintf(stderr, "mknod %s: %s\n", path, strerror(errno));
    exit(1);
  }

  path = strncat(path, temp, strlen(temp));
  path = strncat(path, separator, strlen(temp));
  path = strncat(path, file, strlen(file));

  struct stat sb;
  if (stat(path, &sb) != 0) {
    fprintf(stderr, "stat for %s: %s\n", path, strerror(errno));
    exit(1);
  }

  if (!(sb.st_mode & S_IFREG)) {
    fprintf(stderr, "Expected S_IFREG flag to be set, got mode: %d\n", sb.st_mode);
    exit(1);
  }
}
