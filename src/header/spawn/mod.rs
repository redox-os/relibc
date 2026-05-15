//! `spawn.h` implementation
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/spawn.h.html>.

mod file_actions;
mod spawn_attr;

pub use file_actions::{Operation, posix_spawn_file_actions_t};
pub use spawn_attr::{Flags, posix_spawnattr_t};

use crate::{
    c_str::CStr,
    error::{Errno, Result},
    header::{errno::ENOENT, stdlib::getenv},
    platform::{
        self, Pal,
        types::{c_char, c_int, pid_t},
    },
};

fn spawn(
    pid: Option<&mut pid_t>,
    mut program: &str,
    file_actions: Option<&posix_spawn_file_actions_t>,
    spawn_attr: Option<&posix_spawnattr_t>,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
    use_path: bool,
) -> Result<()> {
    if use_path {
        let path_env = unsafe {
            CStr::from_ptr(getenv("PATH".as_ptr() as *const c_char))
                .to_str()
                .unwrap()
        };
        let path_elements = path_env.split(':');
        let program_name = program.split("/").last().unwrap();
        let mut flag = false;

        for element in path_elements {
            if element.split("/").last().unwrap() == program_name {
                flag = true;
                program = element;
                break;
            }
            if !flag {
                return Err(Errno(ENOENT));
            }
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
    let program = unsafe { CStr::from_ptr(program).to_str().unwrap() };

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
    let program = unsafe { CStr::from_ptr(program).to_str().unwrap() };

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
