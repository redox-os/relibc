//http://www2.cs.uregina.ca/~hamilton/courses/330/notes/unix/pipes/pipes.html
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main()
{

    int pid, pip[2];
    char instring[20];
    char * outstring = "Hello World!";

    pipe(pip);

    pid = fork();
    if (pid == 0)           /* child : sends message to parent*/
    {
        puts("Child: Close Read");
        /* close read end */
        close(pip[0]);
        puts("Child: Write");
        /* send 7 characters in the string, including end-of-string */
        write(pip[1], outstring, strlen(outstring));
        puts("Child: Close Write");
        /* close write end */
        close(pip[1]);
    }
    else			/* parent : receives message from child */
    {
        puts("Parent: Close Write");
        /* close write end */
        close(pip[1]);
        puts("Parent: Read");
        /* read from the pipe */
        read(pip[0], instring, 7);
        puts("Parent: Close Read");
        /* close read end */
        close(pip[0]);
    }
    return 0;
}
