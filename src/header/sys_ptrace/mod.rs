//! `sys/ptrace.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/ptrace.2.html>.

use crate::{
    error::ResultExt,
    platform::{PalPtrace, Sys, types::c_int},
};

/// Indicate that this process is to be traced by its parent.
/// Only used by the tracee.
pub const PTRACE_TRACEME: c_int = 0;
/// Equivalent to `PTRACE_PEEKDATA`.
pub const PTRACE_PEEKTEXT: c_int = 1;
/// Read a word at the address `addr` in the tracee's memory, returning the
/// word as a result of the `ptrace()` call.
pub const PTRACE_PEEKDATA: c_int = 2;
/// Equivalent to `PTRACE_POKEDATA`.
pub const PTRACE_POKETEXT: c_int = 4;
/// Copy the word `data` to the address `addr` in the tracee's memory.
pub const PTRACE_POKEDATA: c_int = 5;
/// Restart the stopped tracee process.
pub const PTRACE_CONT: c_int = 7;
/// Send the tracee a `SIGKILL` to terminate it.
/// Considered deprecated. Should just send `SIGKILL` directly instead.
pub const PTRACE_KILL: c_int = 8;
/// Restart the stopped tracee as for `PTRACE_CONT`, but arrange for the
/// tracee to be stopped after execution of a single instruction.
pub const PTRACE_SINGLESTEP: c_int = 9;
/// Copy the tracee's general-purpose registers to the address `data` in
/// the tracer.
pub const PTRACE_GETREGS: c_int = 12;
/// Modify the tracee's general-purpose registers from the address `data`
/// in the tracer.
pub const PTRACE_SETREGS: c_int = 13;
/// Copy the tracee's floating-point registers to the address `data` in
/// the tracer.
pub const PTRACE_GETFPREGS: c_int = 14;
/// Modify the tracee's floating-point registers from the address `data`
/// in the tracer.
pub const PTRACE_SETFPREGS: c_int = 15;
/// Attach to the process specified in `pid`, making it a tracee of the
/// calling process.
pub const PTRACE_ATTACH: c_int = 16;
/// Restart the stopped tracee as for `PTRACE_CONT`, but first detach from it.
pub const PTRACE_DETACH: c_int = 17;
/// Restart the stopped tracee as for `PTRACE_CONT`, but arrange for the
/// tracee to be stopped at the next entry to or exit from a system call.
pub const PTRACE_SYSCALL: c_int = 24;
/// Continue and stop on entry to the next system call, which will not be
/// executed.
pub const PTRACE_SYSEMU: c_int = 31;
/// Same as `PTRACE_SYSEMU` but also singlestep if not a system call.
pub const PTRACE_SYSEMU_SINGLESTEP: c_int = 32;

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/ptrace.2.html>.
///
/// Provides a means by which one process (the "tracer") may observe and
/// control the execution of another process (the "tracee"), and examine
/// and change the tracee's memory and registers.
///
/// The `__valist` argument represents 3 arguments in this order:
/// - pid
/// - *addr
/// - *data
///
/// Currently only implemented on x86_64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ptrace(request: c_int, mut __valist: ...) -> c_int {
    // Musl also just grabs the arguments from the varargs...
    unsafe {
        Sys::ptrace(
            request,
            __valist.next_arg(),
            __valist.next_arg(),
            __valist.next_arg(),
        )
    }
    .or_minus_one_errno()
}
