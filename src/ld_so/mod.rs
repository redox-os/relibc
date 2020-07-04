use goblin::elf::program_header::{self, program_header32, program_header64, ProgramHeader};

use self::tcb::{Master, Tcb};
use crate::start::Stack;
pub const PAGE_SIZE: usize = 4096;

mod access;
pub mod debug;
mod library;
pub mod linker;
pub mod start;
pub mod tcb;

pub fn static_init(sp: &'static Stack) {
    let mut phdr_opt = None;
    let mut phent_opt = None;
    let mut phnum_opt = None;

    let mut auxv = sp.auxv();
    loop {
        let (kind, value) = unsafe { *auxv };
        if kind == 0 {
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

    let phdr = phdr_opt.expect("failed to find AT_PHDR");
    let phent = phent_opt.expect("failed to find AT_PHENT");
    let phnum = phnum_opt.expect("failed to find AT_PHNUM");

    for i in 0..phnum {
        let ph_addr = phdr + phent * i;
        let ph: ProgramHeader = match phent {
            program_header32::SIZEOF_PHDR => {
                unsafe { *(ph_addr as *const program_header32::ProgramHeader) }.into()
            }
            program_header64::SIZEOF_PHDR => {
                unsafe { *(ph_addr as *const program_header64::ProgramHeader) }.into()
            }
            _ => panic!("unknown AT_PHENT size {}", phent),
        };

        let voff = ph.p_vaddr as usize % PAGE_SIZE;
        let vaddr = ph.p_vaddr as usize - voff;
        let vsize = ((ph.p_memsz as usize + voff + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

        match ph.p_type {
            program_header::PT_TLS => {
                let valign = if ph.p_align > 0 {
                    ((ph.p_memsz + (ph.p_align - 1)) / ph.p_align) * ph.p_align
                } else {
                    ph.p_memsz
                } as usize;

                let tcb_master = Master {
                    ptr: ph.p_vaddr as usize as *const u8,
                    len: ph.p_filesz as usize,
                    offset: vsize - valign,
                };

                unsafe {
                    let tcb = Tcb::new(vsize).expect("failed to allocate TCB");
                    tcb.set_masters(vec![tcb_master].into_boxed_slice());
                    tcb.copy_masters().expect("failed to copy TLS master data");
                    tcb.activate();
                }

                //TODO: Warning on multiple TLS sections?
                return;
            }
            _ => (),
        }
    }
}

#[cfg(target_os = "linux")]
pub unsafe fn init(sp: &'static Stack) {
    let mut tp = 0usize;
    const ARCH_GET_FS: usize = 0x1003;
    syscall!(ARCH_PRCTL, ARCH_GET_FS, &mut tp as *mut usize);
    if tp == 0 {
        static_init(sp);
    }
}

#[cfg(target_os = "redox")]
pub unsafe fn init(_sp: &'static Stack) {}
