use syscall::{self, O_CLOEXEC, O_STAT, O_CREAT, O_EXCL, O_DIRECTORY, O_WRONLY, O_NOFOLLOW, TimeSpec};
use core::slice;
use libc::{c_int, c_char, off_t, mode_t, size_t, ssize_t};
use ::types::{utimbuf, timeval};

pub const PATH_MAX: usize = 4096;


libc_fn!(unsafe access(path: *mut c_char, _amode: c_int) -> Result<c_int> {
    // XXX amode
    ::RawFile::open(::cstr_to_slice(path), O_CLOEXEC | O_STAT)?;
    Ok(0)
});

libc_fn!(unsafe _close(file: c_int) -> Result<c_int> {
    Ok(syscall::close(file as usize)? as c_int)
});

libc_fn!(unsafe dup(file: c_int) -> Result<c_int> {
    Ok(syscall::dup(file as usize, &[])? as c_int)
});

libc_fn!(unsafe dup2(file: c_int, newfile: c_int) -> Result<c_int> {
    Ok(syscall::dup2(file as usize, newfile as usize, &[])? as c_int)
});

libc_fn!(unsafe _fstat(file: c_int, st: *mut syscall::Stat) -> Result<c_int> {
    Ok(syscall::fstat(file as usize, &mut *st)? as c_int)
});

libc_fn!(unsafe _fsync(file: c_int) -> Result<c_int> {
    Ok(syscall::fsync(file as usize)? as c_int)
});

libc_fn!(unsafe ftruncate(file: c_int, len: off_t) -> Result<c_int> {
    Ok(syscall::ftruncate(file as usize, len as usize)? as c_int)
});

libc_fn!(unsafe _lseek(file: c_int, ptr: off_t, dir: c_int) -> Result<off_t> {
    Ok(syscall::lseek(file as usize, ptr as isize, dir as usize)? as off_t)
});


libc_fn!(unsafe mkdir(path: *mut c_char, mode: mode_t) -> Result<c_int> {
    let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | (mode as usize & 0o777);
    ::RawFile::open(::cstr_to_slice(path), flags)?;
    Ok(0)
});

libc_fn!(unsafe _open(path: *mut c_char, flags: c_int, mode: mode_t) -> Result<c_int> {
    let mut path = ::cstr_to_slice(path);
    // XXX hack; use better method if possible
    if path == b"/dev/null" {
        path = b"null:"
    }
    Ok(syscall::open(path, flags as usize | (mode as usize & 0o777))? as c_int)
});

libc_fn!(unsafe pipe(pipefd: *mut [c_int; 2]) -> c_int {
    pipe2(pipefd, 0)
});

libc_fn!(unsafe pipe2(pipefd: *mut [c_int; 2], flags: c_int) -> Result<c_int> {
    let mut syspipefd = [(*pipefd)[0] as usize, (*pipefd)[1] as usize];
    syscall::pipe2(&mut syspipefd, flags as usize)?;
    (*pipefd)[0] = syspipefd[0] as c_int;
    (*pipefd)[1] = syspipefd[1] as c_int;
    Ok(0)
});

libc_fn!(unsafe _read(file: c_int, buf: *mut c_char, len: c_int) -> Result<c_int> {
    let buf = slice::from_raw_parts_mut(buf as *mut u8, len as usize);
    Ok(syscall::read(file as usize, buf)? as c_int)
});

libc_fn!(unsafe rmdir(path: *mut c_char) -> Result<c_int> {
    Ok(syscall::rmdir(::cstr_to_slice(path))? as c_int)
});

libc_fn!(unsafe _stat(path: *const c_char, st: *mut syscall::Stat) -> Result<c_int> {
    let fd = ::RawFile::open(::cstr_to_slice(path), O_CLOEXEC | O_STAT)?;
    Ok(syscall::fstat(*fd, &mut *st)? as c_int)
});

libc_fn!(unsafe lstat(path: *const c_char, st: *mut syscall::Stat) -> Result<c_int> {
    let fd = ::RawFile::open(::cstr_to_slice(path), O_CLOEXEC | O_STAT | O_NOFOLLOW)?;
    Ok(syscall::fstat(*fd, &mut *st)? as c_int)
});

libc_fn!(unsafe _unlink(path: *mut c_char) -> Result<c_int> {
    Ok(syscall::unlink(::cstr_to_slice(path))? as c_int)
});

libc_fn!(unsafe _write(file: c_int, buf: *const c_char, len: c_int) -> Result<c_int> {
    let buf = slice::from_raw_parts(buf as *const u8, len as usize);
    Ok(syscall::write(file as usize, buf)? as c_int)
});

libc_fn!(unsafe chmod(path: *mut c_char, mode: mode_t) -> Result<c_int> {
    Ok(syscall::chmod(::cstr_to_slice(path), mode as usize)? as c_int)
});

libc_fn!(unsafe realpath(path: *const c_char, resolved_path: *mut c_char) -> Result<*mut c_char> {
    let fd = ::RawFile::open(::cstr_to_slice(path), O_STAT)?;

    let resolved_path = ::MallocNull::new(resolved_path, PATH_MAX);
    let buf = slice::from_raw_parts_mut(resolved_path.as_mut_ptr() as *mut u8, PATH_MAX-1);
    let length = syscall::fpath(*fd, buf)?;
    buf[length] = b'\0';

    Ok(resolved_path.into_raw())
});

libc_fn!(unsafe _rename(old: *const c_char, new: *const c_char) -> Result<c_int> {
    // XXX fix this horror when the kernel provides rename() or link()
    let old = ::cstr_to_slice(old);
    let new = ::cstr_to_slice(new);
    let buf = ::file_read_all(old)?;

    let mut stat = syscall::Stat::default();
    let fd = ::RawFile::open(old, syscall::O_STAT)?;
    syscall::fstat(*fd, &mut stat)?;
    drop(fd);
    let mode = (stat.st_mode & 0o777) as usize;

    let fd = ::RawFile::open(new, O_CREAT | O_WRONLY | mode)?;
    syscall::write(*fd, &buf)?;
    syscall::unlink(old)?;
    Ok(0)
});

libc_fn!(fsync(fd: c_int) -> Result<c_int> {
    Ok(syscall::fsync(fd as usize)? as c_int)
});

libc_fn!(unsafe symlink(path1: *const c_char, path2: *const c_char) -> Result<c_int> {
    let fd = ::RawFile::open(::cstr_to_slice(path2), syscall::O_SYMLINK | syscall::O_CREAT | syscall::O_WRONLY | 0o777)?;
    syscall::write(*fd, ::cstr_to_slice(path1))?;
    Ok(0)
});

libc_fn!(unsafe readlink(path: *const c_char, buf: *const c_char, bufsize: size_t) -> Result<ssize_t> {
    let fd = ::RawFile::open(::cstr_to_slice(path), syscall::O_SYMLINK | syscall::O_RDONLY)?;
    let count = syscall::read(*fd, slice::from_raw_parts_mut(buf as *mut u8, bufsize))?;
    Ok(count as ssize_t)
});

libc_fn!(unsafe utime(path: *mut c_char, times: *const utimbuf) -> Result<c_int> {
    let times = if times.is_null() {
        let mut tp = TimeSpec::default();
        syscall::clock_gettime(syscall::flag::CLOCK_REALTIME, &mut tp)?;
        [tp, tp]
    } else {
        [TimeSpec { tv_sec: (*times).actime, tv_nsec: 0 },
         TimeSpec { tv_sec: (*times).modtime, tv_nsec: 0 }]
    };
    let fd = ::RawFile::open(::cstr_to_slice(path), 0)?;
    syscall::futimens(*fd, &times)?;
    Ok(0)
});

libc_fn!(unsafe utimes(path: *mut c_char, times: *const [timeval; 2]) -> Result<c_int> {
    let times =  [TimeSpec { tv_sec: (*times)[0].tv_sec, tv_nsec: (*times)[0].tv_usec as i32 * 1000 },
                  TimeSpec { tv_sec: (*times)[1].tv_sec, tv_nsec: (*times)[0].tv_usec as i32 * 1000 }];
    let fd = ::RawFile::open(::cstr_to_slice(path), 0)?;
    syscall::futimens(*fd, &times)?;
    Ok(0)
});

libc_fn!(unsafe futimens(fd: c_int, times: *const [TimeSpec; 2]) -> Result<c_int> {
    // XXX UTIME_NOW and UTIME_OMIT (in redoxfs?)
    syscall::futimens(fd as usize, &*times)?;
    Ok(0)
});

// XXX variadic
libc_fn!(_fcntl(file: c_int, cmd: c_int, arg: c_int) -> Result<c_int> {
    Ok(syscall::fcntl(file as usize, cmd as usize, arg as usize)? as c_int)
});

libc_fn!(_isatty(file: c_int) -> c_int {
    if let Ok(fd) = syscall::dup(file as usize, b"termios") {
        let _ = syscall::close(fd);
        1
    } else {
        0
    }
});
