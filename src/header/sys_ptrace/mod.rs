//! ptrace compatibility layer for Redox OS

use crate::platform::{types::*, PalPtrace, Sys};
use core::ffi::VaList;

pub const PTRACE_TRACEME: c_int = 0;
pub const PTRACE_PEEKTEXT: c_int = 1;
pub const PTRACE_PEEKDATA: c_int = 2;
pub const PTRACE_POKETEXT: c_int = 4;
pub const PTRACE_POKEDATA: c_int = 5;
pub const PTRACE_CONT: c_int = 7;
pub const PTRACE_KILL: c_int = 8;
pub const PTRACE_SINGLESTEP: c_int = 9;
pub const PTRACE_GETREGS: c_int = 12;
pub const PTRACE_SETREGS: c_int = 13;
pub const PTRACE_GETFPREGS: c_int = 14;
pub const PTRACE_SETFPREGS: c_int = 15;
pub const PTRACE_ATTACH: c_int = 16;
pub const PTRACE_DETACH: c_int = 17;
pub const PTRACE_SYSCALL: c_int = 24;
pub const PTRACE_SYSEMU: c_int = 31;
pub const PTRACE_SYSEMU_SINGLESTEP: c_int = 32;

// Can't use "params: ..." syntax, because... guess what? Cbingen again :(
#[no_mangle]
pub unsafe extern "C" fn sys_ptrace(request: c_int, mut params: VaList) -> c_int {
    // Musl also just grabs the arguments from the varargs...
    Sys::ptrace(request, params.arg(), params.arg(), params.arg())
}
