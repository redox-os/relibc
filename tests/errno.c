#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include "test_helpers.h"

int main(int argc, char **argv) {
    // Recent POSIX requires E2BIG, EACCESS etc. to be available as macros
#ifdef E2BIG
    puts("E2BIG macro available");
#else
    puts("E2BIG macro missing");
#endif /* E2BIG */
#ifdef EACCES
    puts("EACCES macro available");
#else
    puts("EACCES macro missing");
#endif /* EACCES */
#ifdef EADDRINUSE
    puts("EADDRINUSE macro available");
#else
    puts("EADDRINUSE macro missing");
#endif /* EADDRINUSE */
#ifdef EADDRNOTAVAIL
    puts("EADDRNOTAVAIL macro available");
#else
    puts("EADDRNOTAVAIL macro missing");
#endif /* EADDRNOTAVAIL */
#ifdef EADV
    puts("EADV macro available");
#else
    puts("EADV macro missing");
#endif /* EADV */
#ifdef EAFNOSUPPORT
    puts("EAFNOSUPPORT macro available");
#else
    puts("EAFNOSUPPORT macro missing");
#endif /* EAFNOSUPPORT */
#ifdef EAGAIN
    puts("EAGAIN macro available");
#else
    puts("EAGAIN macro missing");
#endif /* EAGAIN */
#ifdef EALREADY
    puts("EALREADY macro available");
#else
    puts("EALREADY macro missing");
#endif /* EALREADY */
#ifdef EBADE
    puts("EBADE macro available");
#else
    puts("EBADE macro missing");
#endif /* EBADE */
#ifdef EBADF
    puts("EBADF macro available");
#else
    puts("EBADF macro missing");
#endif /* EBADF */
#ifdef EBADFD
    puts("EBADFD macro available");
#else
    puts("EBADFD macro missing");
#endif /* EBADFD */
#ifdef EBADMSG
    puts("EBADMSG macro available");
#else
    puts("EBADMSG macro missing");
#endif /* EBADMSG */
#ifdef EBADR
    puts("EBADR macro available");
#else
    puts("EBADR macro missing");
#endif /* EBADR */
#ifdef EBADRQC
    puts("EBADRQC macro available");
#else
    puts("EBADRQC macro missing");
#endif /* EBADRQC */
#ifdef EBADSLT
    puts("EBADSLT macro available");
#else
    puts("EBADSLT macro missing");
#endif /* EBADSLT */
#ifdef EBFONT
    puts("EBFONT macro available");
#else
    puts("EBFONT macro missing");
#endif /* EBFONT */
#ifdef EBUSY
    puts("EBUSY macro available");
#else
    puts("EBUSY macro missing");
#endif /* EBUSY */
#ifdef ECANCELED
    puts("ECANCELED macro available");
#else
    puts("ECANCELED macro missing");
#endif /* ECANCELED */
#ifdef ECHILD
    puts("ECHILD macro available");
#else
    puts("ECHILD macro missing");
#endif /* ECHILD */
#ifdef ECHRNG
    puts("ECHRNG macro available");
#else
    puts("ECHRNG macro missing");
#endif /* ECHRNG */
#ifdef ECOMM
    puts("ECOMM macro available");
#else
    puts("ECOMM macro missing");
#endif /* ECOMM */
#ifdef ECONNABORTED
    puts("ECONNABORTED macro available");
#else
    puts("ECONNABORTED macro missing");
#endif /* ECONNABORTED */
#ifdef ECONNREFUSED
    puts("ECONNREFUSED macro available");
#else
    puts("ECONNREFUSED macro missing");
#endif /* ECONNREFUSED */
#ifdef ECONNRESET
    puts("ECONNRESET macro available");
#else
    puts("ECONNRESET macro missing");
#endif /* ECONNRESET */
#ifdef EDEADLK
    puts("EDEADLK macro available");
#else
    puts("EDEADLK macro missing");
#endif /* EDEADLK */
#ifdef EDEADLOCK
    puts("EDEADLOCK macro available");
#else
    puts("EDEADLOCK macro missing");
#endif /* EDEADLOCK */
#ifdef EDESTADDRREQ
    puts("EDESTADDRREQ macro available");
#else
    puts("EDESTADDRREQ macro missing");
#endif /* EDESTADDRREQ */
#ifdef EDOM
    puts("EDOM macro available");
#else
    puts("EDOM macro missing");
#endif /* EDOM */
#ifdef EDOTDOT
    puts("EDOTDOT macro available");
#else
    puts("EDOTDOT macro missing");
#endif /* EDOTDOT */
#ifdef EDQUOT
    puts("EDQUOT macro available");
#else
    puts("EDQUOT macro missing");
#endif /* EDQUOT */
#ifdef EEXIST
    puts("EEXIST macro available");
#else
    puts("EEXIST macro missing");
#endif /* EEXIST */
#ifdef EFAULT
    puts("EFAULT macro available");
#else
    puts("EFAULT macro missing");
#endif /* EFAULT */
#ifdef EFBIG
    puts("EFBIG macro available");
#else
    puts("EFBIG macro missing");
#endif /* EFBIG */
#ifdef EHOSTDOWN
    puts("EHOSTDOWN macro available");
#else
    puts("EHOSTDOWN macro missing");
#endif /* EHOSTDOWN */
#ifdef EHOSTUNREACH
    puts("EHOSTUNREACH macro available");
#else
    puts("EHOSTUNREACH macro missing");
#endif /* EHOSTUNREACH */
#ifdef EIDRM
    puts("EIDRM macro available");
#else
    puts("EIDRM macro missing");
#endif /* EIDRM */
#ifdef EILSEQ
    puts("EILSEQ macro available");
#else
    puts("EILSEQ macro missing");
#endif /* EILSEQ */
#ifdef EINPROGRESS
    puts("EINPROGRESS macro available");
#else
    puts("EINPROGRESS macro missing");
#endif /* EINPROGRESS */
#ifdef EINTR
    puts("EINTR macro available");
#else
    puts("EINTR macro missing");
#endif /* EINTR */
#ifdef EINVAL
    puts("EINVAL macro available");
#else
    puts("EINVAL macro missing");
#endif /* EINVAL */
#ifdef EIO
    puts("EIO macro available");
#else
    puts("EIO macro missing");
#endif /* EIO */
#ifdef EISCONN
    puts("EISCONN macro available");
#else
    puts("EISCONN macro missing");
#endif /* EISCONN */
#ifdef EISDIR
    puts("EISDIR macro available");
#else
    puts("EISDIR macro missing");
#endif /* EISDIR */
#ifdef EISNAM
    puts("EISNAM macro available");
#else
    puts("EISNAM macro missing");
#endif /* EISNAM */
#ifdef EKEYEXPIRED
    puts("EKEYEXPIRED macro available");
#else
    puts("EKEYEXPIRED macro missing");
#endif /* EKEYEXPIRED */
#ifdef EKEYREJECTED
    puts("EKEYREJECTED macro available");
#else
    puts("EKEYREJECTED macro missing");
#endif /* EKEYREJECTED */
#ifdef EKEYREVOKED
    puts("EKEYREVOKED macro available");
#else
    puts("EKEYREVOKED macro missing");
#endif /* EKEYREVOKED */
#ifdef EL2HLT
    puts("EL2HLT macro available");
#else
    puts("EL2HLT macro missing");
#endif /* EL2HLT */
#ifdef EL2NSYNC
    puts("EL2NSYNC macro available");
#else
    puts("EL2NSYNC macro missing");
#endif /* EL2NSYNC */
#ifdef EL3HLT
    puts("EL3HLT macro available");
#else
    puts("EL3HLT macro missing");
#endif /* EL3HLT */
#ifdef EL3RST
    puts("EL3RST macro available");
#else
    puts("EL3RST macro missing");
#endif /* EL3RST */
#ifdef ELIBACC
    puts("ELIBACC macro available");
#else
    puts("ELIBACC macro missing");
#endif /* ELIBACC */
#ifdef ELIBBAD
    puts("ELIBBAD macro available");
#else
    puts("ELIBBAD macro missing");
#endif /* ELIBBAD */
#ifdef ELIBEXEC
    puts("ELIBEXEC macro available");
#else
    puts("ELIBEXEC macro missing");
#endif /* ELIBEXEC */
#ifdef ELIBMAX
    puts("ELIBMAX macro available");
#else
    puts("ELIBMAX macro missing");
#endif /* ELIBMAX */
#ifdef ELIBSCN
    puts("ELIBSCN macro available");
#else
    puts("ELIBSCN macro missing");
#endif /* ELIBSCN */
#ifdef ELNRNG
    puts("ELNRNG macro available");
#else
    puts("ELNRNG macro missing");
#endif /* ELNRNG */
#ifdef ELOOP
    puts("ELOOP macro available");
#else
    puts("ELOOP macro missing");
#endif /* ELOOP */
#ifdef EMEDIUMTYPE
    puts("EMEDIUMTYPE macro available");
#else
    puts("EMEDIUMTYPE macro missing");
#endif /* EMEDIUMTYPE */
#ifdef EMFILE
    puts("EMFILE macro available");
#else
    puts("EMFILE macro missing");
#endif /* EMFILE */
#ifdef EMLINK
    puts("EMLINK macro available");
#else
    puts("EMLINK macro missing");
#endif /* EMLINK */
#ifdef EMSGSIZE
    puts("EMSGSIZE macro available");
#else
    puts("EMSGSIZE macro missing");
#endif /* EMSGSIZE */
#ifdef EMULTIHOP
    puts("EMULTIHOP macro available");
#else
    puts("EMULTIHOP macro missing");
#endif /* EMULTIHOP */
#ifdef ENAMETOOLONG
    puts("ENAMETOOLONG macro available");
#else
    puts("ENAMETOOLONG macro missing");
#endif /* ENAMETOOLONG */
#ifdef ENAVAIL
    puts("ENAVAIL macro available");
#else
    puts("ENAVAIL macro missing");
#endif /* ENAVAIL */
#ifdef ENETDOWN
    puts("ENETDOWN macro available");
#else
    puts("ENETDOWN macro missing");
#endif /* ENETDOWN */
#ifdef ENETRESET
    puts("ENETRESET macro available");
#else
    puts("ENETRESET macro missing");
#endif /* ENETRESET */
#ifdef ENETUNREACH
    puts("ENETUNREACH macro available");
#else
    puts("ENETUNREACH macro missing");
#endif /* ENETUNREACH */
#ifdef ENFILE
    puts("ENFILE macro available");
#else
    puts("ENFILE macro missing");
#endif /* ENFILE */
#ifdef ENOANO
    puts("ENOANO macro available");
#else
    puts("ENOANO macro missing");
#endif /* ENOANO */
#ifdef ENOBUFS
    puts("ENOBUFS macro available");
#else
    puts("ENOBUFS macro missing");
#endif /* ENOBUFS */
#ifdef ENOCSI
    puts("ENOCSI macro available");
#else
    puts("ENOCSI macro missing");
#endif /* ENOCSI */
#ifdef ENODATA
    puts("ENODATA macro available");
#else
    puts("ENODATA macro missing");
#endif /* ENODATA */
#ifdef ENODEV
    puts("ENODEV macro available");
#else
    puts("ENODEV macro missing");
#endif /* ENODEV */
#ifdef ENOENT
    puts("ENOENT macro available");
#else
    puts("ENOENT macro missing");
#endif /* ENOENT */
#ifdef ENOEXEC
    puts("ENOEXEC macro available");
#else
    puts("ENOEXEC macro missing");
#endif /* ENOEXEC */
#ifdef ENOKEY
    puts("ENOKEY macro available");
#else
    puts("ENOKEY macro missing");
#endif /* ENOKEY */
#ifdef ENOLCK
    puts("ENOLCK macro available");
#else
    puts("ENOLCK macro missing");
#endif /* ENOLCK */
#ifdef ENOLINK
    puts("ENOLINK macro available");
#else
    puts("ENOLINK macro missing");
#endif /* ENOLINK */
#ifdef ENOMEDIUM
    puts("ENOMEDIUM macro available");
#else
    puts("ENOMEDIUM macro missing");
#endif /* ENOMEDIUM */
#ifdef ENOMEM
    puts("ENOMEM macro available");
#else
    puts("ENOMEM macro missing");
#endif /* ENOMEM */
#ifdef ENOMSG
    puts("ENOMSG macro available");
#else
    puts("ENOMSG macro missing");
#endif /* ENOMSG */
#ifdef ENONET
    puts("ENONET macro available");
#else
    puts("ENONET macro missing");
#endif /* ENONET */
#ifdef ENOPKG
    puts("ENOPKG macro available");
#else
    puts("ENOPKG macro missing");
#endif /* ENOPKG */
#ifdef ENOPROTOOPT
    puts("ENOPROTOOPT macro available");
#else
    puts("ENOPROTOOPT macro missing");
#endif /* ENOPROTOOPT */
#ifdef ENOSPC
    puts("ENOSPC macro available");
#else
    puts("ENOSPC macro missing");
#endif /* ENOSPC */
#ifdef ENOSR
    puts("ENOSR macro available");
#else
    puts("ENOSR macro missing");
#endif /* ENOSR */
#ifdef ENOSTR
    puts("ENOSTR macro available");
#else
    puts("ENOSTR macro missing");
#endif /* ENOSTR */
#ifdef ENOSYS
    puts("ENOSYS macro available");
#else
    puts("ENOSYS macro missing");
#endif /* ENOSYS */
#ifdef ENOTBLK
    puts("ENOTBLK macro available");
#else
    puts("ENOTBLK macro missing");
#endif /* ENOTBLK */
#ifdef ENOTCONN
    puts("ENOTCONN macro available");
#else
    puts("ENOTCONN macro missing");
#endif /* ENOTCONN */
#ifdef ENOTDIR
    puts("ENOTDIR macro available");
#else
    puts("ENOTDIR macro missing");
#endif /* ENOTDIR */
#ifdef ENOTEMPTY
    puts("ENOTEMPTY macro available");
#else
    puts("ENOTEMPTY macro missing");
#endif /* ENOTEMPTY */
#ifdef ENOTNAM
    puts("ENOTNAM macro available");
#else
    puts("ENOTNAM macro missing");
#endif /* ENOTNAM */
#ifdef ENOTRECOVERABLE
    puts("ENOTRECOVERABLE macro available");
#else
    puts("ENOTRECOVERABLE macro missing");
#endif /* ENOTRECOVERABLE */
#ifdef ENOTSOCK
    puts("ENOTSOCK macro available");
#else
    puts("ENOTSOCK macro missing");
#endif /* ENOTSOCK */
#ifdef ENOTSUP
    puts("ENOTSUP macro available");
#else
    puts("ENOTSUP macro missing");
#endif /* ENOTSUP */
#ifdef ENOTTY
    puts("ENOTTY macro available");
#else
    puts("ENOTTY macro missing");
#endif /* ENOTTY */
#ifdef ENOTUNIQ
    puts("ENOTUNIQ macro available");
#else
    puts("ENOTUNIQ macro missing");
#endif /* ENOTUNIQ */
#ifdef ENXIO
    puts("ENXIO macro available");
#else
    puts("ENXIO macro missing");
#endif /* ENXIO */
#ifdef EOPNOTSUPP
    puts("EOPNOTSUPP macro available");
#else
    puts("EOPNOTSUPP macro missing");
#endif /* EOPNOTSUPP */
#ifdef EOVERFLOW
    puts("EOVERFLOW macro available");
#else
    puts("EOVERFLOW macro missing");
#endif /* EOVERFLOW */
#ifdef EOWNERDEAD
    puts("EOWNERDEAD macro available");
#else
    puts("EOWNERDEAD macro missing");
#endif /* EOWNERDEAD */
#ifdef EPERM
    puts("EPERM macro available");
#else
    puts("EPERM macro missing");
#endif /* EPERM */
#ifdef EPFNOSUPPORT
    puts("EPFNOSUPPORT macro available");
#else
    puts("EPFNOSUPPORT macro missing");
#endif /* EPFNOSUPPORT */
#ifdef EPIPE
    puts("EPIPE macro available");
#else
    puts("EPIPE macro missing");
#endif /* EPIPE */
#ifdef EPROTO
    puts("EPROTO macro available");
#else
    puts("EPROTO macro missing");
#endif /* EPROTO */
#ifdef EPROTONOSUPPORT
    puts("EPROTONOSUPPORT macro available");
#else
    puts("EPROTONOSUPPORT macro missing");
#endif /* EPROTONOSUPPORT */
#ifdef EPROTOTYPE
    puts("EPROTOTYPE macro available");
#else
    puts("EPROTOTYPE macro missing");
#endif /* EPROTOTYPE */
#ifdef ERANGE
    puts("ERANGE macro available");
#else
    puts("ERANGE macro missing");
#endif /* ERANGE */
#ifdef EREMCHG
    puts("EREMCHG macro available");
#else
    puts("EREMCHG macro missing");
#endif /* EREMCHG */
#ifdef EREMOTE
    puts("EREMOTE macro available");
#else
    puts("EREMOTE macro missing");
#endif /* EREMOTE */
#ifdef EREMOTEIO
    puts("EREMOTEIO macro available");
#else
    puts("EREMOTEIO macro missing");
#endif /* EREMOTEIO */
#ifdef ERESTART
    puts("ERESTART macro available");
#else
    puts("ERESTART macro missing");
#endif /* ERESTART */
#ifdef EROFS
    puts("EROFS macro available");
#else
    puts("EROFS macro missing");
#endif /* EROFS */
#ifdef ESHUTDOWN
    puts("ESHUTDOWN macro available");
#else
    puts("ESHUTDOWN macro missing");
#endif /* ESHUTDOWN */
#ifdef ESOCKTNOSUPPORT
    puts("ESOCKTNOSUPPORT macro available");
#else
    puts("ESOCKTNOSUPPORT macro missing");
#endif /* ESOCKTNOSUPPORT */
#ifdef ESPIPE
    puts("ESPIPE macro available");
#else
    puts("ESPIPE macro missing");
#endif /* ESPIPE */
#ifdef ESRCH
    puts("ESRCH macro available");
#else
    puts("ESRCH macro missing");
#endif /* ESRCH */
#ifdef ESRMNT
    puts("ESRMNT macro available");
#else
    puts("ESRMNT macro missing");
#endif /* ESRMNT */
#ifdef ESTALE
    puts("ESTALE macro available");
#else
    puts("ESTALE macro missing");
#endif /* ESTALE */
#ifdef ESTRPIPE
    puts("ESTRPIPE macro available");
#else
    puts("ESTRPIPE macro missing");
#endif /* ESTRPIPE */
#ifdef ETIME
    puts("ETIME macro available");
#else
    puts("ETIME macro missing");
#endif /* ETIME */
#ifdef ETIMEDOUT
    puts("ETIMEDOUT macro available");
#else
    puts("ETIMEDOUT macro missing");
#endif /* ETIMEDOUT */
#ifdef ETOOMANYREFS
    puts("ETOOMANYREFS macro available");
#else
    puts("ETOOMANYREFS macro missing");
#endif /* ETOOMANYREFS */
#ifdef ETXTBSY
    puts("ETXTBSY macro available");
#else
    puts("ETXTBSY macro missing");
#endif /* ETXTBSY */
#ifdef EUCLEAN
    puts("EUCLEAN macro available");
#else
    puts("EUCLEAN macro missing");
#endif /* EUCLEAN */
#ifdef EUNATCH
    puts("EUNATCH macro available");
#else
    puts("EUNATCH macro missing");
#endif /* EUNATCH */
#ifdef EUSERS
    puts("EUSERS macro available");
#else
    puts("EUSERS macro missing");
#endif /* EUSERS */
#ifdef EWOULDBLOCK
    puts("EWOULDBLOCK macro available");
#else
    puts("EWOULDBLOCK macro missing");
#endif /* EWOULDBLOCK */
#ifdef EXDEV
    puts("EXDEV macro available");
#else
    puts("EXDEV macro missing");
#endif /* EXDEV */
#ifdef EXFULL
    puts("EXFULL macro available");
#else
    puts("EXFULL macro missing");
#endif /* EXFULL */

    assert(argc > 0);

    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);

    argv[0] = "changed to argv[0]";
    program_invocation_name = "changed to program_invocation_name";
    program_invocation_short_name = "changed to program_invocation_short_name";

    puts(argv[0]);
    puts(program_invocation_name);
    puts(program_invocation_short_name);
}
