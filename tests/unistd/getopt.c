#include <unistd.h>
#include <stdio.h>

#include "test_helpers.h"

#define RUN(...) \
    do { \
        optind = 1; \
        optarg = NULL; \
        opterr = 1; \
        optopt = -1; \
        char *args_arr[] = { __VA_ARGS__ }; \
        printf("result: %d\n", runner(sizeof(args_arr) / sizeof(args_arr[0]), args_arr)); \
    } while (0)

int runner(int argc, char *argv[]) {
    int c;
    int bflg = 0, aflg = 0, errflg = 0;
    char *ifile = "";
    char *ofile = "";

    while((c = getopt(argc, argv, ":abf:o:")) != -1) {
        switch(c) {
            case 'a':
                if(bflg)
                    errflg++;
                else
                    aflg++;
                break;
            case 'b':
                if(aflg)
                    errflg++;
                else
                    bflg++;
                break;
            case 'f':
                ifile = optarg;
                break;
            case 'o':
                ofile = optarg;
                break;
            case ':':
                printf("Option -%c requires an operand\n", optopt);
                errflg++;
                break;
            case '?':
                printf("Unrecognized option: -%c\n", optopt);
                errflg++;
        }
    }
    printf("bflg: %d\n", bflg);
    printf("aflg: %d\n", aflg);
    printf("errflg: %d\n", errflg);
    printf("ifile: %s\n", ifile);
    printf("ofile: %s\n", ofile);
    if(errflg) {
        printf("Usage: info goes here\n");
        return 2;
    }
    return 0;
}

int main(int argc, const char *argv[]) {
    RUN("test", "-ao", "arg", "path", "path");
    RUN("test", "-a", "-o", "arg", "path", "path");
    RUN("test", "-o", "arg", "-a", "path", "path");
    RUN("test", "-a", "-o", "arg", "--", "path", "path");
    RUN("test", "-a", "-oarg", "path", "path");
    RUN("test", "-aoarg", "path", "path");
    RUN("test");
}
