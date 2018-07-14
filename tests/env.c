#include <stdio.h>
#include <stdlib.h>

int main() {
    puts(getenv("SHELL"));
    puts(getenv("CC"));

    putenv("KEK=lol");

    puts(getenv("KEK"));
}
