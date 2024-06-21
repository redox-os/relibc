#![no_std]
#![feature(asm_const, array_chunks, int_roundings, let_chains, slice_ptr_get, sync_unsafe_cell)]
#![forbid(unreachable_patterns)]

use generic_rt::{ExpectTlsFree, GenericTcb};

use self::signal::RtSigarea;

extern crate alloc;

#[macro_export]
macro_rules! asmfunction(
    ($name:ident $(-> $ret:ty)? : [$($asmstmt:expr),*$(,)?] <= [$($decl:ident = $(sym $symname:ident)?$(const $constval:expr)?),*$(,)?]$(,)? ) => {
        ::core::arch::global_asm!(concat!("
            .p2align 4
            .section .text.", stringify!($name), ", \"ax\", @progbits
            .globl ", stringify!($name), "
            .type ", stringify!($name), ", @function
        ", stringify!($name), ":
        ", $($asmstmt, "\n",)* "
            .size ", stringify!($name), ", . - ", stringify!($name), "
        "), $($decl = $(sym $symname)?$(const $constval)?),*);

        extern "C" {
            pub fn $name() $(-> $ret)?;
        }
    }
);

pub mod arch;
pub mod proc;

// TODO: Replace auxvs with a non-stack-based interface, but keep getauxval for compatibility
#[path = "../../src/platform/auxv_defs.rs"]
pub mod auxv_defs;

pub mod signal;
pub mod sync;
pub mod thread;

pub type Tcb = GenericTcb<RtSigarea>;

/// OS and architecture specific code to activate TLS - Redox aarch64
#[cfg(target_arch = "aarch64")]
pub unsafe fn tcb_activate(tls_end: usize, tls_len: usize) {
    // Uses ABI page
    let abi_ptr = tls_end - tls_len - 16;
    ptr::write(abi_ptr as *mut usize, tls_end);
    asm!(
        "msr tpidr_el0, {}",
        in(reg) abi_ptr,
    );
}

/// OS and architecture specific code to activate TLS - Redox x86
#[cfg(target_arch = "x86")]
pub unsafe fn tcb_activate(tls_end: usize, _tls_len: usize) {
    let mut env = syscall::EnvRegisters::default();

    let file = syscall::open(
        "thisproc:current/regs/env",
        syscall::O_CLOEXEC | syscall::O_RDWR,
    )
    .expect_notls("failed to open handle for process registers");

    let _ = syscall::read(file, &mut env).expect_notls("failed to read gsbase");

    env.gsbase = tls_end as u32;

    let _ = syscall::write(file, &env).expect_notls("failed to write gsbase");

    let _ = syscall::close(file);
}

/// OS and architecture specific code to activate TLS - Redox x86_64
#[cfg(target_arch = "x86_64")]
pub unsafe fn tcb_activate(tls_end_and_tcb_start: usize, _tls_len: usize) {
    let mut env = syscall::EnvRegisters::default();

    let file = syscall::open(
        "thisproc:current/regs/env",
        syscall::O_CLOEXEC | syscall::O_RDWR,
    )
    .expect_notls("failed to open handle for process registers");

    let _ = syscall::read(file, &mut env).expect_notls("failed to read fsbase");

    env.fsbase = tls_end_and_tcb_start as u64;

    let _ = syscall::write(file, &env).expect_notls("failed to write fsbase");

    let _ = syscall::close(file);
}

/// Initialize redox-rt in situations where relibc is not used
pub fn initialize_freestanding() {
    // TODO: TLS
    let page = unsafe {
        &mut *(syscall::fmap(!0, &syscall::Map {
            offset: 0,
            size: syscall::PAGE_SIZE,
            flags: syscall::MapFlags::PROT_READ | syscall::MapFlags::PROT_WRITE | syscall::MapFlags::MAP_PRIVATE,
            address: 0,
        }).unwrap() as *mut Tcb)
    };
    page.tcb_ptr = page;
    page.tcb_len = syscall::PAGE_SIZE;
    page.tls_end = (page as *mut Tcb).cast();

    unsafe {
        tcb_activate(page as *mut Tcb as usize, 0)
    }
}
