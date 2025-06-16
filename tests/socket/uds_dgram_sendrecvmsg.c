#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <string.h>
#include <assert.h>
#include <errno.h>
#include <fcntl.h> // For open()

/*
 * [Normal Case Test - SCM_CREDENTIALS]
 * Verifies sending and receiving process credentials.
 */
void test_pass_credentials_normal() {
    printf("--- Running Normal Case: test_pass_credentials_normal ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }

    int on = 1;
    if (setsockopt(sv[1], SOL_SOCKET, SO_PASSCRED, &on, sizeof(on)) < 0) { exit(1); }

    struct msghdr msg_send = {0};
    struct iovec iov_send[1];
    char send_buf[] = "hello credential";
    iov_send[0].iov_base = send_buf;
    iov_send[0].iov_len = sizeof(send_buf);
    msg_send.msg_iov = iov_send;
    msg_send.msg_iovlen = 1;
    if (sendmsg(sv[0], &msg_send, 0) < 0) { exit(1); }

    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[32];
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;

    char cmsg_buf[CMSG_SPACE(sizeof(struct ucred))];
    msg_recv.msg_control = cmsg_buf;
    msg_recv.msg_controllen = sizeof(cmsg_buf);

    if (recvmsg(sv[1], &msg_recv, 0) < 0) { exit(1); }
    
    assert(strcmp(send_buf, recv_buf) == 0);
    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg_recv);
    assert(cmsg != NULL);
    assert(cmsg->cmsg_level == SOL_SOCKET);
    assert(cmsg->cmsg_type == SCM_CREDENTIALS);

    struct ucred *cred = (struct ucred *)CMSG_DATA(cmsg);
    printf("Received credentials: pid=%d, uid=%d, gid=%d\n", cred->pid, cred->uid, cred->gid);
    assert(cred->pid == getpid());
    assert(cred->uid == getuid());
    assert(cred->gid == getgid());
    printf("Credentials verified successfully.\n");

    close(sv[0]);
    close(sv[1]);
    printf("--- Normal Case Finished ---\n\n");
}

/*
 * [Normal Case Test - SCM_RIGHTS]
 * Verifies sending and receiving a single file descriptor.
 */
void test_pass_fd_normal() {
    printf("--- Running Normal Case: test_pass_fd_normal (SCM_RIGHTS) ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }

    // Create a temporary file to send its descriptor
    const char* file_path = "test_fd_passing.tmp";
    int fd_to_send = open(file_path, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    if (fd_to_send < 0) { perror("open"); exit(1); }
    const char* file_content = "hello fd";
    write(fd_to_send, file_content, strlen(file_content));
    close(fd_to_send); // The kernel holds a reference, so we can close it here.
    fd_to_send = open(file_path, O_RDONLY); // Re-open for sending
    printf("Prepared to send file descriptor %d for file '%s'\n", fd_to_send, file_path);


    struct msghdr msg_send = {0};
    struct iovec iov_send[1];
    char send_buf[] = "here is a file descriptor";
    iov_send[0].iov_base = send_buf;
    iov_send[0].iov_len = sizeof(send_buf);
    msg_send.msg_iov = iov_send;
    msg_send.msg_iovlen = 1;

    char cmsg_buf[CMSG_SPACE(sizeof(int))];
    msg_send.msg_control = cmsg_buf;
    msg_send.msg_controllen = sizeof(cmsg_buf);

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg_send);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int));
    *(int *)CMSG_DATA(cmsg) = fd_to_send;
    
    if (sendmsg(sv[0], &msg_send, 0) < 0) { perror("sendmsg"); exit(1); }
    printf("Message with FD sent.\n");
    close(fd_to_send);

    // Receive side
    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[64];
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;
    
    char cmsg_recv_buf[CMSG_SPACE(sizeof(int))];
    msg_recv.msg_control = cmsg_recv_buf;
    msg_recv.msg_controllen = sizeof(cmsg_recv_buf);

    if (recvmsg(sv[1], &msg_recv, 0) < 0) { perror("recvmsg"); exit(1); }

    struct cmsghdr *cmsg_recv = CMSG_FIRSTHDR(&msg_recv);
    assert(cmsg_recv != NULL);
    assert(cmsg_recv->cmsg_level == SOL_SOCKET);
    assert(cmsg_recv->cmsg_type == SCM_RIGHTS);

    int received_fd = *(int *)CMSG_DATA(cmsg_recv);
    printf("Received file descriptor %d\n", received_fd);
    assert(received_fd >= 0);

    char file_read_buf[32] = {0};
    read(received_fd, file_read_buf, sizeof(file_read_buf) - 1);
    printf("Read from received FD: '%s'\n", file_read_buf);
    assert(strcmp(file_content, file_read_buf) == 0);
    printf("FD is valid and content matches.\n");

    close(received_fd);
    close(sv[0]);
    close(sv[1]);
    remove(file_path);
    printf("--- SCM_RIGHTS Test Finished ---\n\n");
}


/*
 * [Edge Case Test]
 * Verifies sending and receiving multiple file descriptors in one message.
 */
void test_pass_multiple_fds() {
    printf("--- Running Edge Case: test_pass_multiple_fds ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }

    const char* file1 = "multi_fd1.tmp";
    const char* file2 = "multi_fd2.tmp";
    int fds_to_send[2];
    fds_to_send[0] = open(file1, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    fds_to_send[1] = open(file2, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    printf("Prepared to send FDs %d and %d\n", fds_to_send[0], fds_to_send[1]);

    struct msghdr msg_send = {0};
    struct iovec iov_send[1];
    char send_buf[] = "two fds";
    iov_send[0].iov_base = send_buf;
    iov_send[0].iov_len = sizeof(send_buf);
    msg_send.msg_iov = iov_send;
    msg_send.msg_iovlen = 1;

    char cmsg_buf[CMSG_SPACE(sizeof(int) * 2)];
    msg_send.msg_control = cmsg_buf;
    msg_send.msg_controllen = sizeof(cmsg_buf);

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg_send);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int) * 2);
    memcpy(CMSG_DATA(cmsg), fds_to_send, sizeof(int) * 2);
    
    if (sendmsg(sv[0], &msg_send, 0) < 0) { perror("sendmsg"); exit(1); }
    close(fds_to_send[0]);
    close(fds_to_send[1]);

    // Receive side
    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[64];
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;
    
    char cmsg_recv_buf[CMSG_SPACE(sizeof(int) * 2)];
    msg_recv.msg_control = cmsg_recv_buf;
    msg_recv.msg_controllen = sizeof(cmsg_recv_buf);

    if (recvmsg(sv[1], &msg_recv, 0) < 0) { perror("recvmsg"); exit(1); }

    struct cmsghdr *cmsg_recv = CMSG_FIRSTHDR(&msg_recv);
    assert(cmsg_recv != NULL && cmsg_recv->cmsg_type == SCM_RIGHTS);

    int received_fds[2];
    memcpy(received_fds, CMSG_DATA(cmsg_recv), sizeof(int) * 2);
    printf("Received FDs: %d, %d\n", received_fds[0], received_fds[1]);
    assert(received_fds[0] >= 0 && received_fds[1] >= 0);
    printf("Both received FDs are valid.\n");

    close(received_fds[0]);
    close(received_fds[1]);
    close(sv[0]);
    close(sv[1]);
    remove(file1);
    remove(file2);
    printf("--- Multiple FDs Test Finished ---\n\n");
}


/*
 * [Abnormal Case Test]
 * MSG_CTRUNC: Control message buffer is too small.
 */
void test_control_buffer_truncation() {
    printf("--- Running Abnormal Case: test_control_buffer_truncation (MSG_CTRUNC) ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }

    int on = 1;
    if (setsockopt(sv[1], SOL_SOCKET, SO_PASSCRED, &on, sizeof(on)) < 0) { exit(1); }
    
    char send_buf[] = "ctrunc test";
    if (send(sv[0], send_buf, sizeof(send_buf), 0) < 0) { exit(1); }
    
    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[32];
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;

    char cmsg_buf[1]; // Intentionally small buffer
    msg_recv.msg_control = cmsg_buf;
    msg_recv.msg_controllen = sizeof(cmsg_buf);

    if (recvmsg(sv[1], &msg_recv, 0) < 0) { exit(1); }

    assert(msg_recv.msg_flags & MSG_CTRUNC);
    printf("Verified that MSG_CTRUNC flag is set as expected.\n");
    
    close(sv[0]);
    close(sv[1]);
    printf("--- MSG_CTRUNC Test Finished ---\n\n");
}


/*
 * [Abnormal Case Test]
 * MSG_TRUNC: Data buffer is too small.
 */
void test_data_buffer_truncation() {
    printf("--- Running Abnormal Case: test_data_buffer_truncation (MSG_TRUNC) ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }
    
    const char* long_message = "This is a very long message that is guaranteed to be truncated.";
    if (send(sv[0], long_message, strlen(long_message), 0) < 0) { exit(1); }

    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[10]; // Intentionally small buffer
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;
    
    if (recvmsg(sv[1], &msg_recv, 0) < 0) { exit(1); }

    assert(msg_recv.msg_flags & MSG_TRUNC);
    printf("Verified that MSG_TRUNC flag is set as expected.\n");

    close(sv[0]);
    close(sv[1]);
    printf("--- MSG_TRUNC Test Finished ---\n\n");
}


/*
 * [Combination Test]
 * Verifies receiving credentials when sender uses simple `send()`.
 */
void test_simple_send_with_recvmsg_credentials() {
    printf("--- Running Combination Case: simple send() with recvmsg() ---\n");
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv) < 0) { exit(1); }

    int on = 1;
    if (setsockopt(sv[1], SOL_SOCKET, SO_PASSCRED, &on, sizeof(on)) < 0) { exit(1); }

    const char* message = "Simple send, complex receive!";
    if (send(sv[0], message, strlen(message), 0) < 0) { exit(1); }
    
    struct msghdr msg_recv = {0};
    struct iovec iov_recv[1];
    char recv_buf[64];
    iov_recv[0].iov_base = recv_buf;
    iov_recv[0].iov_len = sizeof(recv_buf);
    msg_recv.msg_iov = iov_recv;
    msg_recv.msg_iovlen = 1;

    char cmsg_buf[CMSG_SPACE(sizeof(struct ucred))];
    msg_recv.msg_control = cmsg_buf;
    msg_recv.msg_controllen = sizeof(cmsg_buf);

    if (recvmsg(sv[1], &msg_recv, 0) < 0) { exit(1); }
    
    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg_recv);
    assert(cmsg != NULL && cmsg->cmsg_type == SCM_CREDENTIALS);
    printf("Verified credentials were received even with simple send().\n");

    close(sv[0]);
    close(sv[1]);
    printf("--- Combination Case Finished ---\n\n");
}


int main() {
    test_pass_credentials_normal();
    test_pass_fd_normal();
    test_pass_multiple_fds();
    test_control_buffer_truncation();
    test_data_buffer_truncation();
    test_simple_send_with_recvmsg_credentials();
    
    printf("All tests completed successfully.\n");
    return 0;
}
