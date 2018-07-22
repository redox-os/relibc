#include <stdio.h>
#include <stdlib.h>

int main() {
    //puts(getenv("SHELL"));
    //puts(getenv("CC"));

    putenv("TEST=It's working!!");

    puts(getenv("TEST"));
}
