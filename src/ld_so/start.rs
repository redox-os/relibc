// Start code adapted from https://gitlab.redox-os.org/redox-os/relibc/blob/master/src/start.rs

use core::slice;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use object::{
    NativeEndian,
    elf::{self, PT_DYNAMIC, PT_PHDR},
    read::elf::{Dyn as _, ProgramHeader as _},
};

use crate::{
    c_str::CStr,
    header::{
        elf::{AT_BASE, AT_ENTRY, AT_PHDR, AT_PHENT, AT_PHNUM},
        unistd,
    },
    ld_so::{
        dso::{
            DT_RELR, DT_RELRENT, DT_RELRSZ, Dyn, ProgramHeader, Rel, Rela, Relocation,
            RelocationKind, Relr, apply_relr,
        },
        linker::DebugFlags,
    },
    platform::{auxv_iter, get_auxvs, types::c_char},
    start::Stack,
    sync::mutex::Mutex,
};

use super::{
    PATH_SEP,
    access::accessible,
    debug::_r_debug,
    linker::{Config, Linker},
    tcb::Tcb,
};

use generic_rt::ExpectTlsFree;

#[cfg(target_pointer_width = "32")]
pub const SIZEOF_EHDR: usize = 52;

#[cfg(target_pointer_width = "64")]
pub const SIZEOF_EHDR: usize = 64;

unsafe fn get_argv(mut ptr: *const usize) -> (Vec<String>, *const usize) {
    //traverse the stack and collect argument vector
    let mut argv = Vec::new();
    while unsafe { *ptr != 0 } {
        let arg = unsafe { *ptr };
        match unsafe { CStr::from_ptr(arg as *const c_char).to_str() } {
            Ok(arg_str) => argv.push(arg_str.to_owned()),
            _ => {
                eprintln!("ld.so: failed to parse argv[{}]", argv.len());
                unistd::_exit(1);
            }
        }
        ptr = unsafe { ptr.add(1) };
    }

    (argv, ptr)
}

unsafe fn get_env(mut ptr: *const usize) -> (BTreeMap<String, String>, *const usize) {
    //traverse the stack and collect argument environment variables
    let mut envs = BTreeMap::new();
    while unsafe { *ptr != 0 } {
        let env = unsafe { *ptr };
        if let Ok(arg_str) = unsafe { CStr::from_ptr(env as *const c_char).to_str() } {
            let mut parts = arg_str.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    envs.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        ptr = unsafe { ptr.add(1) };
    }

    (envs, ptr)
}

#[allow(unsafe_op_in_unsafe_fn)]
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn relibc_ld_so_start(
    sp: &'static mut Stack,
    ld_entry: usize,
    dynamic: *const Dyn,
) -> usize {
    let mut at_phdr = None;
    let mut at_phnum = None;
    let mut at_phent = None;
    let mut at_base = None;
    let mut at_entry = None;
    for [kind, value] in unsafe { auxv_iter(sp.auxv().cast::<usize>()) } {
        match kind {
            AT_PHDR => at_phdr = Some(value as *const ProgramHeader),
            AT_PHNUM => at_phnum = Some(value),
            AT_PHENT => at_phent = Some(value),
            AT_BASE => at_base = Some(value),
            AT_ENTRY => at_entry = Some(value),
            _ => {}
        }
    }

    let at_phdr = at_phdr.unwrap();
    let at_phnum = at_phnum.expect("`AT_PHNUM` must be present if `AT_PHDR` is");
    let at_phent = at_phent.expect("`AT_PHENT` must be present if `AT_PHDR` is");
    assert!(!at_phdr.is_null() && at_phnum != 0 && at_phent == size_of::<ProgramHeader>());
    let phdrs = unsafe { slice::from_raw_parts(at_phdr, at_phnum) };

    let at_entry = at_entry.unwrap();
    let at_base = at_base.unwrap_or_default();

    let self_base = if at_base != 0 {
        at_base
    } else {
        let ph = phdrs
            .iter()
            .find(|ph| ph.p_type(NativeEndian) == PT_DYNAMIC as u32)
            .unwrap();
        unsafe { dynamic.byte_sub(ph.p_vaddr(NativeEndian) as usize) as usize }
    };

    let is_manual = at_entry == ld_entry; // Whether the dynamic linker was invoked as a command.

    let mut i = dynamic;
    let mut rela_ptr = None;
    let mut rela_len = None;
    let mut relr_ptr = None;
    let mut relr_len = None;
    let mut rel_ptr = None;
    let mut rel_len = None;
    loop {
        let entry = unsafe { &*i };
        let val = entry.d_val(NativeEndian);
        let ptr = val as *const u8;
        match entry.d_tag(NativeEndian) as u32 {
            elf::DT_NULL => break,
            elf::DT_RELA => rela_ptr = Some(ptr.cast::<Rela>()),
            elf::DT_RELASZ => rela_len = Some(val as usize / size_of::<Rela>()),
            elf::DT_RELAENT => {
                assert_eq!(val as usize, size_of::<Rela>(),);
            }
            elf::DT_REL => rel_ptr = Some(ptr.cast::<Rel>()),
            elf::DT_RELSZ => rel_len = Some(val as usize / size_of::<Rel>()),
            elf::DT_RELENT => {
                assert_eq!(val as usize, size_of::<Rel>());
            }
            DT_RELR => relr_ptr = Some(ptr.cast::<Relr>()),
            DT_RELRSZ => relr_len = Some(val as usize / size_of::<Relr>()),
            DT_RELRENT => {
                assert_eq!(val as usize, size_of::<Relr>());
            }
            _ => {}
        }
        i = unsafe { i.add(1) };
    }

    unsafe fn get_array<'a, T>(
        ptr: Option<*const T>,
        len: Option<usize>,
        base_addr: usize,
    ) -> &'a [T] {
        if let Some(ptr) = ptr {
            let len = len.expect("dynamic entry was present without it's corresponding size");
            unsafe { core::slice::from_raw_parts(ptr.byte_add(base_addr), len) }
        } else {
            assert!(len.is_none());
            &[]
        }
    }

    for reloc in unsafe { get_array::<Rela>(rela_ptr, rela_len, self_base) }
        .iter()
        .map(Relocation::from)
        .chain(
            unsafe { get_array::<Rel>(rel_ptr, rel_len, self_base) }
                .iter()
                .map(Relocation::from),
        )
    {
        let ptr = (reloc.offset + self_base) as *mut usize;
        match reloc.kind {
            RelocationKind::RELATIVE => {
                unsafe { ptr.write(self_base + reloc.addend.unwrap_or_default()) };
            }
            _ => {}
        }
    }

    unsafe {
        let relr = get_array(relr_ptr, relr_len, self_base);
        apply_relr(self_base as *const u8, relr);
    }

    let mut base_addr = None;
    if !is_manual {
        // if we are not running in manual mode, then the main
        // program is already loaded by the kernel and we want
        // to use it. on redox, we treat it the same.
        for ph in phdrs.iter() {
            if ph.p_type(NativeEndian) == PT_PHDR {
                assert!(base_addr.is_none(), "`PT_PHDR` cannot occur more than once");
                base_addr = Some(unsafe {
                    phdrs
                        .as_ptr()
                        .cast::<u8>()
                        .sub(ph.p_vaddr(NativeEndian) as usize)
                } as usize);
            }
        }
    }

    // Setup TCB for ourselves.
    unsafe {
        #[cfg(target_os = "redox")]
        let thr_fd =
            crate::platform::get_auxv_raw(sp.auxv().cast(), redox_rt::auxv_defs::AT_REDOX_THR_FD)
                .expect_notls("no thread fd present");

        let tcb = Tcb::new(0).expect_notls("[ld.so]: failed to allocate bootstrap TCB");
        tcb.activate(
            #[cfg(target_os = "redox")]
            Some(
                redox_rt::proc::FdGuard::new(thr_fd)
                    .to_upper()
                    .expect_notls("failed to move thread fd to upper table"),
            ),
        );
        #[cfg(target_os = "redox")]
        {
            let proc_fd = crate::platform::get_auxv_raw(
                sp.auxv().cast(),
                redox_rt::auxv_defs::AT_REDOX_PROC_FD,
            )
            .expect_notls("no proc fd present");

            let ns_fd = crate::platform::get_auxv_raw(
                sp.auxv().cast(),
                redox_rt::auxv_defs::AT_REDOX_NS_FD,
            )
            .filter(|&fd| fd != usize::MAX)
            .map(|fd| {
                redox_rt::proc::FdGuard::new(fd)
                    .to_upper()
                    .expect_notls("failed to move ns fd to upper table")
            });

            redox_rt::initialize(
                redox_rt::proc::FdGuard::new(proc_fd)
                    .to_upper()
                    .expect_notls("failed to move proc fd to upper table"),
                ns_fd,
            );
            redox_rt::signal::setup_sighandler(&tcb.os_specific, true);
        }
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
        crate::platform::OUR_ENVIRON.unsafe_set(
            envs.iter()
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
                .collect::<Vec<_>>(),
        );

        crate::platform::environ = crate::platform::OUR_ENVIRON.unsafe_mut().as_mut_ptr();
    }

    // we might need global lock for this kind of stuff
    _r_debug.lock().r_ldbase = self_base;

    // TODO: Fix memory leak, although minimal.
    #[cfg(target_os = "redox")]
    unsafe {
        crate::platform::init_inner(auxv.clone());
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
            eprintln!("[ld.so]: failed to locate '{name_or_path}'");
            unistd::_exit(1);
        }
    };

    let config = Config::from_env(&envs);
    if config.debug_flags.contains(DebugFlags::LOAD) || true {
        println!("[ld.so]: relocated self at {self_base:#x}!");
        if let Some(base_addr) = base_addr {
            println!("[ld.so]: executable has been already loaded at {base_addr:#x?}");
        }
    }

    let mut linker = Linker::new(config);
    let entry = match linker.load_program(&path, base_addr) {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("[ld.so]: failed to link '{path}': {err:?}");
            eprintln!("[ld.so]: enable debug output with `LD_DEBUG=all` for more information");
            unistd::_exit(1);
        }
    };
    if let Some(tcb) = unsafe { Tcb::current() } {
        tcb.linker_ptr = Box::into_raw(Box::new(Mutex::new(linker)));
    }
    if is_manual {
        eprintln!("[ld.so]: entry '{path}': {entry:#x}");
    }
    entry
}
