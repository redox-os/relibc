#include <getopt.h>
#include <stdio.h>

#include "test_helpers.h"

#define RUN(...) \
    do { \
        optind = 1; \
        optarg = NULL; \
        opterr = 1; \
        optopt = -1; \
        char *args_arr[] = { __VA_ARGS__ }; \
        runner(sizeof(args_arr) / sizeof(char*), args_arr); \
    } while (0)

void runner(int argc, char *argv[]) {
    printf("--- Running:");
    for (int i = 0; i < argc; i += 1) {
        printf(" %s", argv[i]);
    }
    puts("");

    static int flag = 0;

    static struct option long_options[] = {
        {"test0", no_argument, NULL,  1},
        {"test1", no_argument, &flag, 2},
        {"test2", optional_argument, NULL, 3},
        {"test3", required_argument, NULL, 4},
        {NULL, 0, NULL, 5},
    };

    int option_index = 0;
    char c;
    while((c = getopt_long(argc, argv, ":a", long_options, &option_index)) != -1) {
        switch(c) {
            case 'a':
                printf("Option -a with value %s\n", optarg);
                break;
            case ':':
                printf("unrecognized argument: -%c\n", optopt);
                break;
            case '?':
                printf("error: -%c\n", optopt);
                break;
            default:
                printf("getopt_long returned %d, ", c);
                if (flag) {
                    printf("set flag to %d, ", flag);
                    flag = 0;
                }
                printf("argument %s=%s\n", long_options[option_index].name, optarg);
                break;
        }
    }
}

int main(int argc, const char *argv[]) {
    RUN("test", "--test0", "-a");
    RUN("test", "--test1", "-a");
    RUN("test", "--test2", "-a");
    RUN("test", "--test2=arg", "-a");
    RUN("test", "--test3", "-a");
    RUN("test", "--test3=arg", "-a");
    RUN("test", "--test3", "arg", "-a");
}
