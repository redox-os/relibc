//! unistd implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

#![no_std]

extern crate libc;

use libc::*;

pub const NULL: c_int = 0;

pub const R_OK: c_int = 1;
pub const W_OK: c_int = 2;
pub const X_OK: c_int = 4;
pub const F_OK: c_int = 8;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

pub const F_ULOCK: c_int = 0;
pub const F_LOCK: c_int = 1;
pub const F_TLOCK: c_int = 2;
pub const F_TEST: c_int = 3;

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

#[no_mangle]
pub extern "C" fn _exit(status: c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn access(path: *const c_char, amode: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn alarm(seconds: c_uint) -> c_uint {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn brk(addr: *mut c_void) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chdir(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chroot(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn close(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn confstr(name: c_int, buf: *mut c_char, len: size_t) -> size_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn crypt(key: *const c_char, salt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctermid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn dup2(fildes: c_int, fildes2: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn encrypt(block: [c_char; 64], edflag: c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execl(path: *const c_char, arg0: *const c_char /* TODO: , mut args: ... */) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execle(path: *const c_char, arg0: *const c_char /* TODO: , mut args: ... */) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execlp(file: *const c_char, arg0: *const c_char /* TODO: , mut args: ... */) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execv(path: *const c_char, argv: *const *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execve(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn execvp(file: *const c_char, argv: *const *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fchown(fildes: c_int, owner: uid_t, group: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fchdir(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fdatasync(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fork() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fpathconf(fildes: c_int, name: c_int) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fsync(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftruncate(fildes: c_int, length: off_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getdtablesize() -> c_int {
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
pub extern "C" fn getgroups(gidsetsize: c_int, grouplist: *mut gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gethostid() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getlogin() -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getlogin_r(name: *mut c_char, namesize: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getopt(argc: c_int, argv: *const *mut c_char, opstring: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpass(prompt: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getpgid(pid: pid_t) -> pid_t {
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
pub extern "C" fn getsid(pid: pid_t) -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getuid() -> uid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getwd(path_name: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn isatty(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lchown(path: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn link(path1: *const c_char, path2: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lockf(fildes: c_int, function: c_int, size: off_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn lseek(fildes: c_int, offset: off_t, whence: c_int) -> off_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn nice(incr: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pathconf(path: *const c_char, name: c_int) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pause() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pipe(fildes: [c_int; 2]) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pread(fildes: c_int, buf: *mut c_void, nbyte: size_t, offset: off_t) -> ssize_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_atfork(prepare: extern "C" fn(), parent: extern "C" fn(), child: extern "C" fn()) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pwrite(fildes: c_int, buf: *const c_void, nbyte: size_t, offset: off_t) -> ssize_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn read(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsize: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rmdir(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sbrk(incr: intptr_t) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setgid(gid: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setpgrp() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setreuid(ruid: uid_t, euid: uid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setsid() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setuid(uid: uid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sleep(seconds: c_uint) -> c_uint {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn swab(src: *const c_void, dest: *mut c_void, nbytes: ssize_t) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn symlink(path1: *const c_char, path2: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sync() {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sysconf(name: c_int) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tcgetpgrp() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tcsetpgrp(fildes: c_int, pgid_id: pid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ttyname(fildes: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ttyname_r(fildes: c_int, name: *mut c_char, namesize: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ualarm(useconds: useconds_t, interval: useconds_t) -> useconds_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn unlink(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn usleep(useconds: useconds_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vfork() -> pid_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn write(fildes: c_int, buf: *const c_void, nbyte: size_t) -> ssize_t {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
