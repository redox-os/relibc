/// unistd implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

extern crate libc;

use libc::*;

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
pub extern "C" fn _exit(status: c_int) {
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

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
