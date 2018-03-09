#include <ctype.h>
#include <stdio.h>

struct test_case {
    char c;
    int isalnum;
    int isalpha;
    int isascii;
    int isblank;
    int iscntrl;
    int isdigit;
    int isgraph;
    int islower;
    int isprint;
    int ispunct;
    int isspace;
    int isupper;
    int isxdigit;
} test_cases[] = {
    //     a  a  a  b  c  d  g  l  p  p  s  u  x
    //     l  l  s  l  n  i  r  o  r  u  p  p  d
    //     n  p  c  a  t  g  a  w  i  n  a  p  i
    //     u  h  i  n  r  i  p  e  n  c  c  e  g
    //     m  a  i  k  l  t  h  r  t  t  e  r  i
    { 'A', 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1},
    { 'z', 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0},
    { ' ', 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0},
    { '1', 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1},
    { '9', 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1},
    {0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
};
size_t num_test_cases = sizeof(test_cases)/sizeof(struct test_case);

#define CHECK_TEST(tc, fn, retval) \
    if (fn(tc.c) != tc.fn) { \
        retval = -1; \
        printf("Unexpected result: " #fn "('%c') != %d\n", tc.c, tc.fn); \
    }
int main(int argc, char* argv[]) {
    int i;
    int retval = 0;
    for(i = 0; i < num_test_cases; ++i) {
        struct test_case tc = test_cases[i];
        CHECK_TEST(tc, isalnum, retval);
        CHECK_TEST(tc, isalpha, retval);
        CHECK_TEST(tc, isascii, retval);
        CHECK_TEST(tc, isdigit, retval);
        CHECK_TEST(tc, islower, retval);
        CHECK_TEST(tc, isspace, retval);
        CHECK_TEST(tc, isupper, retval);
    }
    if (!retval) {
        printf("Success: %d\n", retval);
    } else {
        printf("Failure: %d\n", retval);
    }
    return retval;
}
