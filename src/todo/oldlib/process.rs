use libc::{c_char, c_int, c_void, size_t, gid_t, uid_t, ptrdiff_t};
use ::types::pid_t;
use core::slice;
use alloc::Vec;
use syscall::error::{Error, EINVAL};
use syscall;

const MAXPATHLEN: usize = 1024;
static mut CURR_BRK: usize = 0;

libc_fn!(unsafe chdir(path: *const c_char) -> Result<c_int> {
    Ok(syscall::chdir(::cstr_to_slice(path))? as c_int)
});

libc_fn!(unsafe fchdir(fd: c_int) -> Result<c_int> {
    let mut buf = [0; MAXPATHLEN];
    let length = syscall::fpath(fd as usize, &mut buf)?;
    Ok(syscall::chdir(&buf[0..length])? as c_int)
});

libc_fn!(unsafe _exit(code: c_int) {
    ::__libc_fini_array();
    syscall::exit(code as usize).unwrap();
});

libc_fn!(unsafe _execve(name: *const c_char, argv: *const *const c_char, env: *const *const c_char) -> Result<c_int> {
    let mut env = env;
    while !(*env).is_null() {
        let slice = ::cstr_to_slice(*env);
        // Should always contain a =, but worth checking
        if let Some(sep) = slice.iter().position(|&c| c == b'=') {
            let mut path = b"env:".to_vec();
            path.extend_from_slice(&slice[..sep]);
            if let Ok(fd) = ::RawFile::open(&path, syscall::O_WRONLY | syscall::O_CREAT) {
                let _ = syscall::write(*fd, &slice[sep+1..]);
            }
        }
        env = env.offset(1);
    }

    let mut args: Vec<[usize; 2]> = Vec::new();
    let mut arg = argv;
    while !(*arg).is_null() {
        args.push([*arg as usize, ::strlen(*arg)]);
        arg = arg.offset(1);
    }

    Ok(syscall::execve(::cstr_to_slice(name), &args)? as c_int)
});

libc_fn!(unsafe _fork() -> Result<c_int> {
    Ok(syscall::clone(0)? as c_int)
});

libc_fn!(unsafe vfork() -> Result<c_int> {
    Ok(syscall::clone(syscall::CLONE_VFORK)? as c_int)
});

libc_fn!(unsafe getcwd(buf: *mut c_char, size: size_t) -> Result<*const c_char> {
    let mut size = size;
    if size == 0 {
        size = ::file::PATH_MAX;
    }
    let buf = ::MallocNull::new(buf, size);
    let slice = slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, size);
    let count = syscall::getcwd(&mut slice[..size-1])?;
    slice[count] = b'\0';
    Ok(buf.into_raw())
    // FIXME: buffer too small
});

libc_fn!(unsafe getwd(buf: *mut c_char) -> Result<*const c_char> {

    if buf.is_null() {
        Err(Error::new(EINVAL))
    } else {
        let mut tmp: [u8; MAXPATHLEN] = [0; MAXPATHLEN];
        syscall::getcwd(&mut tmp)?;
        slice::from_raw_parts_mut(buf as *mut u8, MAXPATHLEN)
            .copy_from_slice(&mut tmp);
        Ok(buf)
    }
});

libc_fn!(unsafe _getpid() -> pid_t {
    syscall::getpid().unwrap() as pid_t
});

libc_fn!(unsafe getppid() -> pid_t {
    syscall::getppid().unwrap() as pid_t
});

libc_fn!(unsafe getegid() -> gid_t {
    syscall::getegid().unwrap() as gid_t
});

libc_fn!(unsafe geteuid() -> uid_t {
    syscall::geteuid().unwrap() as uid_t
});

libc_fn!(unsafe getgid() -> gid_t {
    syscall::getgid().unwrap() as gid_t
});

libc_fn!(unsafe getuid() -> uid_t {
    syscall::getuid().unwrap() as uid_t
});

libc_fn!(unsafe _kill(pid: c_int, sig: c_int) -> Result<c_int> {
    Ok(syscall::kill(pid as usize, sig as usize)? as c_int)
});

libc_fn!(unsafe _brk(end_data_segment: *mut c_void) -> Result<c_int> {
    CURR_BRK = syscall::brk(end_data_segment as usize)?;
    Ok(0)
});

libc_fn!(unsafe _sbrk(increment: ptrdiff_t) -> *mut c_void {
    if CURR_BRK == 0 {
        CURR_BRK = syscall::brk(0).unwrap();
    }
    let old_brk = CURR_BRK;
    if increment != 0 {
        let addr = if increment >= 0 {
            old_brk + increment as usize
        } else {
            old_brk - (-increment) as usize
        };
        CURR_BRK = match syscall::brk(addr) {
            Ok(x) => x,
            Err(err) => {
                *::__errno() = err.errno;
                return -1 as isize as *mut c_void;
            }
        }
    }
    old_brk as *mut c_void
});

libc_fn!(unsafe _sched_yield() -> Result<c_int> {
    Ok(syscall::sched_yield()? as c_int)
});

libc_fn!(unsafe _system(s: *const c_char) -> Result<c_int> {
    match syscall::clone(0)? {
        0 => {
            let shell = "/bin/sh";
            let arg1 = "-c";
            let args = [
                [shell.as_ptr() as usize, shell.len()],
                [arg1.as_ptr() as usize, arg1.len()],
                [s as usize, ::strlen(s)]
            ];
            syscall::execve(shell, &args)?;
            syscall::exit(100)?;
            unreachable!()
        }
        pid => {
            let mut status = 0;
            syscall::waitpid(pid, &mut status, 0)?;
            Ok(status as c_int)
        }
    }
});

libc_fn!(unsafe setregid(rgid: gid_t, egid: gid_t) -> Result<c_int> {
    Ok(syscall::setregid(rgid as usize, egid as usize)? as c_int)
});

libc_fn!(unsafe setegid(egid: gid_t) -> Result<c_int> {
    Ok(syscall::setregid(-1isize as usize, egid as usize)? as c_int)
});

libc_fn!(unsafe setgid(gid: gid_t) -> Result<c_int> {
    Ok(syscall::setregid(gid as usize, gid as usize)? as c_int)
});

libc_fn!(unsafe setreuid(ruid: uid_t, euid: uid_t) -> Result<c_int> {
    Ok(syscall::setreuid(ruid as usize, euid as usize)? as c_int)
});

libc_fn!(unsafe seteuid(euid: uid_t) -> Result<c_int> {
    Ok(syscall::setreuid(-1isize as usize, euid as usize)? as c_int)
});

libc_fn!(unsafe setuid(uid: uid_t) -> Result<c_int> {
    Ok(syscall::setreuid(uid as usize, uid as usize)? as c_int)
});

libc_fn!(unsafe _wait(status: *mut c_int) -> Result<c_int> {
    let mut buf = 0;
    let res = syscall::waitpid(0, &mut buf, 0)?;
    *status = buf as c_int;
    Ok(res as c_int)
});

libc_fn!(unsafe waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> Result<c_int> {
    let mut buf = 0;
    let pid = if pid == -1 {
        0
    } else {
        pid as usize
    };
    let res = syscall::waitpid(pid as usize, &mut buf, options as usize)?;
    *status = buf as c_int;
    Ok(res as c_int)
});
