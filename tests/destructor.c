#include <stdio.h>
#include "test_helpers.h"

__attribute__((destructor))
void destructor_no_priority(void) {
    puts("destructor (no priority)");
}

#define TEST(__priority)                          \
    __attribute__((destructor(__priority)))       \
    void destructor_priority_##__priority(void) { \
        puts("destructor ("#__priority")");       \
    }

TEST(101)
TEST(102)
TEST(103)
TEST(104)

int main(void) {
    puts("main");
}
