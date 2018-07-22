/*
 * The process joins process group 0.
 */
#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdlib.h>

int main( void )
  {
    if( setpgid( getpid(), 0 ) == -1 ) {
        perror( "setpgid" );
    }
    printf( "%d belongs to process group %d\n",
         getpid(), getpgrp() );

    if( setregid(-1, -1) == -1 ) {
        perror( "setregid" );
    }
    printf("%d has egid %d and gid %d\n", getpid(), getegid(), getgid());

    if( setreuid(-1, -1) == -1 ) {
        perror( "setreuid" );
    }
    printf("%d has euid %d and uid %d\n", getpid(), geteuid(), getuid());
    return 0;
  }
