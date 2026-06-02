use core::{mem::ManuallyDrop, ptr::null_mut};

use alloc::{ffi::CString, vec::Vec};

use crate::{
    header::errno::EBADF,
    platform::types::{c_char, c_int, mode_t},
};

const OPEN: c_char = 1;
const CLOSE: c_char = 2;
const CHDIR: c_char = 3;
const FCHDIR: c_char = 4;
const DUP2: c_char = 5;

#[repr(C)]
#[derive(Debug, Clone)]
pub enum Action {
    Open {
        fd: c_int,
        path: CString,
        flag: c_int,
        mode: mode_t,
    },
    Close(c_int),
    Chdir(CString),
    FChdir(c_int),
    Dup2(c_int, c_int),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct posix_spawn_file_actions_t {
    file_actions: *mut Action,
    length: usize,
    capacity: usize,
}

impl posix_spawn_file_actions_t {
    pub fn add_action(&mut self, action: Action) {
        let v = unsafe { &mut Vec::from_raw_parts(self.file_actions, self.length, self.capacity) };
        v.push(action);
    }
}

pub struct FileActionsIter {
    actions: posix_spawn_file_actions_t,
    curr: usize,
}

impl Iterator for FileActionsIter {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        let actions = unsafe {
            &Vec::from_raw_parts(
                self.actions.file_actions,
                self.actions.length,
                self.actions.capacity,
            )
        };
        let e = actions.get(self.curr)?;
        self.curr += 1;
        Some((*e).clone())
    }
}

impl<'a> IntoIterator for &'a posix_spawn_file_actions_t {
    type Item = Action;

    type IntoIter = FileActionsIter;

    fn into_iter(self) -> Self::IntoIter {
        FileActionsIter {
            actions: (*self).clone(),
            curr: 0,
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_init(
    file_actions: &mut posix_spawn_file_actions_t,
) -> c_int {
    let mut v = ManuallyDrop::new(Vec::new());
    file_actions.file_actions = v.as_mut_ptr();
    file_actions.capacity = v.capacity();
    file_actions.length = 0;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(
    file_actions: &mut posix_spawn_file_actions_t,
) -> c_int {
    let v = unsafe {
        &Vec::from_raw_parts(
            file_actions.file_actions,
            file_actions.length,
            file_actions.capacity,
        )
    };
    file_actions.capacity = 0;
    file_actions.length = 0;
    file_actions.file_actions = null_mut();

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
    file_actions.add_action(Action::Open {
        fd,
        path: if path.is_null() {
            CString::new("").unwrap()
        } else {
            unsafe { CString::from_raw(path as *mut c_char) }
        },
        flag: oflag,
        mode,
    });
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
    file_actions.add_action(Action::Close(fd));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addchdir(
    file_actions: &mut posix_spawn_file_actions_t,
    path: *const c_char,
) -> c_int {
    file_actions.add_action(Action::Chdir(unsafe {
        if path.is_null() {
            CString::new("").unwrap()
        } else {
            CString::from_raw(path as *mut c_char)
        }
    }));
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
    file_actions.add_action(Action::FChdir(fd));
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
    file_actions.add_action(Action::Dup2(fd, new));
    0
}
