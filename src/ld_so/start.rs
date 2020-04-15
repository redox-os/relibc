// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use crate::{
    c_str::CStr, header::unistd, platform::types::c_char, start::Stack, sync::mutex::Mutex,
};

use super::{
    debug::_r_debug,
    linker::{Linker, DSO},
    tcb::Tcb,
};
use crate::header::sys_auxv::{AT_ENTRY, AT_PHDR};

unsafe fn get_argv(mut ptr: *const usize) -> (Vec<String>, *const usize) {
    //traverse the stack and collect argument vector
    let mut argv = Vec::new();
    while *ptr != 0 {
        let arg = *ptr;
        match CStr::from_ptr(arg as *const c_char).to_str() {
            Ok(arg_str) => argv.push(arg_str.to_owned()),
            _ => {
                eprintln!("ld.so: failed to parse argv[{}]", argv.len());
                unistd::_exit(1);
                loop {}
            }
        }
        ptr = ptr.add(1);
    }
    return (argv, ptr);
}

unsafe fn get_env(mut ptr: *const usize) -> (BTreeMap<String, String>, *const usize) {
    //traverse the stack and collect argument environment variables
    let mut envs = BTreeMap::new();
    while *ptr != 0 {
        let env = *ptr;
        if let Ok(arg_str) = CStr::from_ptr(env as *const c_char).to_str() {
            let mut parts = arg_str.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    envs.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        ptr = ptr.add(1);
    }
    return (envs, ptr);
}

unsafe fn get_auxv(mut ptr: *const usize) -> BTreeMap<usize, usize> {
    //traverse the stack and collect argument environment variables
    let mut auxv = BTreeMap::new();
    while *ptr != 0 {
        let kind = *ptr;
        ptr = ptr.add(1);
        let value = *ptr;
        ptr = ptr.add(1);
        auxv.insert(kind, value);
    }
    return auxv;
}

unsafe fn adjust_stack(sp: &'static mut Stack) {
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
            let mut parts = arg_str.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    if let "LD_LIBRARY_PATH" = key {
                        //library_path = value
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
    }

    sp.argc -= 1;
}
#[no_mangle]
pub extern "C" fn relibc_ld_so_start(sp: &'static mut Stack, ld_entry: usize) -> usize {
    // first we get the arguments, the environment, and the auxilary vector
    let (argv, envs, auxv) = unsafe {
        let argv_start = sp.argv() as *mut usize;
        let (argv, argv_end) = get_argv(argv_start);
        let (envs, envs_end) = get_env(argv_end.add(1));
        let auxv = get_auxv(envs_end.add(1));
        (argv, envs, auxv)
    };

    let is_manual = if let Some(img_entry) = auxv.get(&AT_ENTRY) {
        *img_entry == ld_entry
    } else {
        true
    };

    // we might need global lock for this kind of stuff
    unsafe {
        _r_debug.r_ldbase = ld_entry;
    }

    // Some variables that will be overridden by environment and auxiliary vectors
    let library_path = match envs.get("LD_LIBRARY_PATH") {
        Some(lib_path) => lib_path,
        None => "/lib",
    };

    let path = if is_manual {
        // ld.so is run directly by user and not via execve() or similar systemcall
        println!("argv: {:#?}", argv);
        println!("envs: {:#?}", envs);
        println!("auxv: {:#x?}", auxv);

        if sp.argc < 2 {
            eprintln!("ld.so [executable] [arguments...]");
            unistd::_exit(1);
            loop {}
        }
        unsafe { adjust_stack(sp) };
        &argv[1]
    } else {
        &argv[0]
    };
    // if we are not running in manual mode, then the main
    // program is already loaded by the kernel and we want
    // to use it.
    let program = {
        let mut pr = None;
        if !is_manual {
            let phdr = *auxv.get(&AT_PHDR).unwrap();
            if phdr != 0 {
                let p = DSO {
                    name: path.to_owned(),
                    entry_point: *auxv.get(&AT_ENTRY).unwrap(),
                    // The 0x40 is the size of Elf header not a good idea for different bit size
                    // compatiablility but it will always work on 64 bit systems,
                    base_addr: phdr - 0x40,
                };
                pr = Some(p);
            }
        }
        pr
    };
    let mut linker = Linker::new(library_path, false);
    match linker.load(&path, &path) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("ld.so: failed to load '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    }

    let entry = match linker.link(Some(&path), program) {
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

    if let Some(tcb) = unsafe { Tcb::current() } {
        tcb.linker_ptr = Box::into_raw(Box::new(Mutex::new(linker)));
    }
    if is_manual {
        eprintln!("ld.so: entry '{}': {:#x}", path, entry);
    }
    entry
}
