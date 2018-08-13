//! sys/socket implementation, following http://pubs.opengroup.org/onlinepubs/009696699/basedefs/sys/socket.h.html

use alloc::btree_map::BTreeMap;
use core::fmt::Write;
use core::{mem, ptr, slice};
use spin::{Once, Mutex, MutexGuard};
use syscall::data::Stat as redox_stat;
use syscall::data::TimeSpec as redox_timespec;
use syscall::flag::*;
use syscall::{self, Result};

use types::*;
use *;

const EINVAL: c_int = 22;
const MAP_ANON: c_int = 1;

#[thread_local]
static mut SIG_HANDLER: Option<extern "C" fn(c_int)> = None;

static ANONYMOUS_MAPS: Once<Mutex<BTreeMap<usize, usize>>> = Once::new();

fn anonymous_maps() -> MutexGuard<'static, BTreeMap<usize, usize>> {
    ANONYMOUS_MAPS.call_once(|| Mutex::new(BTreeMap::new())).lock()
}

extern "C" fn sig_handler(sig: usize) {
    if let Some(ref callback) = unsafe { SIG_HANDLER } {
        callback(sig as c_int);
    }
}

fn e(sys: Result<usize>) -> usize {
    match sys {
        Ok(ok) => ok,
        Err(err) => {
            unsafe {
                errno = err.errno as c_int;
            }
            !0
        }
    }
}

macro_rules! bind_or_connect {
    (bind $path:expr) => {
        concat!("/", $path)
    };
    (connect $path:expr) => {
        $path
    };
    ($mode:ident $socket:expr, $address:expr, $address_len:expr) => {{
        if (*$address).sa_family as c_int != AF_INET {
            errno = syscall::EAFNOSUPPORT;
            return -1;
        }
        if ($address_len as usize) < mem::size_of::<sockaddr>() {
            errno = syscall::EINVAL;
            return -1;
        }
        let data = &*($address as *const sockaddr_in);
        let addr = &data.sin_addr.s_addr;
        let port = in_port_t::from_be(data.sin_port); // This is transmuted from bytes in BigEndian order
        let path = format!(bind_or_connect!($mode "{}.{}.{}.{}:{}"), addr[0], addr[1], addr[2], addr[3], port);

        // Duplicate the socket, and then duplicate the copy back to the original fd
        let fd = e(syscall::dup($socket as usize, path.as_bytes()));
        if (fd as c_int) < 0 {
            return -1;
        }
        let result = syscall::dup2(fd, $socket as usize, &[]);
        let _ = syscall::close(fd);
        if (e(result) as c_int) < 0 {
            return -1;
        }
        0
    }}
}

pub unsafe fn accept(socket: c_int, address: *mut sockaddr, address_len: *mut socklen_t) -> c_int {
    let stream = e(syscall::dup(socket as usize, b"listen")) as c_int;
    if stream < 0 {
        return -1;
    }
    if address != ptr::null_mut()
        && address_len != ptr::null_mut()
        && getpeername(stream, address, address_len) < 0
    {
        return -1;
    }
    stream
}

pub fn access(path: *const c_char, mode: c_int) -> c_int {
    let fd = match RawFile::open(path, 0, 0) {
        Ok(fd) => fd,
        Err(_) => return -1
    };
    if mode == F_OK {
        return 0;
    }

    let mut stat = syscall::Stat::default();

    if e(syscall::fstat(*fd as usize, &mut stat)) == !0 {
        return -1;
    }

    let uid = e(syscall::getuid());
    if uid == !0 {
        return -1;
    }
    let gid = e(syscall::getgid());
    if gid == !0 {
        return -1;
    }

    let perms = if stat.st_uid as usize == uid {
        // octal has max 7 characters, binary has max two. And we're interested
        // in the 3rd digit
        stat.st_mode >> ((7 / 2) * 2 & 0o7)
    } else if stat.st_gid as usize == gid {
        stat.st_mode >> ((7 / 2) & 0o7)
    } else {
        stat.st_mode & 0o7
    };
    if (mode & R_OK == R_OK && perms & 0o4 != 0o4)
            || (mode & W_OK == W_OK && perms & 0o2 != 0o2)
            || (mode & X_OK == X_OK && perms & 0o1 != 0o1) {
        unsafe {
            errno = EINVAL;
        }
        return -1;
    }

    0
}

pub unsafe fn bind(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
    bind_or_connect!(bind socket, address, address_len)
}

pub fn brk(addr: *mut c_void) -> *mut c_void {
    unsafe { syscall::brk(addr as usize).unwrap_or(0) as *mut c_void }
}

pub fn chdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::chdir(path)) as c_int
}

pub fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_WRONLY) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = syscall::fchmod(fd as usize, mode as u16);
            let _ = syscall::close(fd);
            e(res) as c_int
        }
    }
}

pub fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_WRONLY) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = syscall::fchown(fd as usize, owner as u32, group as u32);
            let _ = syscall::close(fd);
            e(res) as c_int
        }
    }
}

pub fn close(fd: c_int) -> c_int {
    e(syscall::close(fd as usize)) as c_int
}

pub unsafe fn connect(socket: c_int, address: *const sockaddr, address_len: socklen_t) -> c_int {
    bind_or_connect!(connect socket, address, address_len)
}

pub fn dup(fd: c_int) -> c_int {
    e(syscall::dup(fd as usize, &[])) as c_int
}

pub fn dup2(fd1: c_int, fd2: c_int) -> c_int {
    e(syscall::dup2(fd1 as usize, fd2 as usize, &[])) as c_int
}

pub fn exit(status: c_int) -> ! {
    let _ = syscall::exit(status as usize);
    loop {}
}

pub unsafe extern "C" fn execve(
    path: *const c_char,
    mut argv: *const *mut c_char,
    mut envp: *const *mut c_char,
) -> c_int {
    use alloc::Vec;

    let fd = match RawFile::open(path, O_RDONLY as c_int, 0) {
        Ok(fd) => fd,
        Err(_) => return -1
    };

    let mut len = 0;
    while !(*argv.offset(len)).is_null() {
        len += 1;
    }

    let mut args: Vec<[usize; 2]> = Vec::with_capacity(len as usize);
    while !(*argv).is_null() {
        let arg = *argv;

        let mut len = 0;
        while *arg.offset(len) != 0 {
            len += 1;
        }
        args.push([*arg as usize, len as usize]);
        argv = argv.offset(1);
    }

    let mut len = 0;
    while !(*envp.offset(len)).is_null() {
        len += 1;
    }

    let mut envs: Vec<[usize; 2]> = Vec::with_capacity(len as usize);
    while !(*envp).is_null() {
        let env = *envp;

        let mut len = 0;
        while *env.offset(len) != 0 {
            len += 1;
        }
        envs.push([*env as usize, len as usize]);
        envp = envp.offset(1);
    }

    e(syscall::fexec(*fd as usize, &args, &envs)) as c_int
}

pub fn fchdir(fd: c_int) -> c_int {
    let path: &mut [u8] = &mut [0; 4096];
    if e(syscall::fpath(fd as usize, path)) == !0 {
        !0
    } else {
        e(syscall::chdir(path)) as c_int
    }
}

pub fn fchmod(fd: c_int, mode: mode_t) -> c_int {
    e(syscall::fchmod(fd as usize, mode as u16)) as c_int
}

pub fn fchown(fd: c_int, owner: uid_t, group: gid_t) -> c_int {
    e(syscall::fchown(fd as usize, owner as u32, group as u32)) as c_int
}

pub fn fcntl(fd: c_int, cmd: c_int, args: c_int) -> c_int {
    e(syscall::fcntl(fd as usize, cmd as usize, args as usize)) as c_int
}

pub fn flock(_fd: c_int, _operation: c_int) -> c_int {
    // TODO: Redox does not have file locking yet
    0
}

pub fn fork() -> pid_t {
    e(unsafe { syscall::clone(0) }) as pid_t
}

pub fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    let mut redox_buf: redox_stat = redox_stat::default();
    match e(syscall::fstat(fildes as usize, &mut redox_buf)) {
        0 => {
            unsafe {
                if !buf.is_null() {
                    (*buf).st_dev = redox_buf.st_dev as dev_t;
                    (*buf).st_ino = redox_buf.st_ino as ino_t;
                    (*buf).st_nlink = redox_buf.st_nlink as nlink_t;
                    (*buf).st_mode = redox_buf.st_mode as mode_t;
                    (*buf).st_uid = redox_buf.st_uid as uid_t;
                    (*buf).st_gid = redox_buf.st_gid as gid_t;
                    // TODO st_rdev
                    (*buf).st_rdev = 0;
                    (*buf).st_size = redox_buf.st_size as off_t;
                    (*buf).st_blksize = redox_buf.st_blksize as blksize_t;
                    (*buf).st_atim = timespec {
                        tv_sec: redox_buf.st_atime as time_t,
                        tv_nsec: redox_buf.st_atime_nsec as c_long,
                    };
                    (*buf).st_mtim = timespec {
                        tv_sec: redox_buf.st_mtime as time_t,
                        tv_nsec: redox_buf.st_mtime_nsec as c_long,
                    };
                    (*buf).st_ctim = timespec {
                        tv_sec: redox_buf.st_ctime as time_t,
                        tv_nsec: redox_buf.st_ctime_nsec as c_long,
                    };
                }
            }
            0
        }
        _ => -1,
    }
}

pub fn fsync(fd: c_int) -> c_int {
    e(syscall::fsync(fd as usize)) as c_int
}

pub fn ftruncate(fd: c_int, len: off_t) -> c_int {
    e(syscall::ftruncate(fd as usize, len as usize)) as c_int
}

pub fn futimens(fd: c_int, times: *const timespec) -> c_int {
    let times = [unsafe { redox_timespec::from(&*times) }, unsafe {
        redox_timespec::from(&*times.offset(1))
    }];
    e(syscall::futimens(fd as usize, &times)) as c_int
}

pub fn utimens(path: *const c_char, times: *const timespec) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_STAT) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = futimens(fd as c_int, times);
            let _ = syscall::close(fd);
            res
        }
    }
}

pub fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    let buf_slice = unsafe { slice::from_raw_parts_mut(buf as *mut u8, size as usize - 1) };
    let read = e(syscall::getcwd(buf_slice));
    if read == !0 {
        ptr::null_mut()
    } else {
        unsafe {
            *buf.offset(read as isize + 1) = 0;
        }
        buf
    }
}

pub fn getdents(fd: c_int, mut dirents: *mut dirent, mut bytes: usize) -> c_int {
    let mut amount = 0;

    let mut buf = [0; 1024];
    let mut bindex = 0;
    let mut blen = 0;

    let mut name = [0; 256];
    let mut nindex = 0;

    loop {
        if bindex >= blen {
            bindex = 0;
            blen = match syscall::read(fd as usize, &mut buf) {
                Ok(0) => return amount,
                Ok(n) => n,
                Err(err) => return -err.errno,
            };
        }

        if buf[bindex] == b'\n' {
            // Put a NUL byte either at the end, or if it's too big, at where it's truncated.
            name[nindex.min(name.len() - 1)] = 0;
            unsafe {
                *dirents = dirent {
                    d_ino: 0,
                    d_off: 0,
                    d_reclen: mem::size_of::<dirent>() as c_ushort,
                    d_type: 0,
                    d_name: name,
                };
                dirents = dirents.offset(1);
            }
            amount += 1;
            if bytes <= mem::size_of::<dirent>() {
                return amount;
            }
            bytes -= mem::size_of::<dirent>();
        } else {
            if nindex < name.len() {
                name[nindex] = buf[bindex] as c_char;
            }
            nindex += 1;
            bindex += 1;
        }
    }
}

pub fn getegid() -> gid_t {
    e(syscall::getegid()) as gid_t
}

pub fn geteuid() -> uid_t {
    e(syscall::geteuid()) as uid_t
}

pub fn getgid() -> gid_t {
    e(syscall::getgid()) as gid_t
}

pub fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: getrusage({}, {:p})",
        who,
        r_usage
    );
    -1
}

pub unsafe fn gethostname(mut name: *mut c_char, len: size_t) -> c_int {
    let fd = e(syscall::open("/etc/hostname", O_RDONLY)) as i32;
    if fd < 0 {
        return fd;
    }
    let mut reader = FileReader(fd);
    for _ in 0..len {
        match reader.read_u8() {
            Ok(Some(b)) => {
                *name = b as c_char;
                name = name.offset(1);
            },
            Ok(None) => {
                *name = 0;
                break;
            },
            Err(()) => return -1
        }
    }
    0
}

unsafe fn inner_get_name(
    local: bool,
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> Result<usize> {
    // 32 should probably be large enough.
    // Format: tcp:remote/local
    // and since we only yet support IPv4 (I think)...
    let mut buf = [0; 32];
    let len = syscall::fpath(socket as usize, &mut buf)?;
    let buf = &buf[..len];
    assert!(&buf[..4] == b"tcp:" || &buf[..4] == b"udp:");
    let buf = &buf[4..];

    let mut parts = buf.split(|c| *c == b'/');
    if local {
        // Skip the remote part
        parts.next();
    }
    let part = parts.next().expect("Invalid reply from netstack");

    let data = slice::from_raw_parts_mut(
        &mut (*address).data as *mut _ as *mut u8,
        (*address).data.len(),
    );

    let len = data.len().min(part.len());
    data[..len].copy_from_slice(&part[..len]);

    *address_len = len as socklen_t;
    Ok(0)
}

pub fn getitimer(which: c_int, out: *mut itimerval) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: getitimer({}, {:p})",
        which,
        out
    );

    unsafe {
        *out = itimerval::default();
    }
    0
}

pub unsafe fn getpeername(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    e(inner_get_name(false, socket, address, address_len)) as c_int
}

pub fn getpgid(pid: pid_t) -> pid_t {
    e(syscall::getpgid(pid as usize)) as pid_t
}

pub fn getpid() -> pid_t {
    e(syscall::getpid()) as pid_t
}

pub fn getppid() -> pid_t {
    e(syscall::getppid()) as pid_t
}

pub unsafe fn getsockname(
    socket: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> c_int {
    e(inner_get_name(true, socket, address, address_len)) as c_int
}

pub fn getsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *mut c_void,
    option_len: *mut socklen_t,
) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: getsockopt({}, {}, {}, {:p}, {:p})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    );
    -1
}

pub fn gettimeofday(tp: *mut timeval, tzp: *mut timezone) -> c_int {
    let mut redox_tp = redox_timespec::default();
    let err = e(syscall::clock_gettime(
        syscall::CLOCK_REALTIME,
        &mut redox_tp,
    )) as c_int;
    if err < 0 {
        return err;
    }
    unsafe {
        (*tp).tv_sec = redox_tp.tv_sec as time_t;
        (*tp).tv_usec = (redox_tp.tv_nsec / 1000) as suseconds_t;

        if !tzp.is_null() {
            (*tzp).tz_minuteswest = 0;
            (*tzp).tz_dsttime = 0;
        }
    }
    0
}

pub fn getuid() -> uid_t {
    e(syscall::getuid()) as pid_t
}

pub fn isatty(fd: c_int) -> c_int {
    syscall::dup(fd as usize, b"termios")
        .map(|fd| {
            let _ = syscall::close(fd);
            1
        })
        .unwrap_or(0)
}

pub fn kill(pid: pid_t, sig: c_int) -> c_int {
    e(syscall::kill(pid as usize, sig as usize)) as c_int
}

pub fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    e(syscall::kill(-(pgrp as isize) as usize, sig as usize)) as c_int
}

pub fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    let path1 = unsafe { c_str(path1) };
    let path2 = unsafe { c_str(path2) };
    e(unsafe { syscall::link(path1.as_ptr(), path2.as_ptr()) }) as c_int
}

pub fn listen(_socket: c_int, _backlog: c_int) -> c_int {
    // TODO
    0
}

pub fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    e(syscall::lseek(
        fd as usize,
        offset as isize,
        whence as usize,
    )) as off_t
}

pub fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_STAT | O_NOFOLLOW) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = fstat(fd as i32, buf);
            let _ = syscall::close(fd);
            res
        }
    }
}

pub fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let flags = O_CREAT | O_EXCL | O_CLOEXEC | O_DIRECTORY | mode as usize & 0o777;
    let path = unsafe { c_str(path) };
    match syscall::open(path, flags) {
        Ok(fd) => {
            let _ = syscall::close(fd);
            0
        }
        Err(err) => e(Err(err)) as c_int,
    }
}

pub fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
    let flags = O_CREAT | MODE_FIFO as usize | mode as usize & 0o777;
    let path = unsafe { c_str(path) };
    match syscall::open(path, flags) {
        Ok(fd) => {
            let _ = syscall::close(fd);
            0
        }
        Err(err) => e(Err(err)) as c_int,
    }
}

pub unsafe fn mmap(
    _addr: *mut c_void,
    len: usize,
    _prot: c_int,
    flags: c_int,
    fildes: c_int,
    off: off_t,
) -> *mut c_void {
    if flags & MAP_ANON == MAP_ANON {
        let fd = e(syscall::open("memory:", 0)); // flags don't matter currently
        if fd == !0 {
            return !0 as *mut c_void;
        }

        let addr = e(syscall::fmap(fd, off as usize, len as usize));
        if addr == !0 {
            let _ = syscall::close(fd);
            return !0 as *mut c_void;
        }

        anonymous_maps().insert(addr as usize, fd);
        addr as *mut c_void
    } else {
        e(syscall::fmap(fildes as usize, off as usize, len as usize)) as *mut c_void
    }
}

pub unsafe fn munmap(addr: *mut c_void, _len: usize) -> c_int {
    if e(syscall::funmap(addr as usize)) == !0 {
        return !0;
    }
    if let Some(fd) = anonymous_maps().remove(&(addr as usize)) {
        let _ = syscall::close(fd);
    }
    0
}

pub fn nanosleep(rqtp: *const timespec, rmtp: *mut timespec) -> c_int {
    let redox_rqtp = unsafe { redox_timespec::from(&*rqtp) };
    let mut redox_rmtp: redox_timespec;
    if rmtp.is_null() {
        redox_rmtp = redox_timespec::default();
    } else {
        redox_rmtp = unsafe { redox_timespec::from(&*rmtp) };
    }
    match e(syscall::nanosleep(&redox_rqtp, &mut redox_rmtp)) as c_int {
        -1 => -1,
        _ => {
            unsafe {
                if !rmtp.is_null() {
                    (*rmtp).tv_sec = redox_rmtp.tv_sec;
                    (*rmtp).tv_nsec = redox_rmtp.tv_nsec as i64;
                }
            }
            0
        }
    }
}

pub fn open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::open(path, (oflag as usize) | (mode as usize))) as c_int
}

pub fn pipe(fds: &mut [c_int]) -> c_int {
    let mut usize_fds: [usize; 2] = [0; 2];
    let res = e(syscall::pipe2(&mut usize_fds, 0));
    fds[0] = usize_fds[0] as c_int;
    fds[1] = usize_fds[1] as c_int;
    res as c_int
}

pub fn raise(sig: c_int) -> c_int {
    kill(getpid(), sig)
}

pub fn read(fd: c_int, buf: &mut [u8]) -> ssize_t {
    e(syscall::read(fd as usize, buf)) as ssize_t
}

pub unsafe fn recvfrom(
    socket: c_int,
    buf: *mut c_void,
    len: size_t,
    flags: c_int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    if flags != 0 {
        errno = syscall::EOPNOTSUPP;
        return -1;
    }
    if address != ptr::null_mut()
        && address_len != ptr::null_mut()
        && getpeername(socket, address, address_len) < 0
    {
        return -1;
    }
    read(socket, slice::from_raw_parts_mut(buf as *mut u8, len))
}

pub fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    let (oldpath, newpath) = unsafe { (c_str(oldpath), c_str(newpath)) };
    match syscall::open(oldpath, O_WRONLY) {
        Ok(fd) => {
            let retval = syscall::frename(fd, newpath);
            let _ = syscall::close(fd);
            e(retval) as c_int
        }
        err => e(err) as c_int,
    }
}

pub fn rmdir(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::rmdir(path)) as c_int
}

pub fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    fn isset(set: *mut fd_set, fd: usize) -> bool {
        if set.is_null() {
            return false;
        }

        let mask = 1 << (fd & (8 * mem::size_of::<c_ulong>() - 1));
        unsafe { (*set).fds_bits[fd / (8 * mem::size_of::<c_ulong>())] & mask == mask }
    }

    let event_file = match RawFile::open("event:\0".as_ptr() as *const c_char, 0, 0) {
        Ok(file) => file,
        Err(_) => return -1,
    };

    let mut total = 0;

    for fd in 0..nfds as usize {
        macro_rules! register {
            ($fd:expr, $flags:expr) => {
                if write(
                    *event_file,
                    &syscall::Event {
                        id: $fd,
                        flags: $flags,
                        data: 0,
                    },
                ) < 0
                {
                    return -1;
                }
            };
        }
        if isset(readfds, fd) {
            register!(fd, syscall::EVENT_READ);
            total += 1;
        }
        if isset(writefds, fd) {
            register!(fd, syscall::EVENT_WRITE);
            total += 1;
        }
        if isset(exceptfds, fd) {
            total += 1;
        }
    }

    const TIMEOUT_TOKEN: usize = 1;

    let timeout_file = if timeout.is_null() {
        None
    } else {
        let timeout_file = match RawFile::open(
            format!("time:{}\0", syscall::CLOCK_MONOTONIC).as_ptr() as *const c_char,
            0,
            0,
        ) {
            Ok(file) => file,
            Err(_) => return -1,
        };
        let timeout = unsafe { &*timeout };
        if write(
            *timeout_file,
            &syscall::TimeSpec {
                tv_sec: timeout.tv_sec,
                tv_nsec: timeout.tv_usec * 1000,
            },
        ) < 0
        {
            return -1;
        }
        if write(
            *event_file,
            &syscall::Event {
                id: *timeout_file as usize,
                flags: syscall::EVENT_READ,
                data: TIMEOUT_TOKEN,
            },
        ) < 0
        {
            return -1;
        }

        Some(timeout_file)
    };

    let mut event = syscall::Event::default();
    if read(*event_file, &mut event) < 0 {
        return -1;
    }

    if timeout_file.is_some() && event.data == TIMEOUT_TOKEN {
        return 0;
    }

    // I really don't get why, but select wants me to return the total number
    // of file descriptors that was inputted. I'm confused.
    total
}

pub unsafe fn sendto(
    socket: c_int,
    buf: *const c_void,
    len: size_t,
    flags: c_int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    if dest_addr != ptr::null() || dest_len != 0 {
        errno = syscall::EISCONN;
        return -1;
    }
    if flags != 0 {
        errno = syscall::EOPNOTSUPP;
        return -1;
    }
    write(socket, slice::from_raw_parts(buf as *const u8, len))
}

pub fn setitimer(which: c_int, new: *const itimerval, old: *mut itimerval) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: setitimer({}, {:p}, {:p})",
        which,
        new,
        old
    );

    unsafe {
        *old = itimerval::default();
    }
    0
}

pub fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    e(syscall::setpgid(pid as usize, pgid as usize)) as c_int
}

pub fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    e(syscall::setregid(rgid as usize, egid as usize)) as c_int
}

pub fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    e(syscall::setreuid(ruid as usize, euid as usize)) as c_int
}

pub fn setsockopt(
    socket: c_int,
    level: c_int,
    option_name: c_int,
    option_value: *const c_void,
    option_len: socklen_t,
) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: setsockopt({}, {}, {}, {:p}, {})",
        socket,
        level,
        option_name,
        option_value,
        option_len
    );
    -1
}

pub fn shutdown(socket: c_int, how: c_int) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: shutdown({}, {})",
        socket,
        how
    );
    -1
}

pub unsafe fn sigaction(sig: c_int, act: *const sigaction, oact: *mut sigaction) -> c_int {
    if !oact.is_null() {
        // Assumes the last sigaction() call was made by relibc and not a different one
        if SIG_HANDLER.is_some() {
            (*oact).sa_handler = SIG_HANDLER;
        }
    }
    let act = if act.is_null() {
        None
    } else {
        SIG_HANDLER = (*act).sa_handler;
        let m = (*act).sa_mask;
        Some(syscall::SigAction {
            sa_handler: sig_handler,
            sa_mask: [0, m as u64],
            sa_flags: (*act).sa_flags as usize,
        })
    };
    let mut old = syscall::SigAction::default();
    let ret = e(syscall::sigaction(
        sig as usize,
        act.as_ref(),
        if oact.is_null() { None } else { Some(&mut old) },
    )) as c_int;
    if !oact.is_null() {
        let m = old.sa_mask;
        (*oact).sa_mask = m[1] as c_ulong;
        (*oact).sa_flags = old.sa_flags as c_ulong;
    }
    ret
}

pub fn sigprocmask(how: c_int, set: *const sigset_t, oset: *mut sigset_t) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: sigprocmask({}, {:p}, {:p})",
        how,
        set,
        oset
    );
    -1
}

pub fn stat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { c_str(path) };
    match syscall::open(path, O_STAT) {
        Err(err) => e(Err(err)) as c_int,
        Ok(fd) => {
            let res = fstat(fd as i32, buf);
            let _ = syscall::close(fd);
            res
        }
    }
}

pub unsafe fn socket(domain: c_int, mut kind: c_int, protocol: c_int) -> c_int {
    if domain != AF_INET {
        errno = syscall::EAFNOSUPPORT;
        return -1;
    }
    if protocol != 0 {
        errno = syscall::EPROTONOSUPPORT;
        return -1;
    }

    let mut flags = O_RDWR;
    if kind & SOCK_NONBLOCK == SOCK_NONBLOCK {
        kind &= !SOCK_NONBLOCK;
        flags |= O_NONBLOCK;
    }
    if kind & SOCK_CLOEXEC == SOCK_CLOEXEC {
        kind &= !SOCK_CLOEXEC;
        flags |= O_CLOEXEC;
    }

    // The tcp: and udp: schemes allow using no path,
    // and later specifying one using `dup`.
    match kind {
        SOCK_STREAM => e(syscall::open("tcp:", flags)) as c_int,
        SOCK_DGRAM => e(syscall::open("udp:", flags)) as c_int,
        _ => {
            errno = syscall::EPROTOTYPE;
            -1
        }
    }
}

pub fn socketpair(domain: c_int, kind: c_int, protocol: c_int, socket_vector: *mut c_int) -> c_int {
    let _ = writeln!(
        FileWriter(2),
        "unimplemented: socketpair({}, {}, {}, {:p})",
        domain,
        kind,
        protocol,
        socket_vector
    );
    -1
}

pub fn tcgetattr(fd: c_int, out: *mut termios) -> c_int {
    let dup = e(syscall::dup(fd as usize, b"termios"));
    if dup == !0 {
        return -1;
    }

    let read = e(syscall::read(dup, unsafe { slice::from_raw_parts_mut(
        out as *mut u8,
        mem::size_of::<termios>()
    ) }));
    let _ = syscall::close(dup);

    if read == !0 {
        return -1;
    }
    0
}

pub fn tcsetattr(fd: c_int, _act: c_int, value: *const termios) -> c_int {
    let dup = e(syscall::dup(fd as usize, b"termios"));
    if dup == !0 {
        return -1;
    }

    let write = e(syscall::write(dup, unsafe { slice::from_raw_parts(
        value as *const u8,
        mem::size_of::<termios>()
    ) }));
    let _ = syscall::close(dup);

    if write == !0 {
        return -1;
    }
    0
}

pub fn times(out: *mut tms) -> clock_t {
    let _ = writeln!(FileWriter(2), "unimplemented: times({:p})", out);
    !0
}

pub fn umask(mask: mode_t) -> mode_t {
    let _ = writeln!(FileWriter(2), "unimplemented: umask({})", mask);
    0
}

pub fn unlink(path: *const c_char) -> c_int {
    let path = unsafe { c_str(path) };
    e(syscall::unlink(path)) as c_int
}

pub fn waitpid(mut pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
    if pid == !0 {
        pid = 0;
    }
    unsafe {
        let mut temp: usize = 0;
        let res = e(syscall::waitpid(pid as usize, &mut temp, options as usize));
        if !stat_loc.is_null() {
            *stat_loc = temp as c_int;
        }
        res as pid_t
    }
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    e(syscall::write(fd as usize, buf)) as ssize_t
}

pub fn clock_gettime(clk_id: clockid_t, tp: *mut timespec) -> c_int {
    let mut redox_tp = unsafe { redox_timespec::from(&*tp) };
    match e(syscall::clock_gettime(clk_id as usize, &mut redox_tp)) as c_int {
        -1 => -1,
        _ => {
            unsafe {
                (*tp).tv_sec = redox_tp.tv_sec;
                (*tp).tv_nsec = redox_tp.tv_nsec as i64;
            };
            0
        }
    }
}
