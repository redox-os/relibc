//! Note: This module is not going to be clean. We're not going to be
//! able to follow the specs 100%. Linux ptrace is very, very,
//! different to Redox. Many people agree that Linux ptrace is bad, so
//! we are NOT going to bend our API for the sake of
//! compatibility. So, this module will be a hellhole.

use super::super::types::*;
use super::super::{errno, Pal, PalPtrace, PalSignal, Sys};
use crate::c_str::CString;
use crate::fs::File;
use crate::header::sys_user::user_regs_struct;
use crate::header::{errno as errnoh, fcntl, signal, sys_ptrace};
use crate::io::{self, prelude::*};
use crate::sync::{Mutex, Once};
use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;
use syscall;

pub struct Session {
    pub tracer: File,
    pub mem: File,
    pub regs: File,
    pub fpregs: File
}
pub struct State {
    pub sessions: Mutex<BTreeMap<pid_t, Session>>
}
impl State {
    fn new() -> Self {
        Self {
            sessions: Mutex::new(BTreeMap::new())
        }
    }
}

static STATE: Once<State> = Once::new();

pub fn init_state() -> &'static State {
    STATE.call_once(|| State::new())
}

fn inner_ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> io::Result<c_int> {
    let state = init_state();

    if request == sys_ptrace::PTRACE_TRACEME {
        // let pid = Sys::getpid();
        // todo: only auto-open session on host if this happens
        return Ok(0);
    }

    const NEW_FLAGS: c_int = fcntl::O_RDWR | fcntl::O_CLOEXEC;

    let mut sessions = state.sessions.lock();
    let session = match sessions.entry(pid) {
        Entry::Vacant(entry) => entry.insert(Session {
            tracer: File::open(&CString::new(format!("proc:{}/trace", pid)).unwrap(), NEW_FLAGS | fcntl::O_NONBLOCK)?,
            mem: File::open(&CString::new(format!("proc:{}/mem", pid)).unwrap(), NEW_FLAGS)?,
            regs: File::open(&CString::new(format!("proc:{}/regs/int", pid)).unwrap(), NEW_FLAGS)?,
            fpregs: File::open(&CString::new(format!("proc:{}/regs/float", pid)).unwrap(), NEW_FLAGS)?,
        }),
        Entry::Occupied(entry) => entry.into_mut()
    };
    match request {
        sys_ptrace::PTRACE_CONT | sys_ptrace::PTRACE_SINGLESTEP |
        sys_ptrace::PTRACE_SYSCALL | sys_ptrace::PTRACE_SYSEMU |
        sys_ptrace::PTRACE_SYSEMU_SINGLESTEP => {
            Sys::kill(pid, signal::SIGCONT as _);

            (&mut &session.tracer).write(&[match request {
                sys_ptrace::PTRACE_CONT => syscall::PTRACE_CONT,
                sys_ptrace::PTRACE_SINGLESTEP => syscall::PTRACE_SINGLESTEP,
                sys_ptrace::PTRACE_SYSCALL => syscall::PTRACE_SYSCALL,
                sys_ptrace::PTRACE_SYSEMU => syscall::PTRACE_SYSEMU | syscall::PTRACE_SYSCALL,
                sys_ptrace::PTRACE_SYSEMU_SINGLESTEP => syscall::PTRACE_SYSEMU | syscall::PTRACE_SINGLESTEP,
                _ => unreachable!("unhandled ptrace request type {}", request)
            }])?;
            Ok(0)
        },
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
                ds: 0, // ds: redox_regs.ds as _,
                es: 0, // es: redox_regs.es as _,
                fs: redox_regs.fs as _,
                gs: 0, // gs: redox_regs.gs as _,
            };
            Ok(0)
        },
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
                fs: c_regs.fs as _,
                // gs: c_regs.gs as _,
            };
            (&mut &session.regs).write(&redox_regs)?;
            Ok(0)
        },
        _ => unimplemented!()
    }
}

impl PalPtrace for Sys {
    fn ptrace(request: c_int, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> c_int {
        inner_ptrace(request, pid, addr, data).unwrap_or(-1)
    }
}
