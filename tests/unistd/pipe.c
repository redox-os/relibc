//http://www2.cs.uregina.ca/~hamilton/courses/330/notes/unix/pipes/pipes.html
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    int pip[2];
    char instring[20];
    char *outstring = "Hello World!";

    int pipe_status = pipe(pip);
    ERROR_IF(pipe, pipe_status, == -1);
    UNEXP_IF(pipe, pipe_status, != 0);

    int pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        // child: sends message to parent
        // close read end
        int cr = close(pip[0]);
        ERROR_IF(close, cr, == -1);
        UNEXP_IF(close, cr, != 0);

        // send 7 characters in the string, including end-of-string
        int bytes = write(pip[1], outstring, strlen(outstring));
        ERROR_IF(write, bytes, == -1);

        // check result
        if (bytes != strlen(outstring)) {
            fprintf(stderr, "pipe write: %d != %ld\n", bytes, strlen(outstring));
            exit(EXIT_FAILURE);
        }

        // close write end
        int cw = close(pip[1]);
        ERROR_IF(close, cw, == -1);
        UNEXP_IF(close, cw, != 0);

        exit(EXIT_SUCCESS);
    } else {
        // parent: receives message from child
        // close write end
        int cw = close(pip[1]);
        ERROR_IF(close, cw, == -1);
        UNEXP_IF(close, cw, != 0);

        // clear memory
        memset(instring, 0, sizeof(instring));

        // read from the pipe
        int bytes = read(pip[0], instring, sizeof(instring) - 1);
        ERROR_IF(read, bytes, == -1);

        // check result
        if (bytes != strlen(outstring)) {
            fprintf(stderr, "pipe read: %d != %ld\n", bytes, strlen(outstring));
            exit(EXIT_FAILURE);
        } else if (memcmp(instring, outstring, strlen(outstring)) != 0) {
            fprintf(stderr, "pipe read does not match pipe write\n");
            exit(EXIT_FAILURE);
        } else {
            printf("%s\n", instring);
        }

        // close read end
        int cr = close(pip[0]);
        ERROR_IF(close, cr, == -1);
        UNEXP_IF(close, cr, != 0);

        exit(EXIT_SUCCESS);
    }
}
