#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#include <string.h>
#include <signal.h>
#include <errno.h>

#include "test_helpers.h"

void scm_rights_test(int flags) {
    int sv[2]; 
    pid_t pid;

    char iov_buf[] = "foo";

    int status = socketpair(AF_UNIX, flags, 0, sv);
    ERROR_IF(socketpair, status, == -1);

    pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        close(sv[0]); 

        struct msghdr msg;
        memset(&msg, 0, sizeof(msg));
        memset(iov_buf, 0, sizeof(iov_buf));

        struct iovec iov[1];
        iov[0].iov_base = iov_buf;
        iov[0].iov_len = sizeof(iov_buf);
        msg.msg_iov = iov;
        msg.msg_iovlen = 1;

        char cmsg_buf[CMSG_SPACE(sizeof(int))];
        memset(cmsg_buf, 0, sizeof(cmsg_buf));
        msg.msg_control = cmsg_buf;
        msg.msg_controllen = sizeof(cmsg_buf);

        ssize_t n = recvmsg(sv[1], &msg, 0);
        ERROR_IF(recvmsg, n, == -1);

        struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
        if (cmsg != NULL && cmsg->cmsg_level == SOL_SOCKET && cmsg->cmsg_type == SCM_RIGHTS) {
            int received_fd;
            memcpy(&received_fd, CMSG_DATA(cmsg), sizeof(int));
            printf("recv fd %d and msg '%s'\n", received_fd, iov_buf);

            status = close(received_fd);
            ERROR_IF(close_received_fd, status, == -1);
        } else {
            printf("recv invalid msg\n");
            exit(1);
        }

        close(sv[1]);
        exit(0);
    } else {
        close(sv[1]); 

        int dummy_fds[2];
        status = pipe(dummy_fds);
        ERROR_IF(pipe, status, == -1);
        
        int fd_to_send = dummy_fds[0];
        printf("send fd %d\n", fd_to_send);

        struct msghdr msg;
        memset(&msg, 0, sizeof(msg));

        struct iovec iov[1];
        iov[0].iov_base = iov_buf;
        iov[0].iov_len = sizeof(iov_buf);
        msg.msg_iov = iov;
        msg.msg_iovlen = 1;

        char cmsg_buf[CMSG_SPACE(sizeof(int))];
        memset(cmsg_buf, 0, sizeof(cmsg_buf));
        msg.msg_control = cmsg_buf;
        msg.msg_controllen = sizeof(cmsg_buf);

        struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
        cmsg->cmsg_level = SOL_SOCKET;
        cmsg->cmsg_type = SCM_RIGHTS;
        cmsg->cmsg_len = CMSG_LEN(sizeof(int));
        memcpy(CMSG_DATA(cmsg), &fd_to_send, sizeof(int));

        ssize_t n = sendmsg(sv[0], &msg, 0);
        ERROR_IF(sendmsg, n, == -1);

        close(dummy_fds[0]);
        close(dummy_fds[1]);
        close(sv[0]);
        wait(NULL); 
    }
}

int main(void) {
    printf("SOCK_STREAM\n");
    scm_rights_test(SOCK_STREAM);
    
    printf("SOCK_DGRAM\n");
    scm_rights_test(SOCK_DGRAM);
    
    return 0;
}