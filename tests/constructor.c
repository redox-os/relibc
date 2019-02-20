#include <stdio.h>

__attribute__((constructor))
void constructor_no_priority(void) {
    puts("constructor (no priority)");
}

#define TEST(__priority)                           \
    __attribute__((constructor(__priority)))       \
    void constructor_priority_##__priority(void) { \
        puts("constructor ("#__priority")");       \
    }

TEST(101)
TEST(102)
TEST(103)
TEST(104)

int main(int argc, char *argv[]) {
    puts("main");
}
