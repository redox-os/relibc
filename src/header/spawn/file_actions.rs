use core::{
    ffi::{c_char, c_int},
    ptr::{null, null_mut},
};

use crate::{
    error::{Errno, Result},
    header::{
        errno::{EBADF, EINVAL, ENOMEM},
        stdlib::{free, malloc},
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

pub struct OperationNode {
    pub operation: Operation,
    next: *const OperationNode,
}

#[repr(C)]
pub struct posix_spawn_file_actions_t {
    len: usize,
    head: *const OperationNode,
    tail: *mut OperationNode,
}

pub struct FileActionsIter<'a> {
    curr: Option<&'a OperationNode>,
}

impl<'a> Iterator for FileActionsIter<'a> {
    type Item = &'a OperationNode;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr?;
        self.curr = unsafe { curr.next.as_ref() };
        Some(curr)
    }
}

impl<'a> IntoIterator for &'a posix_spawn_file_actions_t {
    type Item = &'a OperationNode;

    type IntoIter = FileActionsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FileActionsIter {
            curr: unsafe { self.head.as_ref() },
        }
    }
}

fn copy_op(file_actions: &mut posix_spawn_file_actions_t, op: Operation) -> Result<()> {
    let new = unsafe { malloc(size_of::<OperationNode>()) };
    let new_ref = unsafe {
        (new as *const OperationNode)
            .as_ref()
            .ok_or(Errno(ENOMEM))?
    };

    if (*file_actions).head.is_null() && (*file_actions).tail.is_null() {
        (*file_actions).head = new as *const OperationNode;
        (*file_actions).tail = new as *mut OperationNode;
        (*file_actions).len += 1;
    } else {
        let tail = unsafe { (*file_actions).tail.as_mut().unwrap() };
        (*tail).next = new as *mut OperationNode;
        (*file_actions).tail = new as *mut OperationNode;
        (*file_actions).len += 1;
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

    (*file_actions).head = null();
    (*file_actions).tail = null_mut();
    (*file_actions).len = 0;

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

    if (*file_actions).head.is_null() {
        assert!((*file_actions).tail.is_null() && (*file_actions).len == 0);
        return 0;
    }

    let node = unsafe { (*file_actions).head.as_ref().unwrap() };

    while !(*file_actions).head.is_null() {
        let head = (*file_actions).head;
        let next = unsafe { (*head).next };
        (*file_actions).head = next;

        unsafe {
            free(head as *mut c_void);
        }
    }

    (*file_actions).tail = null_mut();
    (*file_actions).len = 0;

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addopen(
    file_actions: &mut posix_spawn_file_actions_t,
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
    file_actions: &mut posix_spawn_file_actions_t,
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
    file_actions: &mut posix_spawn_file_actions_t,
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
    file_actions: &mut posix_spawn_file_actions_t,
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
    file_actions: &mut posix_spawn_file_actions_t,
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
