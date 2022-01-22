use core::{ptr, slice};
use core::arch::global_asm;

use syscall::data::CloneInfo;

use crate::platform::{sys::e, types::*};

#[no_mangle]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    e(syscall::fpath(
        fd as usize,
        slice::from_raw_parts_mut(buf as *mut u8, count),
    )) as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn redox_physalloc(size: size_t) -> *mut c_void {
    let res = e(syscall::physalloc(size));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physfree(physical_address: *mut c_void, size: size_t) -> c_int {
    e(syscall::physfree(physical_address as usize, size)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn redox_physmap(
    physical_address: *mut c_void,
    size: size_t,
    flags: c_int,
) -> *mut c_void {
    let res = e(syscall::physmap(
        physical_address as usize,
        size,
        syscall::PhysmapFlags::from_bits(flags as usize).expect("physmap: invalid bit pattern"),
    ));
    if res == !0 {
        return ptr::null_mut();
    } else {
        return res as *mut c_void;
    }
}

#[no_mangle]
pub unsafe extern "C" fn redox_physunmap(virtual_address: *mut c_void) -> c_int {
    e(syscall::physunmap(virtual_address as usize)) as c_int
}

extern "C" {
    pub fn pte_clone_inner(info: *const CloneInfo) -> usize;
}

#[cfg(target_arch = "x86_64")]
global_asm!("
    .globl pte_clone_inner
    .type pte_clone_inner, @function
    .p2align 6",
    // Parameters: <info_ptr> in RDI
"pte_clone_inner:
    mov rax, {SYS_CLONE}
    mov rsi, rdi
    mov rdi, {CLONE_FLAGS}
    mov rdx, {INFO_LEN}",
    // Call clone(flags, info_ptr, info_len) syscall
    "syscall

    # Check if child or parent
    test rax, rax
    jnz .parent

    # Load registers
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9

    # Call entry point
    call rax

    # Exit
    mov rax, {SYS_EXIT}
    xor rdi, rdi
    syscall

    # Invalid instruction on failure to exit
    ud2

    # Return PID if parent
.parent:
    ret
    ",
    SYS_EXIT = const(syscall::SYS_EXIT),
    SYS_CLONE = const(syscall::SYS_CLONE),
    CLONE_FLAGS = const(
        syscall::CLONE_VM.bits()
            | syscall::CLONE_FS.bits()
            | syscall::CLONE_FILES.bits()
            | syscall::CLONE_SIGHAND.bits()
            | syscall::CLONE_STACK.bits()
    ),
    INFO_LEN = const(core::mem::size_of::<CloneInfo>()),
);

/*global_asm!("
    .globl pte_clone_inner
    .type pte_clone_inner, @function

pte_clone_inner:
    # Move the 1st argument `stack` of this function into the second argument to clone.
    mov rsi, rdi
    mov rax, {SYS_CLONE}
    mov rdi, {flags}

    # Call clone syscall
    syscall

    # Check if child or parent
    test rax, rax
    jnz 2f

    # Load registers
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9

    # Call entry point
    call rax

    # Exit
    mov rax, 1
    xor rdi, rdi
    syscall

    # Invalid instruction on failure to exit
    ud2

    # Return PID if parent
2:
    ret

    .size pte_clone_inner, . - pte_clone_inner

    ",

    flags = const(
        syscall::CLONE_VM.bits()
            | syscall::CLONE_FS.bits()
            | syscall::CLONE_FILES.bits()
            | syscall::CLONE_SIGHAND.bits()
            | syscall::CLONE_STACK.bits()
    ),
    SYS_CLONE = const(syscall::SYS_CLONE),
);*/
