// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use crate::{
    c_str::CStr,
    header::unistd,
    platform::types::c_char,
    start::Stack,
};

use super::linker::Linker;

#[no_mangle]
pub extern "C" fn relibc_ld_so_start(sp: &'static mut Stack) -> usize {
    if sp.argc < 2 {
        eprintln!("ld.so [executable] [arguments...]");
        unistd::_exit(1);
        loop {}
    }

    // Some variables that will be overridden by environment and auxiliary vectors
    let mut library_path = "/lib";
    //let mut page_size = 4096;

    // Pop the first argument (path to ld_so), and get the path of the program
    let path_c = unsafe {
        let mut argv = sp.argv() as *mut usize;

        // Move arguments
        loop {
            let next_argv = argv.add(1);
            let arg = *next_argv;
            *argv = arg;
            argv = next_argv;
            if arg == 0 {
                break;
            }

            if let Ok(arg_str) = CStr::from_ptr(arg as *const c_char).to_str() {
                println!("  arg: '{}'", arg_str);
            }
        }

        // Move environment
        loop {
            let next_argv = argv.add(1);
            let arg = *next_argv;
            *argv = arg;
            argv = next_argv;
            if arg == 0 {
                break;
            }

            if let Ok(arg_str) = CStr::from_ptr(arg as *const c_char).to_str() {
                println!("  env: '{}'", arg_str);

                let mut parts = arg_str.splitn(2, '=');
                if let Some(key) = parts.next() {
                    if let Some(value) = parts.next() {
                        if let "LD_LIBRARY_PATH" = key {
                            library_path = value
                        }
                    }
                }
            }
        }

        // Move auxiliary vectors
        loop {
            let next_argv = argv.add(1);
            let kind = *next_argv;
            *argv = kind;
            argv = next_argv;

            let next_argv = argv.add(1);
            let value = *next_argv;
            *argv = value;
            argv = next_argv;

            if kind == 0 {
                break;
            }

            println!("  aux: {}={:#x}", kind, value);
            //match kind {
            //    6 => page_size = value,
            //    _ => (),
            //}
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

    let mut linker = Linker::new(library_path);
    match linker.load(&path, &path) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("ld.so: failed to load '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    }

    let entry = match linker.link(Some(&path)) {
        Ok(ok) => match ok {
            Some(some) => some,
            None => {
                eprintln!("ld.so: failed to link '{}': missing entry", path);
                unistd::_exit(1);
                loop {}
            }
        },
        Err(err) => {
            eprintln!("ld.so: failed to link '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    };

    eprintln!("ld.so: entry '{}': {:#x}", path, entry);
    entry
}
