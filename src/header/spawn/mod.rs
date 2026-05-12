//! `spawn.h` implementation
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/spawn.h.html>.

mod file_actions;
mod spawn_attr;

use core::ffi::c_char;

use crate::{
    header::spawn::{file_actions::posix_spawn_file_actions_t, spawn_attr::posix_spawnattr_t},
    platform::types::{c_int, pid_t},
};

#[unsafe(no_mangle)]
pub fn posix_spawn(
    pid: *const pid_t,
    path: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    spawn_attr: *const posix_spawnattr_t,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    todo!()
}
