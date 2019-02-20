//http://www2.cs.uregina.ca/~hamilton/courses/330/notes/unix/pipes/pipes.html
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(void) {
    int pid, pip[2];
    char instring[20];
    char * outstring = "Hello World!";

    if (pipe(pip) < 0) {
        perror("pipe");
        return 1;
    }

    pid = fork();
    if (pid == 0)           /* child : sends message to parent*/
    {
        /* close read end */
        close(pip[0]);

        /* send 7 characters in the string, including end-of-string */
        int bytes = write(pip[1], outstring, strlen(outstring));

        /* close write end */
        close(pip[1]);

        /* check result */
        if (bytes < 0) {
            perror("pipe write");
            return 1;
        } else if (bytes != strlen(outstring)) {
            fprintf(stderr, "pipe write: %d != %ld\n", bytes, strlen(outstring));
            return 1;
        }

        return 0;
    }
    else			/* parent : receives message from child */
    {
        /* close write end */
        close(pip[1]);

        /* clear memory */
        memset(instring, 0, sizeof(instring));

        /* read from the pipe */
        int bytes = read(pip[0], instring, sizeof(instring) - 1);

        /* close read end */
        close(pip[0]);

        /* check result */
        if (bytes < 0) {
            perror("pipe read");
            return 1;
        } else if (bytes != strlen(outstring)) {
            fprintf(stderr, "pipe read: %d != %ld\n", bytes, strlen(outstring));
            return 1;
        } else if (memcmp(instring, outstring, strlen(outstring)) != 0) {
            fprintf(stderr, "pipe read does not match pipe write\n");
            return 1;
        } else {
            printf("%s\n", instring);
        }

        return 0;
    }
    return 0;
}
