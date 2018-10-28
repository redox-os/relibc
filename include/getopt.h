#ifndef _GETOPT_H
#define _GETOPT_H

// Generated from:
// `grep "opt" target/include/unistd.h`

#ifdef __cplusplus
extern "C" {
#endif

extern char* optarg;
extern int opterr;
extern int optind;
extern int optopt;
int getopt(int argc, char *const *argv, const char *optstring);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
