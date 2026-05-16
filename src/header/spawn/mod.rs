//! `spawn.h` implementation
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/spawn.h.html>.

mod file_actions;
mod spawn_attr;

use alloc::string::{String, ToString};
pub use file_actions::{Operation, posix_spawn_file_actions_t};
pub use spawn_attr::{Flags, posix_spawnattr_t};

use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{
        dirent::{opendir, readdir},
        errno::ENOENT,
        stdlib::getenv,
    },
    platform::{
        self, Pal,
        types::{c_char, c_int, pid_t},
    },
};

fn spawn(
    pid: Option<&mut pid_t>,
    mut program: String,
    file_actions: Option<&posix_spawn_file_actions_t>,
    spawn_attr: Option<&posix_spawnattr_t>,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
    use_path: bool,
) -> Result<()> {
    if use_path {
        let path = unsafe { getenv(c"PATH".as_ptr()) };
        let path_env = unsafe { CStr::from_nullable_ptr(path).unwrap().to_str().unwrap() };
        let path_elements = path_env.split(':');
        let mut flag = false;

        for path_element in path_elements {
            let dir = if let Some(dir) =
                unsafe { opendir(path_element.as_bytes().as_ptr() as *const c_char).as_mut() }
            {
                dir
            } else {
                continue;
            };

            while let Some(dir_ent) = unsafe { readdir(dir).as_ref() } {
                let dir_ent_name = unsafe {
                    CStr::from_ptr(dir_ent.d_name.as_ptr() as *const c_char)
                        .to_str()
                        .unwrap()
                };
                if dir_ent_name == program {
                    flag = true;
                    program = format!("{}/{}", path_element, program);
                    break;
                }
            }
        }

        if !flag {
            return Err(Errno(ENOENT));
        }
    }

    unsafe {
        platform::Sys::spawn(
            CStr::from_bytes_with_nul_unchecked(program.as_bytes()),
            file_actions,
            spawn_attr,
            argv,
            envp,
            use_path,
        )
        .map(|v| {
            if let Some(pid) = pid {
                *pid = v;
            }
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_spawn(
    pid: *mut pid_t,
    program: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    spawn_attr: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let program = unsafe { CStr::from_ptr(program).to_str().unwrap().to_string() };

    if let Err(e) = spawn(
        unsafe { pid.as_mut() },
        program,
        unsafe { file_actions.as_ref() },
        unsafe { spawn_attr.as_ref() },
        argv,
        envp,
        false,
    ) {
        return e.0;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_spawnp(
    pid: *mut pid_t,
    program: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    spawn_attr: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let program = unsafe { CStr::from_ptr(program).to_str().unwrap().to_string() };

    if let Err(e) = spawn(
        unsafe { pid.as_mut() },
        program,
        unsafe { file_actions.as_ref() },
        unsafe { spawn_attr.as_ref() },
        argv,
        envp,
        true,
    ) {
        return e.0;
    }
    0
}
