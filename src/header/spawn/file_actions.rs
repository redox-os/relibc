use alloc::{ffi::CString, vec::Vec};
use core::ptr;

use crate::{
    header::errno::EBADF,
    platform::types::{c_char, c_int, mode_t, size_t},
};

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

struct FileActions(Vec<Action>);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct posix_spawn_file_actions_t {
    __relibc_internal_size: [u8; 24],
    __relibc_internal_align: size_t,
}

impl posix_spawn_file_actions_t {
    pub fn add_action(&mut self, action: Action) {
        let v = ptr::from_mut(self).cast::<FileActions>();
        unsafe {
            (*v).0.push(action);
        }
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
            ptr::from_mut(&mut self.actions)
                .cast::<FileActions>()
                .as_ref()
                .unwrap()
        };
        let e = actions.0.get(self.curr)?;
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_init.html>
///
/// Panics if `file_actions` is `NULL`.
#[unsafe(no_mangle)]
pub extern "C" fn posix_spawn_file_actions_init(
    file_actions: *mut posix_spawn_file_actions_t,
) -> c_int {
    if file_actions.is_null() {
        panic!("file_actions cannot be NULL");
    }
    let v = Vec::new();
    let actions = FileActions(v);
    unsafe {
        ptr::write(file_actions.cast::<FileActions>(), actions);
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_destroy.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// If `file_actions` is not `NULL`, then it must be a pointer to a `file_actions` object that was initialised by calling `posix_spawn_file_actions_init`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(
    file_actions: *mut posix_spawn_file_actions_t,
) -> c_int {
    if file_actions.is_null() {
        panic!("file_actions cannot be NULL");
    }
    unsafe {
        let _ = *(file_actions.cast::<FileActions>());
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_addopen.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addopen(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
    path: *const c_char,
    oflag: c_int,
    mode: mode_t,
) -> c_int {
    let file_actions = unsafe { file_actions.as_mut().expect("file_actions cannot be NULL") };
    if path.is_null() {
        panic!("path cannot be NULL");
    }
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_addclose.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// `file_actions` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addclose(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
) -> c_int {
    let file_actions = unsafe { file_actions.as_mut().expect("file_actions cannot be NULL") };
    if fd < 0 {
        return EBADF;
    }
    file_actions.add_action(Action::Close(fd));
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_addchdir.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// `file_actions` must be initialised and `path` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addchdir(
    file_actions: *mut posix_spawn_file_actions_t,
    path: *const c_char,
) -> c_int {
    let file_actions = unsafe { file_actions.as_mut().expect("file_actions cannot be NULL") };
    if path.is_null() {
        panic!("path cannot be NULL");
    }
    file_actions.add_action(Action::Chdir(unsafe {
        if path.is_null() {
            CString::new("").unwrap()
        } else {
            CString::from_raw(path as *mut c_char)
        }
    }));
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_addfchdir.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// `file_actions` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_addfchdir(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
) -> c_int {
    let file_actions = unsafe { file_actions.as_mut().expect("file_actions cannot be NULL") };
    if fd < 0 {
        return EBADF;
    }
    file_actions.add_action(Action::FChdir(fd));
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawn_file_actions_adddup2.html>
///
/// Panics if `file_actions` is `NULL`.
///
/// # Safety:
/// `file_actions` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2(
    file_actions: *mut posix_spawn_file_actions_t,
    fd: c_int,
    new: c_int,
) -> c_int {
    let file_actions = unsafe { file_actions.as_mut().expect("file_actions cannot be NULL") };
    if fd < 0 || new < 0 {
        return EBADF;
    }
    file_actions.add_action(Action::Dup2(fd, new));
    0
}
