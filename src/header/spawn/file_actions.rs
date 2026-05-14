use core::{
    ffi::{c_char, c_int},
    ptr::null,
};

use crate::{
    error::{Errno, Result},
    header::{
        errno::{EBADF, EINVAL, ENOMEM},
        stdlib::{free, malloc},
        string::memcpy,
    },
    platform::types::{c_void, mode_t},
};

const OPEN: c_char = 1;
const CLOSE: c_char = 2;
const CHDIR: c_char = 3;
const FCHDIR: c_char = 4;
const DUP2: c_char = 5;

#[repr(C)]
pub enum Operation {
    Open {
        fd: c_int,
        path: *const c_char,
        flag: c_int,
        mode: mode_t,
    },
    Close(c_int),
    Chdir(*const c_char),
    FChdir(c_int),
    Dup2(c_int, c_int),
}

#[repr(C)]
pub struct posix_spawn_file_actions_t {
    size: usize,
    operation: *const Operation,
}

fn copy_op(file_actions: *mut posix_spawn_file_actions_t, op: Operation) -> Result<()> {
    if file_actions.is_null() {
        return Err(Errno(EINVAL));
    }

    let new = unsafe { malloc((*file_actions).size + size_of::<Operation>()) };

    if new == -1isize as *mut c_void {
        return Err(Errno(ENOMEM));
    }

    unsafe {
        if (*file_actions).size >= size_of::<Operation>() {
            memcpy(
                new,
                (*file_actions).operation as *const c_void,
                (*file_actions).size / size_of::<Operation>(),
            );

            free((*file_actions).operation as *mut c_void);
        }

        (*file_actions).operation = new as *const Operation;

        let new = new as *mut Operation;

        (*new.add((*file_actions).size)) = op;
        (*file_actions).size += size_of::<Operation>()
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_init(
    file_actions: *mut posix_spawn_file_actions_t,
) -> c_int {
    let file_actions = match unsafe { file_actions.as_mut().ok_or(Errno(EINVAL)) } {
        Ok(v) => v,
        Err(_) => return EINVAL,
    };

    (*file_actions).operation = null();
    (*file_actions).size = 0;

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(
    file_actions: *mut posix_spawn_file_actions_t,
) -> c_int {
    let file_actions = match unsafe { file_actions.as_mut().ok_or(Errno(EINVAL)) } {
        Ok(v) => v,
        Err(_) => return EINVAL,
    };

    (*file_actions).operation = null();
    (*file_actions).size = 0;

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addopen(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
    path: *const c_char,
    oflag: c_int,
    mode: mode_t,
) -> c_int {
    if fd < 0 {
        return EBADF;
    }

    let open_op = Operation::Open {
        fd,
        path,
        flag: oflag,
        mode,
    };

    if let Err(e) = copy_op(file_actions, open_op) {
        return e.0;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addclose(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
) -> c_int {
    if fd < 0 {
        return EBADF;
    }

    let close_op = Operation::Close(fd);

    if let Err(e) = copy_op(file_actions, close_op) {
        return e.0;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addchdir(
    file_actions: *mut posix_spawn_file_actions_t,
    path: *const c_char,
) -> c_int {
    let chdir_op = Operation::Chdir(path);

    if let Err(e) = copy_op(file_actions, chdir_op) {
        return e.0;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addfchdir(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
) -> c_int {
    if fd < 0 {
        return EBADF;
    }

    let fchdir_op = Operation::FChdir(fd);

    if let Err(e) = copy_op(file_actions, fchdir_op) {
        return e.0;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
    new: c_int,
) -> c_int {
    if fd < 0 || new < 0 {
        return EBADF;
    }

    let dup2_op = Operation::Dup2(fd, new);

    if let Err(e) = copy_op(file_actions, dup2_op) {
        return e.0;
    }

    0
}
