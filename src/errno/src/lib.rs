//! errno implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/errno.h.html

#![no_std]

extern crate platform;

#[repr(C)]
pub enum Errno {
    // Argument list too long
    E2BIG = 1,
    // Permission denied
    EACCES,
    // Address in use
    EADDRINUSE,
    // Address not available
    EADDRNOTAVAIL,
    // Address family not supported
    EAFNOSUPPORT,
    // Resource unavailable, try again (may be the same value as [EWOULDBLOCK])
    EAGAIN,
    // Connection already in progress
    EALREADY,
    // Bad file descriptor
    EBADF,
    // Bad message
    EBADMSG,
    // Device or resource busy
    EBUSY,
    // Operation canceled
    ECANCELED,
    // No child processes
    ECHILD,
    // Connection aborted
    ECONNABORTED,
    // Connection refused
    ECONNREFUSED,
    // Connection reset
    ECONNRESET,
    // Resource deadlock would occur
    EDEADLK,
    // Destination address required
    EDESTADDRREQ,
    // Mathematics argument out of domain of function
    EDOM,
    // Reserved
    EDQUOT,
    // File exists
    EEXIST,
    // Bad address
    EFAULT,
    // File too large
    EFBIG,
    // Host is unreachable
    EHOSTUNREACH,
    // Identifier removed
    EIDRM,
    // Illegal byte sequence
    EILSEQ,
    // Operation in progress
    EINPROGRESS,
    // Interrupted function
    EINTR,
    // Invalid argument
    EINVAL,
    // I/O error
    EIO,
    // Socket is connected
    EISCONN,
    // Is a directory
    EISDIR,
    // Too many levels of symbolic links
    ELOOP,
    // Too many open files
    EMFILE,
    // Too many links
    EMLINK,
    // Message too large
    EMSGSIZE,
    // Reserved
    EMULTIHOP,
    // Filename too long
    ENAMETOOLONG,
    // Network is down
    ENETDOWN,
    // Connection aborted by network
    ENETRESET,
    // Network unreachable
    ENETUNREACH,
    // Too many files open in system
    ENFILE,
    // No buffer space available
    ENOBUFS,
    // No message is available on the STREAM head read queue
    ENODATA,
    // No such device
    ENODEV,
    // No such file or directory
    ENOENT,
    // Executable file format error
    ENOEXEC,
    // No locks available
    ENOLCK,
    // Reserved
    ENOLINK,
    // Not enough space
    ENOMEM,
    // No message of the desired type
    ENOMSG,
    // Protocol not available
    ENOPROTOOPT,
    // No space left on device
    ENOSPC,
    // No STREAM resources
    ENOSR,
    // Not a STREAM
    ENOSTR,
    // Function not supported
    ENOSYS,
    // The socket is not connected
    ENOTCONN,
    // Not a directory
    ENOTDIR,
    // Directory not empty
    ENOTEMPTY,
    // Not a socket
    ENOTSOCK,
    // Not supported
    ENOTSUP,
    // Inappropriate I/O control operation
    ENOTTY,
    // No such device or address
    ENXIO,
    // Operation not supported on socket
    EOPNOTSUPP,
    // Value too large to be stored in data type
    EOVERFLOW,
    // Operation not permitted
    EPERM,
    // Broken pipe
    EPIPE,
    // Protocol error
    EPROTO,
    // Protocol not supported
    EPROTONOSUPPORT,
    // Protocol wrong type for socket
    EPROTOTYPE,
    // Result too large
    ERANGE,
    // Read-only file system
    EROFS,
    // Invalid seek
    ESPIPE,
    // No such process
    ESRCH,
    // Reserved
    ESTALE,
    // Stream ioctl() timeout
    ETIME,
    // Connection timed out
    ETIMEDOUT,
    // Text file busy
    ETXTBSY,
    // Operation would block (may be the same value as [EAGAIN])
    EWOULDBLOCK,
    // Cross-device link
    EXDEV,
}
