// These tests are primarily to ensure the macros compile without 
// causing any funny business.

#include <features.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

__deprecated
static void legacy(void) {}

__deprecatedNote("Sometimes deletes user's home (oops); use foobar")
static void legacy_notes(void) {}

__nodiscard
static uint8_t the_answer(void) {
    return 42;
}

// GCC bug
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wattributes"
__noreturn
static void foobar(void) {
    exit(0);
}
#pragma GCC diagnostic pop

int main(void) {
    #pragma GCC diagnostic push
    #pragma GCC diagnostic ignored "-Wdeprecated-declarations"
    legacy();
    legacy_notes();
    #pragma GCC diagnostic pop
    const int answer = the_answer();
    char buf[40] = {0};
    sprintf(buf, "Hey, -Werror, I'm using answer: %d\n", answer);
    foobar();
}
