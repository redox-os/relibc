#![no_std]
#![allow(internal_features)]
#![feature(
    core_intrinsics,
    int_roundings,
    let_chains,
    slice_ptr_get,
    sync_unsafe_cell
)]
#![forbid(unreachable_patterns)]

use core::{
    cell::UnsafeCell,
    mem::{MaybeUninit, size_of},
};

use generic_rt::{ExpectTlsFree, GenericTcb};
use syscall::Sigcontrol;

use self::{
    proc::{FdGuard, STATIC_PROC_INFO},
    protocol::ProcMeta,
    sync::Mutex,
};

extern crate alloc;

#[macro_export]
macro_rules! asmfunction(
    ($name:ident $(($($arg:ty),*))? $(-> $ret:ty)? : [$($asmstmt:expr),*$(,)?] <= [$($decl:ident = $(sym $symname:ident)?$(const $constval:expr)?),*$(,)?]$(,)? ) => {
        ::core::arch::global_asm!(concat!("
            .p2align 4
            .section .text.", stringify!($name), ", \"ax\", @progbits
            .globl ", stringify!($name), "
            .type ", stringify!($name), ", @function
        ", stringify!($name), ":
        ", $($asmstmt, "\n",)* "
            .size ", stringify!($name), ", . - ", stringify!($name), "
        "), $($decl = $(sym $symname)?$(const $constval)?),*);

        unsafe extern "C" {
            pub fn $name($($(_: $arg),*)?) $(-> $ret)?;
        }
    }
);

pub mod arch;
pub mod proc;

// TODO: Replace auxvs with a non-stack-based interface, but keep getauxval for compatibility
#[path = "../../src/platform/auxv_defs.rs"]
pub mod auxv_defs;

pub mod protocol;
pub mod signal;
pub mod sync;
pub mod sys;
pub mod thread;

#[derive(Debug, Default)]
pub struct RtTcb {
    pub control: Sigcontrol,
    pub arch: UnsafeCell<crate::arch::SigArea>,
    pub thr_fd: UnsafeCell<Option<FdGuard>>,
}
impl RtTcb {
    pub fn current() -> &'static Self {
        unsafe { &Tcb::current().unwrap().os_specific }
    }
    pub fn thread_fd(&self) -> &FdGuard {
        unsafe { (&*self.thr_fd.get()).as_ref().unwrap() }
    }
}

pub type Tcb = GenericTcb<RtTcb>;

/// OS and architecture specific code to activate TLS - Redox aarch64
#[cfg(target_arch = "aarch64")]
pub unsafe fn tcb_activate(_tcb: &RtTcb, tls_end: usize, tls_len: usize) {
    // Uses ABI page
    let abi_ptr = tls_end - tls_len - 16;
    core::ptr::write(abi_ptr as *mut usize, tls_end);
    core::arch::asm!(
        "msr tpidr_el0, {}",
        in(reg) abi_ptr,
    );
}

/// OS and architecture specific code to activate TLS - Redox x86
#[cfg(target_arch = "x86")]
pub unsafe fn tcb_activate(tcb: &RtTcb, tls_end: usize, _tls_len: usize) {
    let mut env = syscall::EnvRegisters::default();

    let file = FdGuard::new(
        syscall::dup(**tcb.thread_fd(), b"regs/env")
            .expect_notls("failed to open handle for process registers"),
    );

    let _ = syscall::read(*file, &mut env).expect_notls("failed to read gsbase");

    env.gsbase = tls_end as u32;

    let _ = syscall::write(*file, &env).expect_notls("failed to write gsbase");
}

/// OS and architecture specific code to activate TLS - Redox x86_64
#[cfg(target_arch = "x86_64")]
pub unsafe fn tcb_activate(tcb: &RtTcb, tls_end_and_tcb_start: usize, _tls_len: usize) {
    let mut env = syscall::EnvRegisters::default();

    let file = FdGuard::new(
        syscall::dup(**tcb.thread_fd(), b"regs/env")
            .expect_notls("failed to open handle for process registers"),
    );

    let _ = syscall::read(*file, &mut env).expect_notls("failed to read fsbase");

    env.fsbase = tls_end_and_tcb_start as u64;

    let _ = syscall::write(*file, &env).expect_notls("failed to write fsbase");
}

/// OS and architecture specific code to activate TLS - Redox riscv64
#[cfg(target_arch = "riscv64")]
pub unsafe fn tcb_activate(_tcb: &RtTcb, tls_end: usize, tls_len: usize) {
    // tp points to static tls block
    // FIXME limited to a single initial master
    let tls_start = tls_end - tls_len;
    let abi_ptr = tls_start - 8;
    core::ptr::write(abi_ptr as *mut usize, tls_end);
    core::arch::asm!(
        "mv tp, {}",
        in(reg) tls_start
    );
}

/// Initialize redox-rt in situations where relibc is not used
#[cfg(not(feature = "proc"))]
pub unsafe fn initialize_freestanding(this_thr_fd: FdGuard) -> &'static FdGuard {
    // TODO: This code is a hack! Integrate the ld_so TCB code into generic-rt, and then use that
    // (this function will need pointers to the ELF structs normally passed in auxvs), so the TCB
    // is initialized properly.

    // TODO: TLS
    let page = {
        &mut *(syscall::fmap(
            !0,
            &syscall::Map {
                offset: 0,
                size: syscall::PAGE_SIZE,
                flags: syscall::MapFlags::PROT_READ
                    | syscall::MapFlags::PROT_WRITE
                    | syscall::MapFlags::MAP_PRIVATE,
                address: 0,
            },
        )
        .unwrap() as *mut Tcb)
    };
    page.tcb_ptr = page;
    page.tcb_len = syscall::PAGE_SIZE;
    page.tls_end = (page as *mut Tcb).cast();

    // Make sure to use ptr::write to prevent dropping the existing FdGuard
    page.os_specific.thr_fd.get().write(Some(this_thr_fd));

    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        let tcb_addr = page as *mut Tcb as usize;
        tcb_activate(&page.os_specific, tcb_addr, 0)
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let abi_ptr = core::ptr::addr_of_mut!(page.tcb_ptr);
        core::arch::asm!("msr tpidr_el0, {}", in(reg) abi_ptr);
    }
    #[cfg(target_arch = "riscv64")]
    unsafe {
        let abi_ptr = core::ptr::addr_of_mut!(page.tcb_ptr) as usize;
        core::arch::asm!("mv tp, {}", in(reg) (abi_ptr + 8));
    }
    initialize();

    (*page.os_specific.thr_fd.get()).as_ref().unwrap()
}
pub(crate) fn read_proc_meta(proc: &FdGuard) -> syscall::Result<ProcMeta> {
    let mut bytes = [0_u8; size_of::<ProcMeta>()];
    let _ = syscall::read(**proc, &mut bytes)?;
    Ok(*plain::from_bytes::<ProcMeta>(&bytes).unwrap())
}
pub unsafe fn initialize(#[cfg(feature = "proc")] proc_fd: FdGuard) {
    #[cfg(feature = "proc")]
    let metadata = read_proc_meta(&proc_fd).unwrap();

    #[cfg(not(feature = "proc"))]
    // Bootstrap mode, don't associate proc fds with PIDs
    let metadata = ProcMeta::default();

    #[cfg(feature = "proc")]
    {
        crate::arch::PROC_FD.get().write(*proc_fd);
    }

    STATIC_PROC_INFO.get().write(StaticProcInfo {
        pid: metadata.pid,

        #[cfg(feature = "proc")]
        proc_fd: MaybeUninit::new(proc_fd),

        #[cfg(not(feature = "proc"))]
        proc_fd: MaybeUninit::uninit(),

        has_proc_fd: cfg!(feature = "proc"),
    });

    #[cfg(feature = "proc")]
    {
        *DYNAMIC_PROC_INFO.lock() = DynamicProcInfo {
            pgid: metadata.pgid,
            ruid: metadata.ruid,
            euid: metadata.euid,
            suid: metadata.suid,
            egid: metadata.egid,
            rgid: metadata.rgid,
            sgid: metadata.sgid,
        };
    }
}

#[repr(C)] // TODO: is repr(C) required?
pub(crate) struct StaticProcInfo {
    pid: u32,
    proc_fd: MaybeUninit<FdGuard>,
    has_proc_fd: bool,
}
pub struct DynamicProcInfo {
    pub pgid: u32,
    pub euid: u32,
    pub suid: u32,
    pub ruid: u32,
    pub egid: u32,
    pub rgid: u32,
    pub sgid: u32,
}

static DYNAMIC_PROC_INFO: Mutex<DynamicProcInfo> = Mutex::new(DynamicProcInfo {
    pgid: u32::MAX,
    ruid: u32::MAX,
    euid: u32::MAX,
    suid: u32::MAX,
    rgid: u32::MAX,
    egid: u32::MAX,
    sgid: u32::MAX,
});

#[inline]
pub(crate) fn static_proc_info() -> &'static StaticProcInfo {
    unsafe { &*STATIC_PROC_INFO.get() }
}
#[inline]
pub fn current_proc_fd() -> &'static FdGuard {
    let info = static_proc_info();
    assert!(info.has_proc_fd);
    unsafe { info.proc_fd.assume_init_ref() }
}

struct ChildHookCommonArgs {
    new_thr_fd: FdGuard,
    new_proc_fd: Option<FdGuard>,
}

unsafe fn child_hook_common(args: ChildHookCommonArgs) {
    // TODO: just pass PID to child rather than obtaining it via IPC?
    #[cfg(feature = "proc")]
    let metadata = read_proc_meta(
        args.new_proc_fd
            .as_ref()
            .expect("must be present with proc feature"),
    )
    .unwrap();

    #[cfg(not(feature = "proc"))]
    let metadata = ProcMeta::default();

    if let Some(proc_fd) = &args.new_proc_fd {
        crate::arch::PROC_FD.get().write(**proc_fd);
    }

    let old_proc_fd = STATIC_PROC_INFO
        .get()
        .replace(StaticProcInfo {
            pid: metadata.pid,
            has_proc_fd: args.new_proc_fd.is_some(),
            proc_fd: args
                .new_proc_fd
                .map_or_else(MaybeUninit::uninit, MaybeUninit::new),
        })
        .proc_fd;
    drop(old_proc_fd);

    let old_thr_fd = RtTcb::current().thr_fd.get().replace(Some(args.new_thr_fd));
    drop(old_thr_fd);
}
