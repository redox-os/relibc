#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

// XXX: this is only here because cbindgen can't handle string constants
#define P_tmpdir "/tmp"

typedef struct FILE FILE;

// A typedef doesn't suffice, because libgmp uses this definition to check if
// STDIO was loaded.
#define FILE FILE

#endif /* _BITS_STDIO_H */
