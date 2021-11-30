use alloc::vec::Vec;
use core::{intrinsics, ptr};

use crate::{
    header::{libgen, stdio, stdlib},
    ld_so,
    platform::{self, new_mspace, types::*, Pal, Sys},
    ALLOCATOR,
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
#[no_mangle]
pub unsafe fn relibc_verify_host() {
    if !Sys::verify() {
        intrinsics::abort();
    }
}
#[link_section = ".init_array"]
#[used]
static INIT_ARRAY: [extern "C" fn(); 1] = [init_array];

static mut init_complete: bool = false;

fn alloc_init() {
    unsafe {
        if let Some(tcb) = ld_so::tcb::Tcb::current() {
            if tcb.mspace != 0 {
                ALLOCATOR.set_book_keeper(tcb.mspace);
            } else if ALLOCATOR.get_book_keeper() == 0 {
                ALLOCATOR.set_book_keeper(new_mspace());
            }
        } else if ALLOCATOR.get_book_keeper() == 0 {
            ALLOCATOR.set_book_keeper(new_mspace());
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

    extern "C" {
        fn pthread_init();
    }
    unsafe {
        pthread_init();
        init_complete = true
    }
}
fn io_init() {
    unsafe {
        // Initialize stdin/stdout/stderr, see https://github.com/rust-lang/rust/issues/51718
        stdio::stdin = stdio::default_stdin.get();
        stdio::stdout = stdio::default_stdout.get();
        stdio::stderr = stdio::default_stderr.get();
    }
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn relibc_start(sp: &'static Stack) -> ! {
    extern "C" {
        static __preinit_array_start: extern "C" fn();
        static __preinit_array_end: extern "C" fn();
        static __init_array_start: extern "C" fn();
        static __init_array_end: extern "C" fn();

        fn _init();
        fn main(argc: isize, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
    }

    // Ensure correct host system before executing more system calls
    relibc_verify_host();

    // Initialize TLS, if necessary
    ld_so::init(sp);

    // Set up the right allocator...
    // if any memory rust based memory allocation happen before this step .. we are doomed.
    alloc_init();

    // Set up argc and argv
    let argc = sp.argc;
    let argv = sp.argv();
    platform::inner_argv = copy_string_array(argv, argc as usize);
    platform::argv = platform::inner_argv.as_mut_ptr();
    // Special code for program_invocation_name and program_invocation_short_name
    if let Some(arg) = platform::inner_argv.get(0) {
        platform::program_invocation_name = *arg;
        platform::program_invocation_short_name = libgen::basename(*arg);
    }

    // Set up envp
    let envp = sp.envp();
    let mut len = 0;
    while !(*envp.add(len)).is_null() {
        len += 1;
    }
    platform::inner_environ = copy_string_array(envp, len);
    platform::environ = platform::inner_environ.as_mut_ptr();

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
    _init();

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
