#[no_mangle]
pub extern "C" fn alarm(arg1: libc::c_uint) -> libc::c_uint {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn brk(arg1: *mut libc::c_void) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chdir(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chroot(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chown(arg1: *const libc::c_char, arg2: uid_t,
                 arg3: gid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn close(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn confstr(arg1: libc::c_int,
                   arg2: *mut libc::c_char, arg3: usize) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn crypt(arg1: *const libc::c_char,
                 arg2: *const libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctermid(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(s: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup2(arg1: libc::c_int, arg2: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn encrypt(arg1: *mut libc::c_char,
                   arg2: libc::c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execl(arg1: *const libc::c_char,
                 arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execle(arg1: *const libc::c_char,
                  arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execlp(arg1: *const libc::c_char,
                  arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execv(arg1: *const libc::c_char,
                 arg2: *const *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execve(arg1: *const libc::c_char,
                  arg2: *const *const libc::c_char,
                  arg3: *const *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execvp(arg1: *const libc::c_char,
                  arg2: *const *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn _exit(arg1: libc::c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fchown(arg1: libc::c_int, arg2: uid_t, arg3: gid_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fchdir(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fdatasync(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fork() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fpathconf(arg1: libc::c_int, arg2: libc::c_int)
     -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fsync(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftruncate(arg1: libc::c_int, arg2: off_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getcwd(arg1: *mut libc::c_char, arg2: usize)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getdtablesize() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getegid() -> gid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn geteuid() -> uid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getgid() -> gid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getgroups(arg1: libc::c_int, arg2: *mut gid_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gethostid() -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getlogin() -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getlogin_r(arg1: *mut libc::c_char, arg2: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getopt(arg1: libc::c_int,
                  arg2: *const *const libc::c_char,
                  arg3: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpagesize() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpass(arg1: *const libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpgid(arg1: pid_t) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpgrp() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getppid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getsid(arg1: pid_t) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getuid() -> uid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getwd(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isatty(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lchown(arg1: *const libc::c_char, arg2: uid_t,
                  arg3: gid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn link(arg1: *const libc::c_char,
                arg2: *const libc::c_char) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lockf(arg1: libc::c_int, arg2: libc::c_int,
                 arg3: off_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lseek(arg1: libc::c_int, arg2: off_t,
                 arg3: libc::c_int) -> off_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn nice(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pathconf(arg1: *const libc::c_char,
                    arg2: libc::c_int) -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pause() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pipe(arg1: *mut libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pread(arg1: libc::c_int,
                 arg2: *mut libc::c_void, arg3: usize, arg4: off_t)
     -> isize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_atfork(arg1: ::std::option::Option<unsafe extern "C" fn()>,
                          arg2: ::std::option::Option<unsafe extern "C" fn()>,
                          arg3: ::std::option::Option<unsafe extern "C" fn()>)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pwrite(arg1: libc::c_int,
                  arg2: *const libc::c_void, arg3: usize,
                  arg4: off_t) -> isize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn read(arg1: libc::c_int,
                arg2: *mut libc::c_void, arg3: usize) -> isize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn readlink(arg1: *const libc::c_char,
                    arg2: *mut libc::c_char, arg3: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rmdir(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sbrk(arg1: isize) -> *mut libc::c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setgid(arg1: gid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setpgid(arg1: pid_t, arg2: pid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setpgrp() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setregid(arg1: gid_t, arg2: gid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setreuid(arg1: uid_t, arg2: uid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setsid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setuid(arg1: uid_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sleep(arg1: libc::c_uint) -> libc::c_uint {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn swab(arg1: *const libc::c_void,
                arg2: *mut libc::c_void, arg3: isize) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn symlink(arg1: *const libc::c_char,
                   arg2: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sync() {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sysconf(arg1: libc::c_int) -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tcgetpgrp(arg1: libc::c_int) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tcsetpgrp(arg1: libc::c_int, arg2: pid_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn truncate(arg1: *const libc::c_char, arg2: off_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ttyname(arg1: libc::c_int)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ttyname_r(arg1: libc::c_int,
                     arg2: *mut libc::c_char, arg3: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ualarm(arg1: useconds_t, arg2: useconds_t) -> useconds_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn unlink(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn usleep(arg1: useconds_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vfork() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn write(arg1: libc::c_int,
                 arg2: *const libc::c_void, arg3: usize) -> isize {
    unimplemented!();
}

