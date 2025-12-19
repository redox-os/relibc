#ifndef _BITS_STDIO_H
#define _BITS_STDIO_H

// XXX: this is only here because cbindgen can't handle string constants
#define P_tmpdir "/tmp"

typedef struct FILE FILE;

// A typedef doesn't suffice, because libgmp uses this definition to check if
// STDIO was loaded.
#define FILE FILE

#if defined (_LARGEFILE64_SOURCE)
#define fgetpos64 fgetpos
#define fopen64 fopen
#define freopen64 freopen
#define fseeko64 fseeko
#define fsetpos64 fsetpos
#define ftello64 ftello
#define tmpfile64 tmpfile
#endif

#endif /* _BITS_STDIO_H */
