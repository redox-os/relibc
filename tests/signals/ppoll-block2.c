
#include "../test_helpers.h"
#include "signals_list.h"
#include "errno.h"
#include <poll.h>
#include "unistd.h"
#include <stdio.h>
#include <err.h>
#include <fcntl.h>
#include <sys/redox.h>
int main(void)
{
    int fds[2];
    if ( pipe(fds) )
        err(1, "pipe");
    
    close(fds[0]);
    printf("FD is %d\n",fds[0]);
    int fcn_ret = fcntl(fds[0], F_GETFD, 0) ;
   
    printf("=== %s ===\n", "before alarm");
    printf("fcn_ret is %d, errno is %d\n",fcn_ret, errno);
    errno=0;

    //int fd_table = open("/scheme/",O_RDONLY);

    char path[64];
    //struct redox_file_desc *fd_info = (struct redox_file_desc *)buf;
    redox_fpath(fds[1], path, 64);
    //rfpath(
    printf("%s \n",path);
   
    //snprintf(path, sizeof(path), "/scheme/logging", getpid());
    //printf(path);
    
    //char buf[4096];
    //read(fd_table, buf, sizeof(buf));
    //printf(buf);
    errno=0;
/* 

    printf("=== %s ===\n", "before alarm");
    char path[64];
    snprintf(path, sizeof(path), "/scheme/", getpid());
    printf(path);
    DIR *dir = opendir(path);
    if (!dir) { perror("opendir"); return 0; }

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_name[0] == '.') continue;
        printf("  fd %s\n", entry->d_name);
    }
    closedir(dir); */
    alarm(1);
    printf("=== %s ===\n", "after alarm");
/* 
    DIR *dir2 = opendir(path);
   
    if (!dir) { perror("opendir"); return 0; }
    while ((entry = readdir(dir2)) != NULL) {
        if (entry->d_name[0] == '.') continue;
        printf("  fd %s\n", entry->d_name);
    }
    closedir(dir); */
    printf("FD is %d\n",fds[0]);
    fcn_ret = fcntl(fds[0], F_GETFD, 0) ;
    printf("fcn_ret is %d, errno is %d\n",fcn_ret, errno);


    redox_fpath(fds[0], path, 64);
    printf(path);
 /*    //struct pollfd pfd = { .fd = fds[0], .events = POLLIN };
    // POSIX requires EINTR or returning the pending events.
    //int ret = ppoll(&pfd, 1, NULL, &empty);
    //if ( ret < 0 )
        //   ERROR_IF(ppoll, ret, <0);
    if ( !ret )
    {
        printf("ppoll() == 0\n");
        return 0;
    }
    printf("0");
    if ( pfd.revents & POLLIN )
        ERROR_IF(ppoll, pfd.revents, & POLLIN);
    if ( pfd.revents & POLLOUT )
        ERROR_IF(ppoll, pfd.revents, & POLLOUT);
    if ( pfd.revents & POLLERR )
        ERROR_IF(ppoll, pfd.revents, & POLLERR);
    if ( pfd.revents & POLLHUP )
        ERROR_IF(ppoll, pfd.revents, & POLLHUP);
    if ( pfd.revents & POLLNVAL ){
        printf("\nPOLLNVAL");
        return EXIT_SUCCESS;
    } */
    printf("\n");
    return 0;
}