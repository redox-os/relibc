#include <assert.h>
#include <stdlib.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    assert(1 == 1);
    assert(1 + 1 == 2);
    puts("yay!");

    if (assert(0 == 0), 1) {
        puts("groovy!");
    }

    //This fails, but I can't test it because that'd
    //make the test fail lol
    //assert(42 == 1337);
}
