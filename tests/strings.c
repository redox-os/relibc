#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <strings.h>

int main() {
    assert(!bcmp("hello", "hehe", 2));
    assert(bcmp("hello", "haha", 2));

    char* new = malloc(3);
    bcopy("hi", new, 3); // include nul byte

    assert(!strcasecmp("hi", new));
    assert(!strcasecmp("hello", "HEllO"));
    assert(!strncasecmp("hello", "Hello World", 5));

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
