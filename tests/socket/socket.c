#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <unistd.h>
#include <string.h>
#include <assert.h>
#include <errno.h>
#include <fcntl.h> // For open()

/*
 * [Client/Server Model Test - SOCK_STREAM]
 * Verifies the socket()->bind()->listen()->accept()->connect() workflow for stream sockets.
 * A server process listens for a connection, and a client process connects to it.
 */
void test_client_server_stream() {
    printf("--- Running Client/Server Model Test (SOCK_STREAM) ---\n");
    const char* socket_path = "/tmp/test_socket_stream.sock";


    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        exit(1);
    }

    // --- Server Process ---
    if (pid == 0) {
        int listen_fd, conn_fd;
        struct sockaddr_un addr;
        char buffer[128];

        listen_fd = socket(AF_UNIX, SOCK_STREAM, 0);
        if (listen_fd < 0) { perror("server: socket"); exit(1); }

        memset(&addr, 0, sizeof(addr));
        addr.sun_family = AF_UNIX;
        strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

        if (bind(listen_fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
            perror("server: bind"); exit(1);
        }
        printf("[Server-STREAM] Socket bound to %s\n", socket_path);

        if (listen(listen_fd, 5) < 0) {
            perror("server: listen"); exit(1);
        }
        printf("[Server-STREAM] Listening for connections...\n");

        conn_fd = accept(listen_fd, NULL, NULL);
        if (conn_fd < 0) {
            perror("server: accept"); exit(1);
        }
        printf("[Server-STREAM] Accepted connection.\n");

        ssize_t n = read(conn_fd, buffer, sizeof(buffer) - 1);
        if (n < 0) { perror("server: read"); exit(1); }
        buffer[n] = '\0';
        printf("[Server-STREAM] Received: '%s'\n", buffer);
        assert(strcmp(buffer, "Hello from client") == 0);

        const char* reply = "Hello from server";
        if (write(conn_fd, reply, strlen(reply)) < 0) {
            perror("server: write"); exit(1);
        }
        printf("[Server-STREAM] Sent reply.\n");
        
        close(conn_fd);
        close(listen_fd);
        exit(0); // Success
    } 
    // --- Client Process ---
    else {
        int client_fd;
        struct sockaddr_un addr;
        char buffer[128];

        sleep(1); // Give the server a moment to start up.

        client_fd = socket(AF_UNIX, SOCK_STREAM, 0);
        if (client_fd < 0) { perror("client: socket"); exit(1); }

        memset(&addr, 0, sizeof(addr));
        addr.sun_family = AF_UNIX;
        strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

        if (connect(client_fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
            perror("client: connect"); exit(1);
        }
        printf("[Client-STREAM] Connected to server.\n");
        
        const char* message = "Hello from client";
        if (write(client_fd, message, strlen(message)) < 0) {
            perror("client: write"); exit(1);
        }
        printf("[Client-STREAM] Sent message.\n");
        
        ssize_t n = read(client_fd, buffer, sizeof(buffer) - 1);
        if (n < 0) { perror("client: read"); exit(1); }
        buffer[n] = '\0';
        printf("[Client-STREAM] Received: '%s'\n", buffer);
        assert(strcmp(buffer, "Hello from server") == 0);

        close(client_fd);

        int status;
        waitpid(pid, &status, 0);
        assert(WIFEXITED(status) && WEXITSTATUS(status) == 0);
        printf("[Client-STREAM] Server process finished successfully.\n");
    }
    printf("--- Client/Server Model Test (SOCK_STREAM) Finished ---\n\n");
}

/*
 * [Client/Server Model Test - SOCK_DGRAM]
 * Verifies the socket()->bind()->sendto()/recvfrom() workflow for datagram sockets.
 * A server process waits for a message, and a client process sends one.
 */
void test_client_server_dgram() {
    printf("--- Running Client/Server Model Test (SOCK_DGRAM) ---\n");
    const char* server_path = "/tmp/test_socket_dgram.sock";
    const char* client_path = "/tmp/test_client_dgram.sock";

    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        exit(1);
    }

    // --- Server Process ---
    if (pid == 0) {
        int server_fd;
        struct sockaddr_un server_addr, client_addr;
        char buffer[128];
        socklen_t client_addr_len = sizeof(client_addr);

        server_fd = socket(AF_UNIX, SOCK_DGRAM, 0);
        if (server_fd < 0) { perror("server: socket"); exit(1); }

        memset(&server_addr, 0, sizeof(server_addr));
        server_addr.sun_family = AF_UNIX;
        strncpy(server_addr.sun_path, server_path, sizeof(server_addr.sun_path) - 1);

        if (bind(server_fd, (struct sockaddr*)&server_addr, sizeof(server_addr)) < 0) {
            perror("server: bind"); exit(1);
        }
        printf("[Server-DGRAM] Socket bound to %s\n", server_path);
        
        ssize_t n = recvfrom(server_fd, buffer, sizeof(buffer) - 1, 0, (struct sockaddr*)&client_addr, &client_addr_len);
        if (n < 0) { perror("server: recvfrom"); exit(1); }
        buffer[n] = '\0';
        printf("[Server-DGRAM] Received: '%s' from %s\n", buffer, client_addr.sun_path);
        assert(strcmp(buffer, "Hello from client") == 0);
        
        const char* reply = "Hello from server";
        if (sendto(server_fd, reply, strlen(reply), 0, (struct sockaddr*)&client_addr, client_addr_len) < 0) {
            perror("server: sendto"); exit(1);
        }
        printf("[Server-DGRAM] Sent reply.\n");

        close(server_fd);
        exit(0);
    }
    // --- Client Process ---
    else {
        int client_fd;
        struct sockaddr_un client_addr, server_addr;
        char buffer[128];

        sleep(1);

        client_fd = socket(AF_UNIX, SOCK_DGRAM, 0);
        if (client_fd < 0) { perror("client: socket"); exit(1); }

        memset(&client_addr, 0, sizeof(client_addr));
        client_addr.sun_family = AF_UNIX;
        strncpy(client_addr.sun_path, client_path, sizeof(client_addr.sun_path) - 1);
        if (bind(client_fd, (struct sockaddr*)&client_addr, sizeof(client_addr)) < 0) {
            perror("client: bind"); exit(1);
        }

        memset(&server_addr, 0, sizeof(server_addr));
        server_addr.sun_family = AF_UNIX;
        strncpy(server_addr.sun_path, server_path, sizeof(server_addr.sun_path) - 1);

        const char* message = "Hello from client";
        if (sendto(client_fd, message, strlen(message), 0, (struct sockaddr*)&server_addr, sizeof(server_addr)) < 0) {
            perror("client: sendto"); exit(1);
        }
        printf("[Client-DGRAM] Sent message.\n");
        
        ssize_t n = recvfrom(client_fd, buffer, sizeof(buffer) - 1, 0, NULL, NULL);
        if (n < 0) { perror("client: recvfrom"); exit(1); }
        buffer[n] = '\0';
        printf("[Client-DGRAM] Received: '%s'\n", buffer);
        assert(strcmp(buffer, "Hello from server") == 0);

        close(client_fd);

        int status;
        waitpid(pid, &status, 0);
        assert(WIFEXITED(status) && WEXITSTATUS(status) == 0);
        printf("[Client-DGRAM] Server process finished successfully.\n");
    }

    printf("--- Client/Server Model Test (SOCK_DGRAM) Finished ---\n\n");
}


int main() {
    printf("====== RUNNING CLIENT/SERVER MODEL TESTS ======\n\n");
    test_client_server_stream();
    test_client_server_dgram();
    
    printf("====== ALL CLIENT/SERVER MODEL TESTS COMPLETED SUCCESSFULLY ======\n");
    return 0;
}
