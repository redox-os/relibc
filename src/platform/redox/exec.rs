use crate::fs::File;

use syscall::error::Result;
use redox_exec::FdGuard;

pub fn fexec_impl(file: File, path: &[u8], args: &[&[u8]], envs: &[&[u8]], args_envs_size_without_nul: usize) -> Result<usize> {
    let fd = *file;
    core::mem::forget(file);
    let image_file = FdGuard::new(fd as usize);

    let open_via_dup = FdGuard::new(syscall::open("thisproc:current/open_via_dup", 0)?);

    let total_args_envs_size = args_envs_size_without_nul + args.len() + envs.len();
    let addrspace_selection_fd = redox_exec::fexec_impl(image_file, open_via_dup, path, args.iter().rev(), envs.iter().rev(), total_args_envs_size)?;

    // Dropping this FD will cause the address space switch.
    drop(addrspace_selection_fd);

    unreachable!();
}
