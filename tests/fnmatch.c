#include <fnmatch.h>
#include <stdio.h>

#include "test_helpers.h"

void test(char* pattern, char* input, int flags) {
    if (!fnmatch(pattern, input, flags)) {
        printf("\"%s\" matches \"%s\"\n", pattern, input);
    } else {
        printf("\"%s\" doesn't match \"%s\"\n", pattern, input);
    }
}

int main(void) {
    puts("Should succeed:");
    test("*World", "Hello World", 0);
    test("*World", "World", 0);
    test("Hello*", "Hello World", 0);
    test("H[ae]llo?World", "Hallo+World", 0);
    test("H[ae]llo?World", "Hello_World", 0);
    test("[0-9][!a]", "1b", 0);
    test("/a/*/d", "/a/b/c/d", 0);
    test("/a/*/d", "/a/bc/d", FNM_PATHNAME);
    test("*hello", ".hello", 0);
    test("/*hello", "/.hello", FNM_PERIOD);
    test("[a!][a!]", "!a", 0);
    test("[\\]]", "]", 0);
    test("[\\\\]", "\\", 0);
    test("hello[/+]world", "hello/world", 0);
    test("hello world", "HELLO WORLD", FNM_CASEFOLD);

    puts("");
    puts("Should fail:");
    test("*World", "Hello Potato", 0);
    test("*World", "Potato", 0);
    test("H[ae]llo?World", "Hillo+World", 0);
    test("H[ae]llo?World", "Hello__World", 0);
    test("[0-9][!a]", "ab", 0);
    test("[0-9][!a]", "2a", 0);
    test("/a/*/d", "/a/b/c/d", FNM_PATHNAME);
    test("/a/*/d", "/a/bc/d/", FNM_PATHNAME);
    test("*hello", ".hello", FNM_PERIOD);
    test("/*hello", "/.hello", FNM_PERIOD | FNM_PATHNAME);
    test("[a!][a!]", "ab", 0);
    test("hello[/+]world", "hello/world", FNM_PATHNAME);
    test("hello world", "HELLO WORLD", 0);
}
