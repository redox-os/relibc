// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    c_str::CStr,
    header::{sys_auxv::AT_NULL, unistd},
    platform::{new_mspace, types::c_char},
    start::Stack,
    sync::mutex::Mutex,
    ALLOCATOR,
};

use super::{access::accessible, debug::_r_debug, linker::Linker, tcb::Tcb, PATH_SEP};
use crate::header::sys_auxv::{AT_ENTRY, AT_PHDR};
use goblin::elf::header::header64::SIZEOF_EHDR;

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
    while *ptr != AT_NULL {
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

fn resolve_path_name(
    name_or_path: &str,
    envs: &BTreeMap<String, String>,
) -> Option<(String, String)> {
    if accessible(name_or_path, unistd::F_OK) == 0 {
        return Some((
            name_or_path.to_string(),
            name_or_path
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string(),
        ));
    }
    if name_or_path.split("/").collect::<Vec<&str>>().len() != 1 {
        return None;
    }

    let env_path = envs.get("PATH")?;
    for part in env_path.split(PATH_SEP) {
        let path = if part.is_empty() {
            format!("./{}", name_or_path)
        } else {
            format!("{}/{}", part, name_or_path)
        };
        if accessible(&path, unistd::F_OK) == 0 {
            return Some((path.to_string(), name_or_path.to_string()));
        }
    }
    None
}
#[no_mangle]
pub extern "C" fn relibc_ld_so_start(sp: &'static mut Stack, ld_entry: usize) -> usize {
    // First thing we initialize the mspace
    ALLOCATOR.set_book_keeper(new_mspace());
    // next we get the arguments, the environment, and the auxilary vector
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
    let ld_library_path = envs.get("LD_LIBRARY_PATH").map(|s| s.to_owned());

    let name_or_path = if is_manual {
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
        argv[1].to_string()
    } else {
        argv[0].to_string()
    };

    let (path, name) = match resolve_path_name(&name_or_path, &envs) {
        Some((p, n)) => (p, n),
        None => {
            eprintln!("ld.so: failed to locate '{}'", name_or_path);
            unistd::_exit(1);
            loop {}
        }
    };

    // if we are not running in manual mode, then the main
    // program is already loaded by the kernel and we want
    // to use it. on redox, we treat it the same.
    let base_addr = {
        let mut base = None;
        if !is_manual && cfg!(not(target_os = "redox")) {
            let phdr = *auxv.get(&AT_PHDR).unwrap();
            if phdr != 0 {
                base = Some(phdr - SIZEOF_EHDR);
            }
        }
        base
    };
    let mut linker = Linker::new(ld_library_path);
    let entry = match linker.load_program(&path, base_addr) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("ld.so: failed to link '{}': {}", path, err);
            unistd::_exit(1);
            loop {}
        }
    };
    if let Some(tcb) = unsafe { Tcb::current() } {
        tcb.linker_ptr = Box::into_raw(Box::new(Mutex::new(linker)));
        tcb.mspace = ALLOCATOR.get_book_keeper();
    }
    if is_manual {
        eprintln!("ld.so: entry '{}': {:#x}", path, entry);
    }
    entry
}
