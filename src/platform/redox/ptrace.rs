//! Note: This module is not going to be clean. We're not going to be
//! able to follow the specs 100%. Linux ptrace is very, very,
//! different to Redox. Many people agree that Linux ptrace is bad, so
//! we are NOT going to bend our API for the sake of
//! compatibility. So, this module will be a hellhole.

#[allow(unused_imports)]
// i586, riscv64gc, aarch64: PalSignal, signal, sys_ptrace, core::mem, syscall, Pal, io::prelude
use super::super::{ERRNO, Pal, PalPtrace, PalSignal, Sys, types::*};
#[allow(unused_imports)]
#[cfg(target_arch = "aarch64")]
use crate::header::arch_aarch64_user::user_regs_struct; // unused
#[cfg(target_arch = "x86_64")]
use crate::header::arch_x64_user::user_regs_struct;
#[allow(unused_imports)]
use crate::{
    c_str::{CStr, CString},
    error::Errno,
    fs::File,
    header::{
        errno::{self as errnoh, EIO},
        fcntl, signal, sys_ptrace,
    },
    io::{self, prelude::*},
    raw_cell::RawCell,
    sync::Mutex,
};

use alloc::collections::{BTreeMap, btree_map::Entry};
#[allow(unused_imports)]
use core::mem;
#[allow(unused_imports)]
use syscall;

pub struct Session {
    pub first: bool,
    pub fpregs: File,
    pub mem: File,
    pub regs: File,
    pub tracer: File,
}
pub struct State {
    pub sessions: Mutex<BTreeMap<pid_t, Session>>,
}
impl State {
    fn new() -> Self {
        Self {
            sessions: Mutex::new(BTreeMap::new()),
        }
    }
}

#[thread_local]
static STATE: RawCell<Option<State>> = RawCell::new(None);

pub fn init_state() -> &'static State {
    // Safe due to STATE being thread_local (TODO: is it though?)
    unsafe {
        if STATE.unsafe_ref().is_none() {
            STATE.unsafe_set(Some(State::new()));
        }
        let state_ptr = STATE.unsafe_ref().as_ref().unwrap() as *const State;
        &*state_ptr
    }
}
pub fn is_traceme(pid: pid_t) -> bool {
    // Skip special PIDs (<=0)
    if pid <= 0 {
        return false;
    }
    File::open(
        CStr::borrow(&CString::new(format!("/scheme/chan/ptrace-relibc/{}/traceme", pid)).unwrap()),
        fcntl::O_PATH,
    )
    .is_ok()
}
pub fn get_session(
    sessions: &mut BTreeMap<pid_t, Session>,
    pid: pid_t,
) -> io::Result<&mut Session> {
    const NEW_FLAGS: c_int = fcntl::O_RDWR | fcntl::O_CLOEXEC;

    match sessions.entry(pid) {
        Entry::Vacant(entry) => {
            if is_traceme(pid) {
                Ok(entry.insert(Session {
                    first: true,
                    tracer: File::open(
                        CStr::borrow(&CString::new(format!("/scheme/proc/{}/trace", pid)).unwrap()),
                        NEW_FLAGS,
                    )?,
                    mem: File::open(
                        CStr::borrow(&CString::new(format!("/scheme/proc/{}/mem", pid)).unwrap()),
                        NEW_FLAGS,
                    )?,
                    regs: File::open(
                        CStr::borrow(
                            &CString::new(format!("/scheme/proc/{}/regs/int", pid)).unwrap(),
                        ),
                        NEW_FLAGS,
                    )?,
                    fpregs: File::open(
                        CStr::borrow(
                            &CString::new(format!("/scheme/proc/{}/regs/float", pid)).unwrap(),
                        ),
                        NEW_FLAGS,
                    )?,
                }))
            } else {
                ERRNO.set(errnoh::ESRCH);
                Err(io::last_os_error())
            }
        }
        Entry::Occupied(entry) => Ok(entry.into_mut()),
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn inner_ptrace(
    request: c_int,
    pid: pid_t,
    addr: *mut c_void,
    data: *mut c_void,
) -> io::Result<c_int> {
    //TODO: aarch64
    unimplemented!("inner_ptrace not implemented on aarch64");
}

#[cfg(target_arch = "x86")]
unsafe fn inner_ptrace(
    request: c_int,
    pid: pid_t,
    addr: *mut c_void,
    data: *mut c_void,
) -> io::Result<c_int> {
    //TODO: x86
    unimplemented!("inner_ptrace not implemented on x86");
}

#[cfg(target_arch = "x86_64")]
unsafe fn inner_ptrace(
    request: c_int,
    pid: pid_t,
    addr: *mut c_void,
    data: *mut c_void,
) -> io::Result<c_int> {
    let state = init_state();

    if request == sys_ptrace::PTRACE_TRACEME {
        // Mark this child as traced, parent will check for this marker file
        let pid = Sys::getpid();
        mem::forget(File::open(
            CStr::borrow(
                &CString::new(format!("/scheme/chan/ptrace-relibc/{}/traceme", pid)).unwrap(),
            ),
            fcntl::O_CREAT | fcntl::O_PATH | fcntl::O_EXCL,
        )?);
        return Ok(0);
    }

    let mut sessions = state.sessions.lock();
    let session = get_session(&mut sessions, pid)?;

    match request {
        sys_ptrace::PTRACE_CONT
        | sys_ptrace::PTRACE_SINGLESTEP
        | sys_ptrace::PTRACE_SYSCALL
        | sys_ptrace::PTRACE_SYSEMU
        | sys_ptrace::PTRACE_SYSEMU_SINGLESTEP => {
            Sys::kill(pid, signal::SIGCONT as _);

            // TODO: Translate errors
            let syscall = syscall::PTRACE_STOP_PRE_SYSCALL | syscall::PTRACE_STOP_POST_SYSCALL;
            (&mut &session.tracer).write(&match request {
                sys_ptrace::PTRACE_CONT => syscall::PtraceFlags::empty(),
                sys_ptrace::PTRACE_SINGLESTEP => syscall::PTRACE_STOP_SINGLESTEP,
                // Skip the first post-syscall when connected
                sys_ptrace::PTRACE_SYSCALL if session.first => syscall::PTRACE_STOP_PRE_SYSCALL,
                sys_ptrace::PTRACE_SYSCALL => syscall,
                // Skip the first post-syscall when connected
                sys_ptrace::PTRACE_SYSEMU if session.first => {
                    syscall::PTRACE_FLAG_IGNORE | syscall::PTRACE_STOP_PRE_SYSCALL
                }
                sys_ptrace::PTRACE_SYSEMU => syscall::PTRACE_FLAG_IGNORE | syscall,
                sys_ptrace::PTRACE_SYSEMU_SINGLESTEP => {
                    syscall::PTRACE_FLAG_IGNORE | syscall::PTRACE_STOP_SINGLESTEP
                }
                _ => unreachable!("unhandled ptrace request type {}", request),
            })?;

            session.first = false;
            Ok(0)
        }
        sys_ptrace::PTRACE_GETREGS => {
            let c_regs = unsafe { &mut *(data as *mut user_regs_struct) };
            let mut redox_regs = syscall::IntRegisters::default();
            (&mut &session.regs).read(&mut redox_regs)?;
            *c_regs = user_regs_struct {
                r15: redox_regs.r15 as _,
                r14: redox_regs.r14 as _,
                r13: redox_regs.r13 as _,
                r12: redox_regs.r12 as _,
                rbp: redox_regs.rbp as _,
                rbx: redox_regs.rbx as _,
                r11: redox_regs.r11 as _,
                r10: redox_regs.r10 as _,
                r9: redox_regs.r9 as _,
                r8: redox_regs.r8 as _,
                rax: redox_regs.rax as _,
                rcx: redox_regs.rcx as _,
                rdx: redox_regs.rdx as _,
                rsi: redox_regs.rsi as _,
                rdi: redox_regs.rdi as _,
                orig_rax: redox_regs.rax as _, // redox_regs.orig_rax as _,
                rip: redox_regs.rip as _,
                cs: redox_regs.cs as _,
                eflags: redox_regs.rflags as _,
                rsp: redox_regs.rsp as _,
                ss: redox_regs.ss as _,
                fs_base: 0, // fs_base: redox_regs.fs_base as _,
                gs_base: 0, // gs_base: redox_regs.gs_base as _,
                ds: 0,      // ds: redox_regs.ds as _,
                es: 0,      // es: redox_regs.es as _,
                fs: 0,      // fs: redox_regs.fs as _,
                gs: 0,      // gs: redox_regs.gs as _,
            };
            Ok(0)
        }
        sys_ptrace::PTRACE_SETREGS => {
            let c_regs = unsafe { &*(data as *mut user_regs_struct) };
            let redox_regs = syscall::IntRegisters {
                r15: c_regs.r15 as _,
                r14: c_regs.r14 as _,
                r13: c_regs.r13 as _,
                r12: c_regs.r12 as _,
                rbp: c_regs.rbp as _,
                rbx: c_regs.rbx as _,
                r11: c_regs.r11 as _,
                r10: c_regs.r10 as _,
                r9: c_regs.r9 as _,
                r8: c_regs.r8 as _,
                rax: c_regs.orig_rax as _, // c_regs.rax as _,
                rcx: c_regs.rcx as _,
                rdx: c_regs.rdx as _,
                rsi: c_regs.rsi as _,
                rdi: c_regs.rdi as _,
                // orig_rax: c_regs.orig_rax as _,
                rip: c_regs.rip as _,
                cs: c_regs.cs as _,
                rflags: c_regs.eflags as _,
                rsp: c_regs.rsp as _,
                ss: c_regs.ss as _,
                // fs_base: c_regs.fs_base as _,
                // gs_base: c_regs.gs_base as _,
                // ds: c_regs.ds as _,
                // es: c_regs.es as _,
                // fs: c_regs.fs as _,
                // gs: c_regs.gs as _,
            };
            (&mut &session.regs).write(&redox_regs)?;
            Ok(0)
        }
        _ => unimplemented!(),
    }
}

#[cfg(target_arch = "riscv64")]
fn inner_ptrace(
    request: c_int,
    pid: pid_t,
    addr: *mut c_void,
    data: *mut c_void,
) -> io::Result<c_int> {
    //TODO: riscv64
    unimplemented!("inner_ptrace not implemented on riscv64");
}

impl PalPtrace for Sys {
    #[allow(unused_unsafe)] // keeping inner unsafe fails x86_64, removing fails riscv cross-build
    unsafe fn ptrace(
        request: c_int,
        pid: pid_t,
        addr: *mut c_void,
        data: *mut c_void,
    ) -> Result<c_int, Errno> {
        unsafe { inner_ptrace(request, pid, addr, data) }
            .map_err(|err| Errno(err.raw_os_error().unwrap_or(EIO)))
    }
}
