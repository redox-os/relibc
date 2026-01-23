#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

// XXX: this is only here because cbindgen can't handle string constants
#define P_tmpdir "/tmp"

typedef struct FILE FILE;

// A typedef doesn't suffice, because libgmp uses this definition to check if
// STDIO was loaded.
#define FILE FILE
// Likewise, stdin, stdout, and stderr are expected to be macros.
#define stdin stdin
#define stdout stdout
#define stderr stderr

#endif /* _BITS_STDIO_H */
