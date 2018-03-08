//http://www2.cs.uregina.ca/~hamilton/courses/330/notes/unix/pipes/pipes.html
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
        /* close read end */
        close(pip[0]);
        /* send 7 characters in the string, including end-of-string */
        write(pip[1], outstring, strlen(outstring));
        /* close write end */
        close(pip[1]);
    }
    else			/* parent : receives message from child */
    {
        /* close write end */
        close(pip[1]);
        /* read from the pipe */
        read(pip[0], instring, 7);
        /* close read end */
        close(pip[0]);
    }
}
