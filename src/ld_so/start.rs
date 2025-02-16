// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use generic_rt::ExpectTlsFree;

use crate::{
    c_str::CStr,
    header::unistd,
    platform::{get_auxv, get_auxvs, types::c_char},
    start::Stack,
    sync::mutex::Mutex,
    ALLOCATOR,
};

use super::{
    access::accessible,
    debug::_r_debug,
    linker::{Config, Linker},
    tcb::Tcb,
    PATH_SEP,
};
use crate::header::sys_auxv::{AT_ENTRY, AT_PHDR};

#[cfg(target_pointer_width = "32")]
pub const SIZEOF_EHDR: usize = 52;

#[cfg(target_pointer_width = "64")]
pub const SIZEOF_EHDR: usize = 64;

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
            }
        }
        ptr = ptr.add(1);
    }

    (argv, ptr)
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

    (envs, ptr)
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
    if accessible(name_or_path, unistd::F_OK).is_ok() {
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
        if accessible(&path, unistd::F_OK).is_ok() {
            return Some((path.to_string(), name_or_path.to_string()));
        }
    }
    None
}

// TODO: Make unsafe
#[no_mangle]
pub extern "C" fn relibc_ld_so_start(sp: &'static mut Stack, ld_entry: usize) -> usize {
    // Setup TCB for ourselves.
    unsafe {
        let tcb = Tcb::new(0).expect_notls("ld.so: failed to allocate bootstrap TCB");
        tcb.activate(todo!());

        #[cfg(target_os = "redox")]
        redox_rt::signal::setup_sighandler(&tcb.os_specific);
    }

    // We get the arguments, the environment, and the auxilary vector
    let (argv, envs, auxv) = unsafe {
        let argv_start = sp.argv() as *mut usize;
        let (argv, argv_end) = get_argv(argv_start);
        let (envs, envs_end) = get_env(argv_end.add(1));
        let auxv = get_auxvs(envs_end.add(1));
        (argv, envs, auxv)
    };

    unsafe {
        crate::platform::OUR_ENVIRON = envs
            .iter()
            .map(|(k, v)| {
                let mut var = Vec::with_capacity(k.len() + v.len() + 2);
                var.extend(k.as_bytes());
                var.push(b'=');
                var.extend(v.as_bytes());
                var.push(b'\0');
                let mut var = var.into_boxed_slice();
                let ptr = var.as_mut_ptr();
                core::mem::forget(var);
                ptr.cast()
            })
            .chain(core::iter::once(core::ptr::null_mut()))
            .collect::<Vec<_>>();

        crate::platform::environ = crate::platform::OUR_ENVIRON.as_mut_ptr();
    }

    let is_manual = if let Some(img_entry) = get_auxv(&auxv, AT_ENTRY) {
        img_entry == ld_entry
    } else {
        true
    };

    // we might need global lock for this kind of stuff
    unsafe {
        _r_debug.r_ldbase = ld_entry;
    }

    // TODO: Fix memory leak, although minimal.
    unsafe {
        crate::platform::init(auxv.clone());
    }

    let name_or_path = if is_manual {
        // ld.so is run directly by user and not via execve() or similar systemcall
        println!("argv: {:#?}", argv);
        println!("envs: {:#?}", envs);
        println!("auxv: {:#x?}", auxv);

        if sp.argc < 2 {
            eprintln!("ld.so [executable] [arguments...]");
            unistd::_exit(1);
        }
        unsafe { adjust_stack(sp) };
        argv[1].to_string()
    } else {
        argv[0].to_string()
    };

    let (path, _name) = match resolve_path_name(&name_or_path, &envs) {
        Some((p, n)) => (p, n),
        None => {
            eprintln!("ld.so: failed to locate '{}'", name_or_path);
            unistd::_exit(1);
        }
    };

    // if we are not running in manual mode, then the main
    // program is already loaded by the kernel and we want
    // to use it. on redox, we treat it the same.
    let base_addr = {
        let mut base = None;
        if !is_manual && cfg!(not(target_os = "redox")) {
            let phdr = get_auxv(&auxv, AT_PHDR).unwrap();
            if phdr != 0 {
                base = Some(phdr - SIZEOF_EHDR);
            }
        }
        base
    };
    let mut linker = Linker::new(Config::from_env(&envs));
    let entry = match linker.load_program(&path, base_addr) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("ld.so: failed to link '{}': {:?}", path, err);
            eprintln!("ld.so: enable debug output with `LD_DEBUG=all` for more information");
            unistd::_exit(1);
        }
    };
    if let Some(tcb) = unsafe { Tcb::current() } {
        tcb.linker_ptr = Box::into_raw(Box::new(Mutex::new(linker)));
        tcb.mspace = ALLOCATOR.get();
    }
    if is_manual {
        eprintln!("ld.so: entry '{}': {:#x}", path, entry);
    }
    entry
}
