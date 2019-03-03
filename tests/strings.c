#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <strings.h>

#include "test_helpers.h"

int main(void) {
    assert(!bcmp("hello", "hehe", 2));
    assert(bcmp("hello", "haha", 2));

    char* new = malloc(3);
    bcopy("hi", new, 3); // include nul byte

    assert(!strcasecmp("hi", new));
    assert(strcasecmp("he", new));

    assert(strcasecmp("hello", "HEllO") == 0);
    assert(strcasecmp("hello", "HEllOo") < 0);
    assert(strcasecmp("5", "5") == 0);
    assert(strcasecmp("5", "4") > 0);
    assert(strcasecmp("5", "6") < 0);
    assert(strncasecmp("hello", "Hello World", 5) == 0);
    assert(strncasecmp("FLOOR0_1", "FLOOR0_1FLOOR4_1", 8) == 0);
    assert(strncasecmp("FL00RO_1", "FLOOR0_1FLOOR4_1", 8) < 0);

    // Ensure we aren't relying on the 5th (lowercase) bit on non-alpha characters
    assert(strcasecmp("{[", "[{") > 0);

    bzero(new, 1);
    assert(*new == 0);
    assert(*(new+1) == 'i');
    assert(*(new+2) == 0);

    assert(ffs(1) == 1);
    assert(ffs(2) == 2);
    assert(ffs(3) == 1);
    assert(ffs(10) == 2);

    char* str = "hihih";
    assert(index(str, 'i') == str + 1);
    assert(rindex(str, 'i') == str + 3);
}
