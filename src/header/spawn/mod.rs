//! `spawn.h` implementation
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/spawn.h.html>.

mod file_actions;
mod spawn_attr;

use core::str::FromStr;

use alloc::{
    ffi::CString,
    string::{String, ToString},
};
pub use file_actions::{Action, posix_spawn_file_actions_t};
pub use spawn_attr::{Flags, posix_spawnattr_t};

use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{
        dirent::{opendir, readdir},
        errno::{ENOENT, ENOTDIR},
        limits::PATH_MAX,
        stdlib::getenv,
        unistd::{chdir, getcwd},
    },
    iter::NulTerminated,
    platform::{
        self, ERRNO, Pal,
        types::{c_char, c_int, pid_t},
    },
};

unsafe fn spawn(
    pid: Option<&mut pid_t>,
    mut program: String,
    file_actions: Option<&posix_spawn_file_actions_t>,
    spawn_attr: Option<&posix_spawnattr_t>,
    argv: NulTerminated<*mut c_char>,
    envp: Option<NulTerminated<*mut c_char>>,
    use_path: bool,
) -> Result<()> {
    let mut original_cwd = [0 as c_char; PATH_MAX];
    assert!(
        unsafe { !getcwd(original_cwd.as_mut_ptr() as *mut c_char, PATH_MAX).is_null() },
        "Error getting cwd: {}",
        ERRNO.get()
    );

    let mut path_path = None;
    if use_path {
        let path = unsafe { getenv(c"PATH".as_ptr()) };
        let path_env = unsafe { CStr::from_nullable_ptr(path).unwrap().to_str().unwrap() };
        let path_elements = path_env.split(':');
        let mut flag = false;

        'a: for path_element in path_elements {
            let pe = CString::from_str(path_element).unwrap();
            let dir = if let Some(dir) =
                unsafe { opendir(pe.as_bytes_with_nul().as_ptr() as *const c_char).as_mut() }
            {
                dir
            } else if ERRNO.get() == 0 || ERRNO.get() == ENOTDIR || ERRNO.get() == ENOENT {
                continue;
            } else {
                return Err(Errno(ERRNO.get()));
            };

            while let Some(dir_ent) = unsafe { readdir(dir).as_ref() } {
                let dir_ent_name = unsafe {
                    CStr::from_ptr(dir_ent.d_name.as_ptr() as *const c_char)
                        .to_str()
                        .unwrap()
                        .to_string()
                };
                if dir_ent_name == program {
                    flag = true;
                    program = format!("{}/{}", path_element, program);
                    path_path = Some(path_element.to_string());
                    break 'a;
                }
            }
        }

        if !flag {
            return Err(Errno(ENOENT));
        }
    }

    let program = CString::from_str(program.as_str()).unwrap();

    unsafe {
        platform::Sys::spawn(
            CStr::from_bytes_with_nul(program.as_bytes_with_nul()).unwrap(),
            file_actions,
            spawn_attr,
            argv,
            envp,
            path_path,
        )
        .map(|v| {
            if let Some(pid) = pid {
                *pid = v;
            }
        })
        .map_err(|e| {
            let status = chdir(original_cwd.as_ptr() as *const c_char);
            if status != 0 {
                panic!("Error switching back to original cwd: {}", ERRNO.get());
            }
            e
        })?;
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn(
    pid: *mut pid_t,
    path: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let argv = {
        if argv.is_null() {
            panic!("argv cannot be NULL")
        } else {
            if unsafe { (*argv).is_null() } {
                panic!("argv must contain the program name");
            }
            unsafe { NulTerminated::new(argv).unwrap() }
        }
    };
    let envp = unsafe { NulTerminated::new(envp) };
    let program = unsafe {
        CStr::from_ptr(path)
            .to_str()
            .expect("path cannot be NULL")
            .to_string()
    };

    if let Err(e) = unsafe {
        spawn(
            pid.as_mut(),
            program,
            file_actions.as_ref(),
            attrp.as_ref(),
            argv,
            envp,
            false,
        )
    } {
        return e.0;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnp(
    pid: *mut pid_t,
    path: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let argv = {
        if argv.is_null() {
            panic!("argv cannot be NULL")
        } else {
            if unsafe { (*argv).is_null() } {
                panic!("argv must contain the program name");
            }
            unsafe { NulTerminated::new(argv).unwrap() }
        }
    };
    let envp = unsafe { NulTerminated::new(envp) };
    let program = unsafe {
        CStr::from_ptr(path)
            .to_str()
            .expect("path cannot be NULL")
            .to_string()
    };

    if let Err(e) = unsafe {
        spawn(
            pid.as_mut(),
            program.clone(),
            file_actions.as_ref(),
            attrp.as_ref(),
            argv,
            envp,
            if program.contains('/') { false } else { true },
        )
    } {
        return e.0;
    }
    0
}
