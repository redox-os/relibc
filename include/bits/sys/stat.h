#ifndef _BITS_STAT_H
#define _BITS_STAT_H

#define S_ISDIR(mode)  (((mode) & S_IFMT) == S_IFDIR)
#define S_ISCHR(mode)  (((mode) & S_IFMT) == S_IFCHR)
#define S_ISBLK(mode)  (((mode) & S_IFMT) == S_IFBLK)
#define S_ISREG(mode)  (((mode) & S_IFMT) == S_IFREG)
#define S_ISFIFO(mode) (((mode) & S_IFMT) == S_IFIFO)
#define S_ISLNK(mode)  (((mode) & S_IFMT) == S_IFLNK)
#define S_ISSOCK(mode) (((mode) & S_IFMT) == S_IFSOCK)

#define st_atime st_atim.tv_sec
#define st_mtime st_mtim.tv_sec
#define st_ctime st_ctim.tv_sec

#if defined (_LARGEFILE64_SOURCE)
#define fstat64(int fd, struct stat* buf) fstat(fd, buf)
#define lstat64(const char* path, struct stat* buf) lstat(path, buf)
#define stat64(const char* path, struct stat* buf) stat(path, buf)
#endif

#endif
