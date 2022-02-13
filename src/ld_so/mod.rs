use core::{mem, ptr};
use goblin::elf::program_header::{self, program_header32, program_header64, ProgramHeader};

use self::tcb::{Master, Tcb};
use crate::{
    header::sys_auxv::AT_NULL,
    platform::{Pal, Sys},
    start::Stack,
};

#[cfg(target_os = "redox")]
pub const PATH_SEP: char = ';';

#[cfg(target_os = "linux")]
pub const PATH_SEP: char = ':';

mod access;
pub mod callbacks;
pub mod debug;
mod dso;
pub mod linker;
pub mod start;
pub mod tcb;

static mut STATIC_TCB_MASTER: Master = Master {
    ptr: ptr::null_mut(),
    len: 0,
    offset: 0,
};

fn panic_notls(msg: impl core::fmt::Display) -> ! {
    eprintln!("panicked in ld.so: {}", msg);

    unsafe {
        core::intrinsics::abort();
    }
}

pub trait ExpectTlsFree {
    type Unwrapped;

    fn expect_notls(self, msg: &str) -> Self::Unwrapped;
}
impl<T, E: core::fmt::Debug> ExpectTlsFree for Result<T, E> {
    type Unwrapped = T;

    fn expect_notls(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(err) => panic_notls(format_args!("{}: expect failed for Result with err: {:?}", msg, err)),
        }
    }
}
impl<T> ExpectTlsFree for Option<T> {
    type Unwrapped = T;

    fn expect_notls(self, msg: &str) -> T {
        match self {
            Some(t) => t,
            None => panic_notls(format_args!("{}: expect failed for Option", msg)),
        }
    }
}

#[inline(never)]
pub fn static_init(sp: &'static Stack) {
    let mut phdr_opt = None;
    let mut phent_opt = None;
    let mut phnum_opt = None;

    let mut auxv = sp.auxv();
    loop {
        let (kind, value) = unsafe { *auxv };
        if kind == AT_NULL {
            break;
        }

        match kind {
            3 => phdr_opt = Some(value),
            4 => phent_opt = Some(value),
            5 => phnum_opt = Some(value),
            _ => (),
        }

        auxv = unsafe { auxv.add(1) };
    }

    let phdr = phdr_opt.expect_notls("failed to find AT_PHDR");
    let phent = phent_opt.expect_notls("failed to find AT_PHENT");
    let phnum = phnum_opt.expect_notls("failed to find AT_PHNUM");

    for i in 0..phnum {
        let ph_addr = phdr + phent * i;
        let ph: ProgramHeader = match phent {
            program_header32::SIZEOF_PHDR => {
                unsafe { *(ph_addr as *const program_header32::ProgramHeader) }.into()
            }
            program_header64::SIZEOF_PHDR => {
                unsafe { *(ph_addr as *const program_header64::ProgramHeader) }.into()
            }
            _ => panic_notls(format_args!("unknown AT_PHENT size {}", phent)),
        };

        let page_size = Sys::getpagesize();
        let voff = ph.p_vaddr as usize % page_size;
        let vaddr = ph.p_vaddr as usize - voff;
        let vsize = ((ph.p_memsz as usize + voff + page_size - 1) / page_size) * page_size;

        match ph.p_type {
            program_header::PT_TLS => {
                let valign = if ph.p_align > 0 {
                    ((ph.p_memsz + (ph.p_align - 1)) / ph.p_align) * ph.p_align
                } else {
                    ph.p_memsz
                } as usize;

                unsafe {
                    STATIC_TCB_MASTER.ptr = ph.p_vaddr as usize as *const u8;
                    STATIC_TCB_MASTER.len = ph.p_filesz as usize;
                    STATIC_TCB_MASTER.offset = valign;

                    let tcb = Tcb::new(vsize).expect_notls("failed to allocate TCB");
                    tcb.masters_ptr = &mut STATIC_TCB_MASTER;
                    tcb.masters_len = mem::size_of::<Master>();
                    tcb.copy_masters().expect_notls("failed to copy TLS master data");
                    tcb.activate();
                }

                //TODO: Warning on multiple TLS sections?
                return;
            }
            _ => (),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "redox"))]
pub unsafe fn init(sp: &'static Stack) {
    let mut tp = 0usize;

    #[cfg(target_os = "linux")]
    {
        const ARCH_GET_FS: usize = 0x1003;
        syscall!(ARCH_PRCTL, ARCH_GET_FS, &mut tp as *mut usize);
    }
    #[cfg(all(target_os = "redox", target_arch = "x86_64"))]
    {
        let mut env = syscall::EnvRegisters::default();

        let file = syscall::open("thisproc:current/regs/env", syscall::O_CLOEXEC | syscall::O_RDONLY)
            .expect_notls("failed to open handle for process registers");

        let _ = syscall::read(file, &mut env)
            .expect_notls("failed to read fsbase");

        let _ = syscall::close(file);

        tp = env.fsbase as usize;
    }

    if tp == 0 {
        static_init(sp);
    }
}

pub unsafe fn fini() {
    if let Some(tcb) = Tcb::current() {
        if tcb.linker_ptr != ptr::null_mut() {
            let linker = (&*tcb.linker_ptr).lock();
            linker.fini();
        }
    }
}
