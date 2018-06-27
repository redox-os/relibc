#include <assert.h>
#include <stdlib.h>
#include <stdio.h>

int main() {
    assert(1 == 1);
    assert(1 + 1 == 2);

    puts("yay!");

    //This fails, but I can't test it because that'd
    //make the test fail lol
    //assert(42 == 1337);
}
