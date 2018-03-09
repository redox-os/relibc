#ifndef _STAT_H
#define _STAT_H

#include <sys/types.h>

struct stat {
  dev_t st_dev;
  ino_t st_ino;
  nlink_t st_nlink;
  mode_t st_mode;
  uid_t st_uid;
  gid_t st_gid;
  dev_t st_rdev;
  off_t st_size;
  blksize_t st_blksize;
  time_t st_atim;
  time_t st_mtim;
  time_t st_ctim;
};

int chmod(const char *path, mode_t mode);

int fchmod(int fildes, mode_t mode);

int fstat(int fildes, struct stat *buf);

int lstat(const char *path, struct stat *buf);

int mkdir(const char *path, mode_t mode);

int mkfifo(const char *path, mode_t mode);

int mknod(const char *path, mode_t mode, dev_t dev);

int stat(const char *file, struct stat *buf);

mode_t umask(mode_t mask);

#endif /* _STAT_H */
