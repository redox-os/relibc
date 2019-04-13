// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use c_str::CStr;
use header::unistd;
use platform::types::c_char;

use super::linker::Linker;

#[repr(C)]
pub struct Stack {
    argc: isize,
    argv0: *const c_char,
}

#[no_mangle]
pub extern "C" fn relibc_ld_so_start(sp: &'static mut Stack) -> usize {
    if sp.argc < 2 {
        eprintln!("ld.so [executable] [arguments...]");
        unistd::_exit(1);
        loop {}
    }

    // Pop the first argument (path to ld_so), and get the path of the program
    // TODO: Also retrieve LD_LIBRARY_PATH this way
    let path_c = unsafe {
        let mut argv = &mut sp.argv0 as *mut *const c_char;
        loop {
            let next_argv = argv.add(1);
            let arg = *next_argv;
            *argv = arg;
            argv = next_argv;
            if arg.is_null() {
                break;
            }
        }
        sp.argc -= 1;

        CStr::from_ptr(sp.argv0)
    };

    let path = match path_c.to_str() {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("ld.so: failed to parse path: {}", err);
            unistd::_exit(1);
            loop {}
        }
    };

    let mut linker = Linker::new("/usr/local/lib:/lib/x86_64-linux-gnu");
    match linker.load(&path, &path) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("ld.so: failed to load '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    }

    match linker.link(&path) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("ld.so: failed to link '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    }
}
