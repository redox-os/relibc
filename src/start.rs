//! Startup code.

use alloc::{boxed::Box, vec::Vec};
use core::{intrinsics, ptr};
use generic_rt::ExpectTlsFree;

use crate::{
    ALLOCATOR,
    header::{libgen, stdio, stdlib},
    ld_so::{self, linker::Linker, tcb::Tcb},
    platform::{self, Pal, Sys, get_auxvs, types::*},
    raw_cell::RawCell,
    sync::mutex::Mutex,
};

#[repr(C)]
pub struct Stack {
    pub argc: isize,
    pub argv0: *const c_char,
}

impl Stack {
    pub fn argv(&self) -> *const *const c_char {
        &self.argv0 as *const _
    }

    pub fn envp(&self) -> *const *const c_char {
        unsafe { self.argv().offset(self.argc + 1) }
    }

    pub fn auxv(&self) -> *const (usize, usize) {
        unsafe {
            let mut envp = self.envp();
            while !(*envp).is_null() {
                envp = envp.add(1);
            }
            envp.add(1) as *const (usize, usize)
        }
    }
}

unsafe fn copy_string_array(array: *const *const c_char, len: usize) -> Vec<*mut c_char> {
    let mut vec = Vec::with_capacity(len + 1);
    for i in 0..len {
        let item = *array.add(i);
        let mut len = 0;
        while *item.add(len) != 0 {
            len += 1;
        }

        let buf = platform::alloc(len + 1) as *mut c_char;
        for i in 0..=len {
            *buf.add(i) = *item.add(i);
        }
        vec.push(buf);
    }
    vec.push(ptr::null_mut());
    vec
}

// Since Redox and Linux are so similar, it is easy to accidentally run a binary from one on the
// other. This will test that the current system is compatible with the current binary
#[unsafe(no_mangle)]
pub unsafe fn relibc_verify_host() {
    if !Sys::verify() {
        intrinsics::abort();
    }
}
#[unsafe(link_section = ".init_array")]
#[used]
static INIT_ARRAY: [extern "C" fn(); 1] = [init_array];

static mut init_complete: bool = false;

#[used]
#[unsafe(no_mangle)]
static mut __relibc_init_environ: *mut *mut c_char = ptr::null_mut();

fn alloc_init() {
    unsafe {
        if init_complete {
            return;
        }
    }
    unsafe {
        if let Some(tcb) = ld_so::tcb::Tcb::current() {
            if !tcb.mspace.is_null() {
                ALLOCATOR.get().write(tcb.mspace.read());
            }
        }
    }
}

extern "C" fn init_array() {
    // The thing is that we cannot guarantee if
    // init_array runs first or if relibc_start runs first
    // Still whoever gets to run first must initialize rust
    // memory allocator before doing anything else.

    unsafe {
        if init_complete {
            return;
        }
    }

    alloc_init();
    io_init();

    unsafe {
        if platform::environ.is_null() {
            platform::environ = __relibc_init_environ;
        }
    }

    unsafe {
        crate::pthread::init();
        init_complete = true
    }
}
fn io_init() {
    unsafe {
        // Initialize stdin/stdout/stderr.
        // TODO: const fn initialization of FILE
        stdio::stdin = stdio::default_stdin().get();
        stdio::stdout = stdio::default_stdout().get();
        stdio::stderr = stdio::default_stderr().get();
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn relibc_start_v1(
    sp: &'static Stack,
    main: unsafe extern "C" fn(
        argc: isize,
        argv: *mut *mut c_char,
        envp: *mut *mut c_char,
    ) -> c_int,
) -> ! {
    unsafe extern "C" {
        static __preinit_array_start: extern "C" fn();
        static __preinit_array_end: extern "C" fn();
        static __init_array_start: extern "C" fn();
        static __init_array_end: extern "C" fn();

        fn _init();
    }

    // Ensure correct host system before executing more system calls
    relibc_verify_host();

    #[cfg(target_os = "redox")]
    let thr_fd = redox_rt::proc::FdGuard::new(
        crate::platform::get_auxv_raw(sp.auxv().cast(), redox_rt::auxv_defs::AT_REDOX_THR_FD)
            .expect_notls("no thread fd present"),
    );

    // Initialize TLS, if necessary
    ld_so::init(
        sp,
        #[cfg(target_os = "redox")]
        thr_fd,
    );

    // Set up the right allocator...
    // if any memory rust based memory allocation happen before this step .. we are doomed.
    alloc_init();

    if let Some(tcb) = ld_so::tcb::Tcb::current() {
        // Update TCB mspace
        tcb.mspace = ALLOCATOR.get();

        // Set linker pointer if necessary
        if tcb.linker_ptr.is_null() {
            //TODO: get ld path
            let linker = Linker::new(ld_so::linker::Config::default());
            //TODO: load root object
            tcb.linker_ptr = Box::into_raw(Box::new(Mutex::new(linker)));
        }
        #[cfg(target_os = "redox")]
        redox_rt::signal::setup_sighandler(&tcb.os_specific, true);
    }

    // Set up argc and argv
    let argc = sp.argc;
    let argv = sp.argv();
    *platform::inner_argv.as_mut_ptr() = copy_string_array(argv, argc as usize);
    platform::argv = platform::inner_argv.unsafe_mut().as_mut_ptr();
    // Special code for program_invocation_name and program_invocation_short_name
    if let Some(arg) = platform::inner_argv.unsafe_ref().get(0) {
        platform::program_invocation_name = *arg;
        platform::program_invocation_short_name = libgen::basename(*arg);
    }
    // We check for NULL here since ld.so might already have initialized it for us, and we don't
    // want to overwrite it if constructors in .init_array of dependency libraries have called
    // setenv.
    if platform::environ.is_null() {
        // Set up envp
        let envp = sp.envp();
        let mut len = 0;
        while !(*envp.add(len)).is_null() {
            len += 1;
        }
        *platform::OUR_ENVIRON.as_mut_ptr() = copy_string_array(envp, len);
        platform::environ = platform::OUR_ENVIRON.unsafe_mut().as_mut_ptr();
    }

    let auxvs = get_auxvs(sp.auxv().cast());
    crate::platform::init(auxvs);

    init_array();

    // Run preinit array
    {
        let mut f = &__preinit_array_start as *const _;
        #[allow(clippy::op_ref)]
        while f < &__preinit_array_end {
            (*f)();
            f = f.offset(1);
        }
    }

    // Call init section
    #[cfg(not(target_arch = "riscv64"))] // risc-v uses arrays exclusively
    {
        _init();
    }

    // Run init array
    {
        let mut f = &__init_array_start as *const _;
        #[allow(clippy::op_ref)]
        while f < &__init_array_end {
            (*f)();
            f = f.offset(1);
        }
    }

    // not argv or envp, because programs like bash try to modify this *const* pointer :|
    stdlib::exit(main(argc, platform::argv, platform::environ));

    unreachable!();
}
